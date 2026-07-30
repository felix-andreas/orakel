#!/usr/bin/env python3
"""Find markets that resolved while we weren't looking.

Sweep the set of markets carrying an **unresolved prediction**, read from
`predictions/predictions.csv` minus `predictions/resolutions.csv`. That set is
the authority on what we owe a resolution for.

## Why not the watchlist

Because that is how we missed three rows. The CEO playbook mirrors the
watchlist at the start of a run, and `tools/watchlist` *drops markets that have
resolved* — correctly, since a resolved market should not be polled. The
resolution sweep then read the freshly mirrored watchlist. So the sweep was
structurally incapable of seeing anything that had resolved since the previous
run: mirroring removed exactly the markets the sweep existed to find.

On 2026-07-30 that hid two markets (three ledger rows,
`will-wti-reach-85-in-july-2026-from-july-27` and
`will-xauusd-dip-to-4000-by-july-27-2026`) which had resolved YES on 07-29 and
gone **against** us — so the omission flattered the trial's headline and would
have made the pre-registered completeness gate read unmet for a bookkeeping
reason rather than a real one.

Slot 1 put the general lesson better than the fix does: every check we had
asked *"is the archive complete as of the last run?"* and none asked *"did
something resolve while we weren't looking?"*

## Gamma quirks this relies on

- `closed` is a **filter in both directions**, not an include-flag: `closed=true`
  returns only closed markets, `closed=false` only open ones. Omitting it from a
  resolution sweep makes the sweep structurally unable to find anything. So we
  query both forms and union them, which also handles a **mixed batch** — on a
  resolution day UMA settles over hours, and a single-form query returns half the
  rows with no error and looks complete.
- `condition_ids` (plural) filters. **`condition_id` (singular) is silently
  ignored**, serving the unfiltered default list — a 200 with a plausible row that
  has nothing to do with the query. So every returned row's `conditionId` is
  asserted to be one we asked for.

Usage: sweep.py [--repo <root>]   — prints findings; never writes.
The CEO appends to `resolutions.csv` by hand, because that file is the firm's
evidence and a script should not be able to grow it unattended.
"""

import argparse
import csv
import json
import pathlib
import subprocess
import sys

GAMMA = "https://gamma-api.polymarket.com"
CHUNK = 20


def get(url: str):
    # curl, not urllib: the agent proxy 403s urllib's default requests.
    out = subprocess.run(
        ["curl", "-sS", "--retry", "3", "--retry-delay", "2", url],
        capture_output=True, text=True, timeout=180,
    )
    if out.returncode != 0:
        raise RuntimeError(f"curl failed: {out.stderr.strip()}")
    return json.loads(out.stdout)


def owed(repo: pathlib.Path):
    """(condition_id, slug, row_count) for every market we owe a resolution for."""
    resolved = {
        r["market_slug"].strip()
        for r in csv.DictReader(open(repo / "predictions" / "resolutions.csv"))
    }
    owed: dict[str, dict] = {}
    for r in csv.DictReader(open(repo / "predictions" / "predictions.csv")):
        slug = r["market_slug"].strip()
        if slug in resolved:
            continue
        e = owed.setdefault(slug, {"condition_id": r["condition_id"].strip(), "rows": 0})
        e["rows"] += 1
    return owed


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    args = ap.parse_args()
    repo = pathlib.Path(args.repo)

    o = owed(repo)
    print(f"{len(o)} markets carry an unresolved prediction ({sum(v['rows'] for v in o.values())} rows)")

    by_cid = {v["condition_id"]: s for s, v in o.items() if v["condition_id"]}
    missing_cid = [s for s, v in o.items() if not v["condition_id"]]
    if missing_cid:
        print(f"  ! {len(missing_cid)} have no condition_id in the ledger and cannot be swept:")
        for s in missing_cid:
            print(f"      {s}")

    cids = list(by_cid)
    closed, seen_open, identity_fail = {}, set(), []
    for i in range(0, len(cids), CHUNK):
        batch = cids[i:i + CHUNK]
        q = "&".join(f"condition_ids={c}" for c in batch)
        # Both forms, unioned — see module docstring.
        for flag, sink in (("true", closed), ("false", None)):
            for row in get(f"{GAMMA}/markets?{q}&closed={flag}&limit=100"):
                cid = row.get("conditionId")
                if cid not in batch:
                    identity_fail.append((row.get("slug"), cid))
                    continue
                if sink is None:
                    seen_open.add(cid)
                else:
                    sink[cid] = row

    print(f"  closed: {len(closed)}   open: {len(seen_open)}   "
          f"unaccounted: {len(cids) - len(closed) - len(seen_open)}")
    if identity_fail:
        print(f"  ! IDENTITY FAILURES ({len(identity_fail)}) — Gamma returned rows we did not ask for:")
        for slug, cid in identity_fail:
            print(f"      {slug} ({cid})")

    if not closed:
        print("\nNothing new resolved.")
        return 0

    print(f"\n{len(closed)} NEWLY RESOLVED — append these to predictions/resolutions.csv:")
    print("  (quote any note containing a comma; a malformed resolution row is a hard error)")
    for cid, row in sorted(closed.items(), key=lambda kv: kv[1].get("closedTime") or ""):
        slug = row.get("slug")
        outcomes = json.loads(row.get("outcomes") or "[]")
        prices = json.loads(row.get("outcomePrices") or "[]")
        winner = next(
            (name for name, p in zip(outcomes, prices) if str(p) in ("1", "1.0")),
            "UNCLEAR",
        )
        print(f"\n  {slug}")
        print(f"    rows owed     {o[slug]['rows']}")
        print(f"    condition_id  {cid}")
        print(f"    winner        {winner}   (outcomes {outcomes} prices {prices})")
        print(f"    closedTime    {row.get('closedTime')}")
        print(f"    umaStatus     {row.get('umaResolutionStatus')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
