#!/usr/bin/env python3
"""Pull the GISTEMP monthly bucket-family markets from Polymarket (Gamma + CLOB + Data API).

Outputs a directory of raw JSON (one file per API response) plus a parsed legs.csv.
Everything raw gets tarred and frozen to R2 by the caller; analysis reads legs.csv +
the prices-history files.

Usage: python3 pull_markets.py <outdir>

Endpoints (wiki/recipes/polymarket-api.md):
  gamma  /public-search?q=...          series discovery
  gamma  /events?slug=...              event + markets metadata
  clob   /prices-history?market=<tok>&startTs=...&fidelity=60
  clob   /book?token_id=<tok>          live books (open events only)
  data   /trades?market=<cond>&limit=500&offset=N  tape (capacity gate)
"""

import csv
import json
import os
import sys
import time
import urllib.parse
import urllib.request

UA = {"User-Agent": "Mozilla/5.0 (research; orakel)"}
GAMMA = "https://gamma-api.polymarket.com"
CLOB = "https://clob.polymarket.com"
DATA = "https://data-api.polymarket.com"

# Monthly bucket-family instances (public-search 2026-07-24, q="temperature increase").
# No family exists for 2024-07 or 2025-08 (only "hottest on record" binaries those months).
MONTHLY_SLUGS = [
    "how-hot-will-april-2024-be",
    "may-2024-temperature-increase-c",
    "june-2024-temperature-increase-c",
    "august-2024-temperature-increase-c",
    "september-2024-temperature-increase-c",
    "october-2024-temperature-increase-c",
    "november-2024-temperature-increase-c",
    "december-2024-temperature-increase-c",
    "january-2025-temperature-increase-c",
    "february-2025-temperature-increase-c",
    "march-2025-temperature-increase-c",
    "april-2025-temperature-increase-c4",
    "april-2025-temperature-increase-c-lower-ranges",
    "may-2025-temperature-increase-c",
    "june-2025-temperature-increase-c-549",
    "july-2025-temperature-increase-c-394",
    "july-2025-temperature-increase-c-513",
    "september-2025-temperature-increase-c",
    "october-2025-temperature-increase-c",
    "october-2025-temperature-increase-c-577",
    "november-2025-temperature-increase-c",
    "december-2025-temperature-increase-c",
    "january-2026-temperature-increase-c",
    "february-2026-temperature-increase-c",
    "march-2026-temperature-increase-c",
    "april-2026-temperature-increase-c",
    "may-2026-temperature-increase-c",
    "june-2026-temperature-increase-c",
    "july-2026-temperature-increase-c-20260608140824583",  # LIVE
]

# Secondary applications / context (ranking + annual), live ones get books too.
EXTRA_SLUGS = [
    "2026-july-1st-2nd-3rd-hottest-on-record-20260706144334512",  # LIVE
    "where-will-2026-rank-among-the-hottest-years-on-record",  # LIVE annual
    "2026-june-1st-2nd-3rd-hottest-on-record",
    "2026-may-1st-2nd-3rd-hottest-on-record",
    "2026-april-1st-2nd-3rd-hottest-on-record",
    "2026-march-1st-2nd-3rd-hottest-on-record",
    "2026-january-1st-2nd-3rd-hottest-on-record",
]

# Tape pulls (capacity gate) restricted to recent resolved instances; page cap keeps
# the pull bounded — resolved events, newest-first, we page until before month start.
TAPE_SLUGS = [
    "june-2026-temperature-increase-c",
    "may-2026-temperature-increase-c",
    "april-2026-temperature-increase-c",
    "march-2026-temperature-increase-c",
    "february-2026-temperature-increase-c",
    "january-2026-temperature-increase-c",
    "july-2025-temperature-increase-c-513",
    "september-2025-temperature-increase-c",
]
TAPE_MAX_PAGES = 60  # 60*500 = 30k trades per market leg, far beyond need


def get(url: str, retries: int = 4) -> object:
    for i in range(retries):
        try:
            req = urllib.request.Request(url, headers=UA)
            with urllib.request.urlopen(req, timeout=45) as r:
                return json.load(r)
        except Exception as e:  # noqa: BLE001
            if i == retries - 1:
                raise
            time.sleep(1.5 * (i + 1))
            sys.stderr.write(f"retry {i+1} {url}: {e}\n")
    return None


def save(outdir: str, name: str, obj: object) -> None:
    with open(os.path.join(outdir, name), "w") as f:
        json.dump(obj, f)


