#!/usr/bin/env python3
"""Friday scoring helper: resolve every outstanding ladder-rv ledger row, safely.

Why this is a script and not a paragraph of instructions: on 2026-07-31 the batch
is MIXED (boards close 21:00Z, UMA settles over hours) and every trap below has
already bitten this project once.

  1. Gamma's `closed` is a FILTER whose default is false, NOT an override.
     `?condition_ids=<cid>`             -> returns OPEN markets,   [] for closed
     `?condition_ids=<cid>&closed=true` -> returns CLOSED markets, [] for open
     No single query finds both. A one-form scorer drops half the batch SILENTLY,
     because [] is a valid 200. We union both forms.
  2. `?condition_id=` (SINGULAR) is not an error. Gamma ignores the unknown
     parameter and returns an arbitrary unrelated market with a 200. So we assert
     the returned conditionId equals the one we asked for, per market, and refuse
     to emit a resolution on any mismatch.
  3. `closedTime` is `2026-07-29 16:10:11+00` -- space separator, 2-digit offset,
     NOT RFC3339. Handled here; the Rust side was fixed 2026-07-30.
  4. `endDate` for the July monthlies is 2026-08-01T03:59:59.999Z. That is NOT the
     resolution window, which ends at the last session close, 2026-07-31 21:00Z.
     Never score a monthly off endDate.
  5. A leg can be `closed` with `outcomePrices` still ["0.5","0.5"] or absent --
     closed but not yet adjudicated. That is UNSETTLED, not a 50/50 outcome.

Usage:
    python3 scripts/resolve_sweep.py                  # report only
    python3 scripts/resolve_sweep.py --emit out.csv   # + resolutions.csv rows for the SETTLED ones

Exit code 0 always; read the summary. `--emit` writes only rows that passed every
check, in predictions/resolutions.csv column order. It never appends -- the CEO is
the single writer of predictions/.
"""

import argparse
import csv
import json
import os
import sys
import time
import urllib.request

GAMMA = "https://gamma-api.polymarket.com/markets"
_HERE = os.path.abspath(__file__)  # <repo>/strategies/barrier-touch/ladder-rv/scripts/this.py
REPO = os.path.normpath(os.path.join(os.path.dirname(_HERE), "..", "..", "..", ".."))
LEDGER = os.path.join(REPO, "predictions", "predictions.csv")
RESOLUTIONS = os.path.join(REPO, "predictions", "resolutions.csv")
VARIANT = "ladder-rv"


def get(url, tries=4):
    """GET with retries. A transient 5xx must never be read as an empty result."""
    last = None
    for i in range(tries):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "orakel-ladderrv/1.0"})
            with urllib.request.urlopen(req, timeout=30) as r:
                return json.load(r)
        except Exception as e:  # noqa: BLE001
            last = e
            time.sleep(1.5 * (i + 1))
    raise RuntimeError(f"GET failed after {tries} tries: {url}: {last}")


def lookup(cid):
    """Union of both query forms. Returns (market_or_None, note)."""
    hits = []
    for suffix in ("", "&closed=true"):
        for m in get(f"{GAMMA}?condition_ids={cid}{suffix}"):
            # TRAP 2: assert identity. Never trust that the answer is the question.
            if m.get("conditionId") != cid:
                return None, f"IDENTITY MISMATCH: asked {cid}, got {m.get('conditionId')}"
            hits.append(m)
    if not hits:
        return None, "not found in EITHER query form"
    # prefer the closed record if both forms answered (mid-transition)
    hits.sort(key=lambda m: bool(m.get("closed")), reverse=True)
    return hits[0], "ok"


def outcome_of(m):
    """('Yes'|'No'|None, note). None = closed but not adjudicated yet."""
    if not m.get("closed"):
        return None, "still OPEN"
    raw = m.get("outcomePrices")
    if isinstance(raw, str):
        try:
            raw = json.loads(raw)
        except Exception:  # noqa: BLE001
            raw = None
    if not raw or len(raw) < 2:
        return None, "closed, no outcomePrices yet (UMA lag)"
    try:
        yes = float(raw[0])
    except (TypeError, ValueError):
        return None, f"closed, unparseable outcomePrices {raw!r}"
    # TRAP 5: 0.5/0.5 is "not adjudicated", not a tie.
    if yes not in (0.0, 1.0):
        return None, f"closed, outcomePrices not final yet ({raw})"
    return ("Yes" if yes == 1.0 else "No"), "settled"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--emit", metavar="PATH", help="write resolutions rows for settled markets")
    args = ap.parse_args()

    rows = list(csv.DictReader(open(LEDGER)))
    ours = [r for r in rows if r["variant"] == VARIANT]
    done = {r["condition_id"] for r in csv.DictReader(open(RESOLUTIONS))}

    outstanding = {}
    for r in ours:
        if r["condition_id"] not in done:
            outstanding.setdefault(r["condition_id"], []).append(r)

    print(f"ledger rows (variant={VARIANT}): {len(ours)}")
    print(f"already in resolutions.csv:     {len(done)}")
    print(f"OUTSTANDING markets: {len(outstanding)}   rows: {sum(len(v) for v in outstanding.values())}")
    print()

    settled, unsettled, broken = [], [], []
    for cid, rs in sorted(outstanding.items(), key=lambda kv: kv[1][0]["market_slug"]):
        slug = rs[0]["market_slug"]
        try:
            m, note = lookup(cid)
        except RuntimeError as e:
            broken.append((slug, cid, len(rs), str(e)))
            continue
        if m is None:
            broken.append((slug, cid, len(rs), note))
            continue
        if m.get("slug") != slug:
            broken.append((slug, cid, len(rs), f"slug drift: ledger {slug}, gamma {m.get('slug')}"))
            continue
        oc, why = outcome_of(m)
        if oc is None:
            unsettled.append((slug, cid, len(rs), why))
        else:
            settled.append((slug, cid, len(rs), oc, m.get("closedTime") or ""))

    for label, items in (("SETTLED", settled), ("UNSETTLED", unsettled), ("NEEDS A HUMAN", broken)):
        print(f"=== {label}: {len(items)} markets, {sum(i[2] for i in items)} rows ===")
        for it in items:
            if label == "SETTLED":
                print(f"  {it[0]:<48} rows={it[2]:<3} -> {it[3]:<4} closedTime={it[4]}")
            else:
                print(f"  {it[0]:<48} rows={it[2]:<3} {it[3]}")
        print()

    n_rows = sum(len(v) for v in outstanding.values())
    n_settled_rows = sum(i[2] for i in settled)
    print(f"COMPLETENESS GATE (ops/state.toml): {n_settled_rows}/{n_rows} outstanding rows resolvable now.")
    print("The gate is met only when that is N/N and every row is APPENDED." if n_settled_rows != n_rows
          else "Gate condition met on the lookup side; append, then re-run to confirm 0 outstanding.")
    if broken:
        print(f"\n!! {len(broken)} market(s) need a human before any number is quoted. Do not average around them.")

    if args.emit:
        with open(args.emit, "w", newline="") as fh:
            w = csv.writer(fh)
            w.writerow(["market_slug", "condition_id", "winning_outcome", "resolved_date", "note"])
            for slug, cid, _n, oc, ct in settled:
                date = (ct or "")[:10] or "2026-07-31"
                w.writerow([slug, cid, oc, date, "gamma closed+outcomePrices, identity-checked"])
        print(f"\nwrote {len(settled)} candidate resolution rows -> {args.emit}")
        print("NOT appended. Hand to the CEO; predictions/ has one writer.")


if __name__ == "__main__":
    sys.exit(main())
