"""Live CLOB books + Data-API tape for the open boards (Gate 5: capacity and book reality).

wiki/reference/thin-market-price-read.md: spread > 10c or top-of-book < $100 => the
midpoint is an artifact. Satellites are thin, so the book is diagnosed before any midpoint
is treated as tradeable.
"""

import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor

UA = "orakel-research/1.0"


def curl_json(url, tries=3):
    for i in range(tries):
        r = subprocess.run(["curl", "-s", "--max-time", "60", "-A", UA, url],
                           capture_output=True, text=True)
        if r.returncode == 0 and r.stdout.strip():
            try:
                return json.loads(r.stdout)
            except json.JSONDecodeError:
                pass
        time.sleep(1.0 * (i + 1))
    return None


def book_stats(tok):
    b = curl_json(f"https://clob.polymarket.com/book?token_id={tok}")
    if not b:
        return None
    bids = b.get("bids") or []
    asks = b.get("asks") or []
    # best bid / best ask are the LAST elements of each array
    bb = float(bids[-1]["price"]) if bids else None
    ba = float(asks[-1]["price"]) if asks else None
    bbs = float(bids[-1]["size"]) if bids else 0.0
    bas = float(asks[-1]["size"]) if asks else 0.0
    mid = (bb + ba) / 2 if (bb is not None and ba is not None) else None

    def depth(levels, lo, hi):
        s = 0.0
        for l in levels:
            p = float(l["price"])
            if lo <= p <= hi:
                s += p * float(l["size"])
        return s

    d10 = None
    if mid is not None:
        d10 = depth(bids, mid - 0.10, mid) + depth(asks, mid, mid + 0.10)
    return dict(
        best_bid=bb, best_ask=ba, mid=mid,
        spread=(ba - bb) if (bb is not None and ba is not None) else None,
        top_bid_usd=(bb * bbs) if bb else 0.0,
        top_ask_usd=(ba * bas) if ba else 0.0,
        depth_10c_usd=d10, n_bid_levels=len(bids), n_ask_levels=len(asks),
    )


def tape(cond, limit=500, pages=4):
    out = []
    for p in range(pages):
        d = curl_json(
            f"https://data-api.polymarket.com/trades?market={cond}&limit={limit}&offset={p*limit}"
        )
        if not d:
            break
        out += d
        if len(d) < limit:
            break
    return out


def main(root, out_path, only_open=True):
    boards = json.load(open(f"{root}/poly/boards.json"))
    sel = [b for b in boards if b["board_type"] and b["check_et"]
           and (not only_open or not b["closed"])]
    jobs = []
    for b in sel:
        for l in b["legs"]:
            if l["token_id"]:
                jobs.append((b["slug"], b["board_type"], b["check_et"], l))
    print(f"{len(jobs)} legs across {len(sel)} open boards")

    def job(j):
        slug, bt, chk, l = j
        st = book_stats(l["token_id"])
        return dict(slug=slug, board_type=bt, check_et=chk, company=l["company"],
                    condition_id=l["condition_id"], token_id=l["token_id"],
                    gamma_price=l["price"], volume=l["volume"], liquidity=l["liquidity"],
                    last=l["last"], book=st)

    res = []
    with ThreadPoolExecutor(max_workers=8) as ex:
        for i, r in enumerate(ex.map(job, jobs)):
            res.append(r)
            if (i + 1) % 50 == 0:
                print(f"  {i+1}/{len(jobs)}")
    json.dump(res, open(out_path, "w"), indent=1)
    print(f"-> {out_path}")

    # tape for the current cohort's boards only (capacity measurement)
    cur = [b for b in sel if b["check_et"].startswith("2026-07")]
    tapes = {}
    for b in cur:
        for l in b["legs"]:
            if l["condition_id"] and (l["volume"] or 0) > 100:
                tapes[l["condition_id"]] = tape(l["condition_id"])
    json.dump(tapes, open(out_path.replace(".json", "_tape.json"), "w"))
    print(f"-> tape for {len(tapes)} legs")


if __name__ == "__main__":
    main("strategies/arena-rank/satellites/data", sys.argv[1])