def main() -> None:
    outdir = sys.argv[1]
    os.makedirs(outdir, exist_ok=True)

    # 1. Discovery snapshots (provenance for the slug lists above).
    for q in ["temperature increase", "hottest on record", "hottest year"]:
        d = get(f"{GAMMA}/public-search?q={urllib.parse.quote(q)}&limit_per_type=50")
        save(outdir, f"search_{q.replace(' ', '_')}.json", d)
        print(f"search '{q}': {len(d.get('events', []))} events")

    # 2. Event metadata.
    legs = []
    events_meta = {}
    for slug in MONTHLY_SLUGS + EXTRA_SLUGS:
        d = get(f"{GAMMA}/events?slug={urllib.parse.quote(slug)}")
        if not d:
            print(f"MISSING event {slug}")
            continue
        e = d[0]
        save(outdir, f"event_{slug}.json", e)
        events_meta[slug] = e
        for m in e.get("markets", []):
            toks = json.loads(m.get("clobTokenIds") or "[]")
            outs = json.loads(m.get("outcomes") or "[]")
            prices = json.loads(m.get("outcomePrices") or "[]")
            legs.append(
                {
                    "event_slug": slug,
                    "event_closed": e.get("closed"),
                    "event_start": e.get("startDate"),
                    "event_end": e.get("endDate"),
                    "event_closedTime": e.get("closedTime"),
                    "market_slug": m.get("slug"),
                    "question": m.get("question"),
                    "condition_id": m.get("conditionId"),
                    "market_closedTime": m.get("closedTime"),
                    "token_yes": toks[0] if toks else "",
                    "token_no": toks[1] if len(toks) > 1 else "",
                    "outcome_yes": outs[0] if outs else "",
                    "price_yes": prices[0] if prices else "",
                    "price_no": prices[1] if len(prices) > 1 else "",
                    "volume": m.get("volumeNum"),
                    "liquidity": m.get("liquidityNum"),
                    "start_date": m.get("startDate"),
                }
            )
        print(f"event {slug}: {len(e.get('markets', []))} markets, closed={e.get('closed')}")

    with open(os.path.join(outdir, "legs.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(legs[0].keys()))
        w.writeheader()
        w.writerows(legs)

    # 3. CLOB price history per Yes token (full series: startTs from event start).
    ph_dir = os.path.join(outdir, "prices_history")
    os.makedirs(ph_dir, exist_ok=True)
    n_ok = n_empty = 0
    for leg in legs:
        tok = leg["token_yes"]
        if not tok:
            continue
        start = leg["event_start"] or leg["start_date"]
        try:
            start_ts = int(
                time.mktime(time.strptime(start[:19], "%Y-%m-%dT%H:%M:%S"))
            ) - 86400
        except Exception:  # noqa: BLE001
            start_ts = int(time.time()) - 400 * 86400
        d = get(f"{CLOB}/prices-history?market={tok}&startTs={start_ts}&fidelity=60")
        hist = (d or {}).get("history", [])
        if not hist:
            d = get(f"{CLOB}/prices-history?market={tok}&startTs={start_ts}")
            hist = (d or {}).get("history", [])
        save(ph_dir, f"{tok}.json", {"history": hist})
        if hist:
            n_ok += 1
        else:
            n_empty += 1
    print(f"prices-history: {n_ok} ok, {n_empty} empty")

    # 4. Live books (open events only).
    book_dir = os.path.join(outdir, "books")
    os.makedirs(book_dir, exist_ok=True)
    for leg in legs:
        if leg["event_closed"]:
            continue
        for side in ("token_yes", "token_no"):
            tok = leg[side]
            if not tok:
                continue
            try:
                d = get(f"{CLOB}/book?token_id={tok}")
                save(book_dir, f"{tok}.json", d)
            except Exception as e:  # noqa: BLE001
                print(f"book fail {leg['market_slug']} {side}: {e}")
    print("books: done")

    # 5. Tape for capacity gate.
    tape_dir = os.path.join(outdir, "trades")
    os.makedirs(tape_dir, exist_ok=True)
    for leg in legs:
        if leg["event_slug"] not in TAPE_SLUGS:
            continue
        cond = leg["condition_id"]
        all_trades = []
        for page in range(TAPE_MAX_PAGES):
            d = get(f"{DATA}/trades?market={cond}&limit=500&offset={page * 500}")
            if not d:
                break
            all_trades.extend(d)
            if len(d) < 500:
                break
        save(tape_dir, f"{cond}.json", all_trades)
        print(f"tape {leg['market_slug']}: {len(all_trades)} trades")

    print("DONE")


if __name__ == "__main__":
    main()
