"""Live predictions for the open cohort.

As-built method after day-1 backtesting (see results/backtest-2026-07-25.md):

  1. Read the resolving arena slice named in each board's rules text (NOT the default view
     - the July #1/#2/#3/Chinese boards resolve on text/overall-no-style-control, whose
     ordering differs from the site's default style-control-on table).
  2. Confirm the board's current standing is coherent with that table (a sanity screen; the
     joint order-statistic simulation failed Gate 2 and is NOT used to price).
  3. Price by sharpening the de-vigged market: p^alpha renormalised, alpha=1.75 (low end of
     the leave-one-month-out range 1.75-2.5), clipped to [0.003, 0.995]. This is the only
     mechanism that beat the crowd out of sample.

Book gate: a board is `active` only if the favourite's spread <= 5c and top-of-book depth
within 10c >= $500 (wiki/reference/thin-market-price-read.md).
"""

import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from resolve import norm  # noqa: E402

ALPHA = 1.75
PCAP = (0.003, 0.995)


def sharpen(dv, a=ALPHA):
    p = {k: max(v, 1e-6) ** a for k, v in dv.items()}
    s = sum(p.values())
    return {k: v / s for k, v in p.items()}


def clip(p):
    return min(max(p, PCAP[0]), PCAP[1])


def main(books_path, month_prefix="2026-07"):
    bk = json.load(open(books_path))
    boards = defaultdict(list)
    for l in bk:
        if l["check_et"].startswith(month_prefix):
            boards[(l["board_type"], l["slug"])].append(l)

    out = []
    for (bt, slug), legs in sorted(boards.items()):
        live = [l for l in legs if l["book"] and l["book"]["mid"] is not None]
        if len(live) < 2:
            continue
        mids = {l["company"]: l["book"]["mid"] for l in live}
        s = sum(mids.values())
        dv = {k: v / s for k, v in mids.items()}
        sh = sharpen(dv)
        fav = max(dv, key=dv.get)
        fb = next(l["book"] for l in live if l["company"] == fav)
        book_ok = (fb["spread"] is not None and fb["spread"] <= 0.05
                   and (fb["depth_10c_usd"] or 0) >= 500)
        for l in live:
            c = l["company"]
            out.append(dict(
                board_type=bt, market_slug=l["slug"],
                condition_id=l["condition_id"], outcome=c, token_id=l["token_id"],
                probability=round(clip(sh[c]), 4),
                clob_midpoint=round(l["book"]["mid"], 4),
                devigged=round(dv[c], 4),
                best_bid=l["book"]["best_bid"], best_ask=l["book"]["best_ask"],
                spread_c=round(100 * (l["book"]["spread"] or 0), 1),
                depth10c_usd=round(l["book"]["depth_10c_usd"] or 0),
                leg_sum=round(s, 4), book_ok=book_ok, favourite=(c == fav),
            ))
    return out


if __name__ == "__main__":
    rows = main(sys.argv[1])
    json.dump(rows, open("strategies/arena-rank/satellites/data/predictions_live.json", "w"),
              indent=1)
    print(f"{len(rows)} legs across "
          f"{len(set(r['board_type'] for r in rows))} board types\n")
    hdr = ("market_slug", "condition_id", "outcome", "token_id", "probability",
           "clob_midpoint")
    print(",".join(hdr))
    for r in sorted(rows, key=lambda z: (z["board_type"], -z["probability"])):
        if r["probability"] < 0.01 and not r["favourite"]:
            continue
        print(f"{r['market_slug']},{r['condition_id']},{r['outcome']},{r['token_id']},"
              f"{r['probability']},{r['clob_midpoint']}")
