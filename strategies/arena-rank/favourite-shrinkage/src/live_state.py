"""Live book + realised taker tape for the July cohort, on the day of the fundable-band test.

Three things the day-3 kill test needs and a book measurement alone cannot give:

  1. a FRESH book (yesterday's `[book]` blocks are already stale -- the Chinese favourite
     moved 0.8275 -> 0.7765 overnight), including the executable ask, not the midpoint
     (wiki/reference/midpoint-is-not-a-fill.md);
  2. the de-vigged LEG-SUM per board, which is the checkpoint-artifact gate
     (wiki/reference/checkpoint-artifact.md) -- an unpriced board manufactures edge;
  3. REALISED taker flow on the side we would take, in the band we would trade, from the
     Data API tape. `tools/fillcheck` folds No-side trades into Yes-equivalent units; the
     same fold is applied here (a No sell at q is a Yes buy at 1-q).

Writes data/live-<date>.json.
"""

import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor

UA = "orakel-research/1.0"
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BOARDS = json.load(open(f"{ROOT}/../satellites/data/poly/boards.json"))

COHORT = {
    "chinese": "best-chinese-ai-company-end-of-july",
    "math-1": "which-company-has-the-best-math-ai-model-end-of-july",
    "overall-nosc-2": "which-company-has-second-best-ai-model-end-of-july",
    "overall-nosc-3": "which-company-has-the-third-best-ai-model-end-of-july",
    "overall-sc-1": "which-company-has-1-ai-model-end-of-july-style-control-on",
    "overall-sc-2": "which-company-has-the-2-ai-model-end-of-july-style-control-on",
    "overall-sc-3": "which-company-has-the-3-ai-model-end-of-july-style-control-on",
}


def curl_json(url, tries=4):
    for i in range(tries):
        r = subprocess.run(["curl", "-s", "--max-time", "90", "-A", UA, url],
                           capture_output=True, text=True)
        if r.returncode == 0 and r.stdout.strip():
            try:
                return json.loads(r.stdout)
            except json.JSONDecodeError:
                pass
        time.sleep(1.0 * (i + 1))
    return None


def book(tok):
    """Best bid/ask are the LAST elements of each side (wiki/recipes/polymarket-api.md)."""
    b = curl_json(f"https://clob.polymarket.com/book?token_id={tok}")
    if not b:
        return None
    bids, asks = b.get("bids") or [], b.get("asks") or []
    bb = float(bids[-1]["price"]) if bids else None
    ba = float(asks[-1]["price"]) if asks else None
    mid = (bb + ba) / 2 if (bb is not None and ba is not None) else None

    def usd(levels, lo, hi):
        return sum(float(l["price"]) * float(l["size"]) for l in levels
                   if lo <= float(l["price"]) <= hi)

    def shares(levels, lo, hi):
        return sum(float(l["size"]) for l in levels if lo <= float(l["price"]) <= hi)

    out = dict(best_bid=bb, best_ask=ba, mid=mid,
               spread=(ba - bb) if (bb is not None and ba is not None) else None,
               n_bid_levels=len(bids), n_ask_levels=len(asks))
    if mid is not None:
        out["depth_10c_usd"] = usd(bids, mid - .10, mid) + usd(asks, mid, mid + .10)
        # the side WE take when buying the favourite: asks at or below mid+5c
        out["ask_depth_5c_usd"] = usd(asks, mid, mid + .05)
        out["ask_depth_5c_shares"] = shares(asks, mid, mid + .05)
        out["ask_shares_at_touch"] = float(asks[-1]["size"]) if asks else 0.0
    return out


def trades(cond):
    """Full tape for one leg. Data API pages 500 at a time."""
    out, off = [], 0
    while True:
        page = curl_json("https://data-api.polymarket.com/trades"
                         f"?market={cond}&limit=500&offset={off}")
        if not page:
            break
        out += page
        if len(page) < 500:
            break
        off += 500
        if off > 20000:
            break
    return out


def main():
    date = sys.argv[1] if len(sys.argv) > 1 else "2026-07-26"
    by_slug = {b["slug"]: b for b in BOARDS}
    res = {}
    for key, slug in COHORT.items():
        b = by_slug[slug]
        legs = [l for l in b["legs"] if l["token_id"]]
        with ThreadPoolExecutor(8) as ex:
            books = list(ex.map(lambda l: book(l["token_id"]), legs))
        with ThreadPoolExecutor(6) as ex:
            tapes = list(ex.map(lambda l: trades(l["condition_id"]), legs))
        res[key] = dict(slug=slug, board_type=b["board_type"], check_et=b["check_et"],
                        legs=[dict(company=l["company"], slug=l["slug"],
                                   condition_id=l["condition_id"], token_id=l["token_id"],
                                   book=bk, trades=tp)
                              for l, bk, tp in zip(legs, books, tapes)])
        n_tr = sum(len(t) for t in tapes)
        print(f"{key:16s} legs={len(legs):3d} books={sum(1 for x in books if x):3d} "
              f"trades={n_tr}", flush=True)
    json.dump(res, open(f"{ROOT}/data/live-{date}.json", "w"))
    print("wrote", f"{ROOT}/data/live-{date}.json")


if __name__ == "__main__":
    main()
