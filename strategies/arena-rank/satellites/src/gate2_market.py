"""Gate 2a — measure the satellite crowd's own precision on resolved instances.

Before building any model, ask what the market already knows (wiki/market-selection.md,
the gistemp-era5 rule). De-vig each board's legs at T-30/14/7/1d, score the crowd with
log-loss and Brier against the realised winner, and bucket the favourite's de-vigged price
against its realised win rate.

If the satellite boards are already calibrated, the idea dies here.
"""

import json
import math
import os
import sys
from collections import defaultdict
from datetime import timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from resolve import norm  # noqa: E402
from vintage import et_to_utc  # noqa: E402

CHECKPOINTS = [30, 14, 7, 1]
EPS = 1e-4


def price_at(hist, ts):
    """Last print at or before ts (unix seconds); None if the series starts later."""
    lo, hi, best = 0, len(hist) - 1, None
    while lo <= hi:
        mid = (lo + hi) // 2
        if hist[mid]["t"] <= ts:
            best = hist[mid]
            lo = mid + 1
        else:
            hi = mid - 1
    return best["p"] if best else None


def load_hist(clob_dir, token_id):
    p = f"{clob_dir}/{token_id}.json"
    if not os.path.exists(p):
        return None
    h = json.load(open(p)).get("history") or None
    if h:
        h.sort(key=lambda z: z["t"])
    return h


def devig(prices):
    s = sum(prices.values())
    if s <= 0:
        return None
    return {k: v / s for k, v in prices.items()}


def board_snapshot(board, clob_dir, when):
    ts = int(when.timestamp())
    raw = {}
    for l in board["legs"]:
        if not l["token_id"]:
            continue
        h = load_hist(clob_dir, l["token_id"])
        if not h:
            continue
        p = price_at(h, ts)
        if p is not None:
            raw[l["company"]] = p
    if len(raw) < 2:
        return None, None
    return raw, devig(raw)


def main(root, clob_dir):
    boards = json.load(open(f"{root}/poly/boards.json"))
    rows = []
    for b in boards:
        if not (b["closed"] and b["board_type"] and b["check_et"]):
            continue
        if not isinstance(b["winner"], str):
            continue
        T = et_to_utc(b["check_et"])
        for d in CHECKPOINTS:
            raw, dv = board_snapshot(b, clob_dir, T - timedelta(days=d))
            if not dv:
                continue
            win = None
            for k in dv:
                if norm(k) == norm(b["winner"]):
                    win = k
            if win is None:
                continue
            p = min(max(dv[win], EPS), 1 - EPS)
            fav = max(dv, key=dv.get)
            rows.append(
                dict(
                    slug=b["slug"],
                    board_type=b["board_type"],
                    check=b["check_et"],
                    volume=b["volume"],
                    d=d,
                    n_legs=len(dv),
                    overround=sum(raw.values()),
                    p_winner=p,
                    fav=fav,
                    p_fav=dv[fav],
                    fav_won=(norm(fav) == norm(b["winner"])),
                    logloss=-math.log(p),
                    brier=(1 - p) ** 2 + sum(v**2 for k, v in dv.items() if k != win),
                )
            )
    json.dump(rows, open(f"{root}/gate2_market.json", "w"), indent=1)
    return rows


def report(rows):
    def agg(rs):
        n = len(rs)
        return (
            n,
            sum(r["logloss"] for r in rs) / n,
            sum(r["brier"] for r in rs) / n,
            sum(r["fav_won"] for r in rs) / n,
            sum(r["p_fav"] for r in rs) / n,
        )

    print("--- crowd precision by checkpoint (ALL resolved boards) ---")
    print(f"{'T-d':>5} {'n':>4} {'logloss':>9} {'brier':>8} {'fav win%':>9} {'mean p_fav':>11}")
    for d in CHECKPOINTS:
        rs = [r for r in rows if r["d"] == d]
        if rs:
            n, ll, br, fw, pf = agg(rs)
            print(f"{d:5d} {n:4d} {ll:9.4f} {br:8.4f} {100*fw:8.1f}% {pf:11.3f}")

    print("\n--- satellites only (exclude the deep #1-overall anchor board) ---")
    sat = [r for r in rows if r["board_type"] != "text_overall_nosc_1"]
    print(f"{'T-d':>5} {'n':>4} {'logloss':>9} {'brier':>8} {'fav win%':>9} {'mean p_fav':>11}")
    for d in CHECKPOINTS:
        rs = [r for r in sat if r["d"] == d]
        if rs:
            n, ll, br, fw, pf = agg(rs)
            print(f"{d:5d} {n:4d} {ll:9.4f} {br:8.4f} {100*fw:8.1f}% {pf:11.3f}")

    print("\n--- modal calibration (favourite's de-vigged price vs realised win rate) ---")
    buckets = [(0, 0.5), (0.5, 0.7), (0.7, 0.85), (0.85, 0.95), (0.95, 1.01)]
    for lo, hi in buckets:
        rs = [r for r in sat if lo <= r["p_fav"] < hi]
        if rs:
            print(
                f"  p_fav [{lo:.2f},{hi:.2f}) n={len(rs):4d} mean p={sum(r['p_fav'] for r in rs)/len(rs):.3f}"
                f"  realised={sum(r['fav_won'] for r in rs)/len(rs):.3f}"
            )

    print("\n--- by board type, at T-7d ---")
    g = defaultdict(list)
    for r in rows:
        if r["d"] == 7:
            g[r["board_type"]].append(r)
    for k, rs in sorted(g.items(), key=lambda z: -len(z[1])):
        n, ll, br, fw, pf = agg(rs)
        print(f"  {k:28s} n={n:3d} logloss={ll:7.4f} brier={br:7.4f} favwin={100*fw:5.1f}% p_fav={pf:.3f}")

    print("\n--- overround (leg sum) by board type ---")
    g = defaultdict(list)
    for r in rows:
        g[r["board_type"]].append(r["overround"])
    for k, v in sorted(g.items(), key=lambda z: -len(z[1])):
        v.sort()
        print(f"  {k:28s} n={len(v):3d} median={v[len(v)//2]:.3f} p10={v[int(.1*len(v))]:.3f} p90={v[int(.9*len(v))]:.3f}")


if __name__ == "__main__":
    rows = main("strategies/arena-rank/satellites/data", sys.argv[1])
    print(f"{len(rows)} board-checkpoint observations\n")
    report(rows)
