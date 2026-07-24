#!/usr/bin/env python3
"""Gate 5: capacity from the tape on resolved instances + today's live books.

For each tape event: in the late window [day21, close], measure
  (a) total taker notional printed at token prices in the fundable zone (3-50c), and
  (b) the subset on legs where the backtest had a signal, on the side we would have
      quoted, at prices leaving >= 2c of model edge — an upper bound on what a passive
      quoter could have absorbed.
Then read today's July-2026 books: top-of-book and within-5c depth per leg.

Usage: python3 capacity.py <pulldir> <bt_outdir>
"""

import csv
import json
import os
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone

FUNDABLE = (0.03, 0.50)
EDGE_MIN = 0.02

TAPE_EVENTS = [
    "july-2025-temperature-increase-c-513",
    "september-2025-temperature-increase-c",
    "january-2026-temperature-increase-c",
    "february-2026-temperature-increase-c",
    "march-2026-temperature-increase-c",
    "april-2026-temperature-increase-c",
    "may-2026-temperature-increase-c",
    "june-2026-temperature-increase-c",
]


def main():
    pulldir, btdir = sys.argv[1], sys.argv[2]
    legs = list(csv.DictReader(open(os.path.join(pulldir, "legs.csv"))))
    records = json.load(open(os.path.join(btdir, "checkpoint_records.json")))

    # signals per (slug, market_slug): list of (ckpt, dir, model_p)
    sig = defaultdict(list)
    for r in records:
        if r["ckpt"] not in ("day21", "month_end", "preprint") or not r["market_probs"]:
            continue
        for i, ms in enumerate([l["market_slug"] for l in legs
                                if l["event_slug"] == r["slug"]]):
            pass  # market slugs come from legs below instead
    # rebuild per-record leg lists from legs.csv ordering used in backtest (sorted by lo)
    # simpler: recompute signals from model_probs vs mids with the same threshold
    for r in records:
        if r["ckpt"] not in ("day21", "month_end", "preprint") or not r["market_probs"]:
            continue
        ev_legs = [l for l in legs if l["event_slug"] == r["slug"]]
        # order legs as in backtest: by bucket lower bound; reparse cheaply by mids match
        if len(r["mids"]) != len(ev_legs):
            continue
        # match by mid values is fragile; instead store signals by index and map through
        # sorted-bucket order reconstructed identically (lexicographic on question is not
        # stable) -> use model_probs order == backtest bucket order == sorted by lo.
        # We re-sort ev_legs by the numeric bound in the question.
        import re

        def lo_key(l):
            q = l["question"].replace("–", "-")
            nums = re.findall(r"(\d+\.\d+)", q)
            if "less than" in q:
                return -999.0
            return float(nums[0]) if nums else 999.0

        ev_legs.sort(key=lo_key)
        for i, l in enumerate(ev_legs):
            mp, mid = r["model_probs"][i], r["mids"][i]
            if mid is None:
                continue
            if mp - mid > 0.06:
                sig[(r["slug"], l["market_slug"])].append(("BUY_YES", mp))
            elif mid - mp > 0.06:
                sig[(r["slug"], l["market_slug"])].append(("BUY_NO", mp))

    print("=== GATE 5a: late-window (day21->close) tape notional per instance ===")
    print(f"{'event':44s} {'zone_$':>9s} {'matched_$':>10s} {'n_prints':>8s}")
    summary = []
    for ev_slug in TAPE_EVENTS:
        ev_legs = [l for l in legs if l["event_slug"] == ev_slug]
        if not ev_legs:
            continue
        y_m = ev_legs[0]["event_start"][:7]
        closed = datetime.strptime(ev_legs[0]["event_closedTime"][:19],
                                   "%Y-%m-%dT%H:%M:%S").replace(tzinfo=timezone.utc)
        # day21 of the target month: from the backtest records
        recs = [r for r in records if r["slug"] == ev_slug and r["ckpt"] == "day21"]
        if not recs:
            continue
        t21 = datetime.fromisoformat(recs[0]["t"])
        zone = matched = 0.0
        nprints = 0
        for l in ev_legs:
            path = os.path.join(pulldir, "trades", f"{l['condition_id']}.json")
            if not os.path.exists(path):
                continue
            trades = json.load(open(path))
            sigs = sig.get((ev_slug, l["market_slug"]), [])
            want = None
            if sigs:
                # majority direction across checkpoints, mean model_p
                dirs = [s[0] for s in sigs]
                want = max(set(dirs), key=dirs.count)
                mp = sum(s[1] for s in sigs if s[0] == want) / dirs.count(want)
            for tr in trades:
                ts = datetime.fromtimestamp(tr["timestamp"], tz=timezone.utc)
                if not (t21 <= ts <= closed):
                    continue
                p, sz = float(tr["price"]), float(tr["size"])
                yes_p = p if tr["outcome"] == "Yes" else 1 - p
                notional = p * sz
                if FUNDABLE[0] <= p <= FUNDABLE[1]:
                    zone += notional
                    nprints += 1
                if want is None:
                    continue
                # flow a passive quoter on our side could absorb:
                sell_yes_flow = (tr["outcome"] == "Yes" and tr["side"] == "SELL") or \
                                (tr["outcome"] == "No" and tr["side"] == "BUY")
                if want == "BUY_YES" and sell_yes_flow and yes_p <= mp - EDGE_MIN \
                        and FUNDABLE[0] <= yes_p <= FUNDABLE[1]:
                    matched += yes_p * sz
                if want == "BUY_NO" and not sell_yes_flow and (1 - yes_p) <= (1 - mp) - EDGE_MIN \
                        and FUNDABLE[0] <= (1 - yes_p) <= FUNDABLE[1]:
                    matched += (1 - yes_p) * sz
        print(f"{ev_slug:44s} {zone:9.0f} {matched:10.0f} {nprints:8d}")
        summary.append({"event": ev_slug, "zone_notional": round(zone),
                        "matched_notional": round(matched), "n_zone_prints": nprints})

    print("\n=== GATE 5b: today's live July-2026 books ===")
    ev_legs = [l for l in legs
               if l["event_slug"] == "july-2026-temperature-increase-c-20260608140824583"]
    books = []
    for l in ev_legs:
        b = json.load(open(os.path.join(pulldir, "books", f"{l['token_yes']}.json")))
        bids, asks = b.get("bids") or [], b.get("asks") or []
        bb = bids[-1] if bids else None
        ba = asks[-1] if asks else None
        def within(levels, ref, side):
            tot = 0.0
            for lv in levels:
                p, s = float(lv["price"]), float(lv["size"])
                if side == "bid" and p >= ref - 0.05:
                    tot += p * s
                if side == "ask" and p <= ref + 0.05:
                    tot += p * s
            return tot
        row = {
            "market_slug": l["market_slug"],
            "bid": bb and float(bb["price"]), "bid_top_usd": bb and round(float(bb["price"]) * float(bb["size"])),
            "ask": ba and float(ba["price"]), "ask_top_usd": ba and round(float(ba["price"]) * float(ba["size"])),
            "bid_5c_usd": bb and round(within(bids, float(bb["price"]), "bid")),
            "ask_5c_usd": ba and round(within(asks, float(ba["price"]), "ask")),
        }
        books.append(row)
        print(f"{l['question'][35:80]:47s} bid {row['bid']} (${row['bid_top_usd']}, "
              f"5c ${row['bid_5c_usd']}) ask {row['ask']} (${row['ask_top_usd']}, 5c ${row['ask_5c_usd']})")

    with open(os.path.join(btdir, "capacity.json"), "w") as f:
        json.dump({"late_window_tape": summary, "live_books": books}, f, indent=1)


if __name__ == "__main__":
    main()
