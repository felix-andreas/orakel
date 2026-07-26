"""The strong null: does the LEADERBOARD MARGIN at the checkpoint already explain the
favourite's win rate -- and does the market beat that null?

wiki/reference/checkpoint-artifact.md rule 2: always run a null through the same pipeline;
it should lose. Uniform and flat-0.90 nulls lose easily (band_split.py S0). The null that
actually matters here is the one a human would use without any market data:

    "whoever owns the place today owns it at the check, with a probability read off the
     historical persistence at that leaderboard margin"

For each resolved satellite instance x checkpoint, pin the vintage that was live at T-d,
compute the margin between the place-holder and the best model of any other company, and
compare (a) the realised outcome, (b) the market's de-vigged favourite price, (c) our
sharpened price -- bucketed by margin.

This is what decides the live Chinese board: its margin today is +3 points, and the
question is whether a +3-margin favourite priced 0.80 has historically been UNDER-priced
(sharpen it) or OVER-priced (leave it alone).
"""

import json
import math
import os
import sys
from collections import defaultdict
from datetime import timedelta

SAT = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))), "satellites")
sys.path.insert(0, f"{SAT}/src")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gate0 import SLICE_PATHS, is_company_board, slice_key  # noqa: E402
from gate2_market import board_snapshot  # noqa: E402
from gate_flb import sharpen  # noqa: E402
from resolve import CHINESE, norm, order_models, pin_vintage, winner  # noqa: E402
from vintage import et_to_utc  # noqa: E402

CHECKPOINTS = [30, 14, 7]
ALPHA = 1.75
EPS = 5e-3
MBANDS = [(-99, 4, "margin 0-3"), (4, 8, "margin 4-7"), (8, 15, "margin 8-14"),
          (15, 999, "margin 15+")]


def board_margin(rows, place, res_var, restriction):
    o, r = winner(rows, place, res_var, restriction)
    if not o:
        return None, None
    ordered = order_models(rows, res_var)
    if restriction == "chinese":
        ordered = [x for x in ordered if norm(x["org"]) in CHINESE]
    for x in ordered[place - 1:]:
        if norm(x["org"]) != norm(o):
            return o, r["score"] - x["score"]
    return o, None


def main(clob_dir):
    boards = json.load(open(f"{SAT}/data/poly/boards.json"))
    vint = json.load(open(f"{SAT}/data/vintages.json"))
    rows = []
    for b in boards:
        if not (b["closed"] and b["board_type"] and b["check_et"] and is_company_board(b)):
            continue
        if not isinstance(b["winner"], str):
            continue
        if b["board_type"] == "text_overall_nosc_1":
            continue
        paths = SLICE_PATHS.get(slice_key(b))
        if not paths:
            continue
        T = et_to_utc(b["check_et"])
        for d in CHECKPOINTS:
            t = T - timedelta(days=d)
            cap, qual = pin_vintage(vint, paths, t)
            if cap is None:
                continue
            lead, marg = board_margin(cap["rows"], b["place"], b["res_var"], b["restriction"])
            if lead is None or marg is None:
                continue
            raw, dv = board_snapshot(b, clob_dir, t)
            if not dv:
                continue
            win = next((k for k in dv if norm(k) == norm(b["winner"])), None)
            if win is None:
                continue
            fav = max(dv, key=dv.get)
            rows.append(dict(slug=b["slug"], board_type=b["board_type"],
                             month=b["check_et"][:7], d=d, quality=qual,
                             margin=marg, leader=norm(lead), fav=fav, p_fav=dv[fav],
                             fav_won=(fav == win), lead_is_fav=(norm(lead) == norm(fav)),
                             lead_won=(norm(lead) == norm(b["winner"])),
                             p_sharp=sharpen(dv, ALPHA)[fav], legsum=sum(raw.values())))

    print(f"## 6. The leaderboard-margin null\n")
    print(f"{len(rows)} satellite board-checkpoints with a pinned vintage AND a live book, "
          f"{len({r['month'] for r in rows})} months\n")
    print(f"{'margin band':14s} {'n':>4} {'lead=fav':>9} {'lead won':>9} {'mkt p_fav':>10} "
          f"{'fav won':>8} {'mkt gap':>8} {'our p':>7} {'our gap':>8}")
    for lo, hi, lab in MBANDS:
        sub = [r for r in rows if lo <= r["margin"] < hi]
        if not sub:
            continue
        n = len(sub)
        print(f"{lab:14s} {n:>4} {sum(r['lead_is_fav'] for r in sub)/n:>9.3f} "
              f"{sum(r['lead_won'] for r in sub)/n:>9.3f} "
              f"{sum(r['p_fav'] for r in sub)/n:>10.3f} "
              f"{sum(r['fav_won'] for r in sub)/n:>8.3f} "
              f"{100*(sum(r['fav_won'] for r in sub)-sum(r['p_fav'] for r in sub))/n:>+7.1f}pp "
              f"{sum(r['p_sharp'] for r in sub)/n:>7.3f} "
              f"{100*(sum(r['fav_won'] for r in sub)-sum(r['p_sharp'] for r in sub))/n:>+7.1f}pp")

    print("\n### the same, restricted to the FUNDABLE band (favourite 0.60-0.90)\n")
    print(f"{'margin band':14s} {'n':>4} {'mnth':>5} {'mkt p_fav':>10} {'fav won':>8} "
          f"{'mkt gap':>9} {'our p':>7} {'our gap':>9}")
    for lo, hi, lab in MBANDS:
        sub = [r for r in rows if lo <= r["margin"] < hi and 0.60 <= r["p_fav"] < 0.90]
        if not sub:
            print(f"{lab:14s} {0:>4}   --")
            continue
        n = len(sub)
        mg = sum(r["fav_won"] for r in sub) / n - sum(r["p_fav"] for r in sub) / n
        og = sum(r["fav_won"] for r in sub) / n - sum(r["p_sharp"] for r in sub) / n
        print(f"{lab:14s} {n:>4} {len({r['month'] for r in sub}):>5} "
              f"{sum(r['p_fav'] for r in sub)/n:>10.3f} "
              f"{sum(r['fav_won'] for r in sub)/n:>8.3f} {100*mg:>+8.1f}pp "
              f"{sum(r['p_sharp'] for r in sub)/n:>7.3f} {100*og:>+8.1f}pp")

    print("\n### log-loss: market vs sharpened vs the margin null, by margin band\n")
    # margin null: P(leader keeps the place) estimated leave-one-month-out from the same
    # margin band, then put on the leader's leg and the rest spread by the market's shape.
    print(f"{'margin band':14s} {'n':>4} {'market LL':>10} {'sharp LL':>9} {'null LL':>8} "
          f"  verdict")
    for lo, hi, lab in MBANDS:
        sub = [r for r in rows if lo <= r["margin"] < hi]
        if len(sub) < 5:
            continue
        mkt = shp = nul = 0.0
        for r in sub:
            months = [x for x in rows if lo <= x["margin"] < hi and x["month"] != r["month"]]
            q = (sum(x["lead_won"] for x in months) / len(months)) if months else 0.5
            q = min(max(q, EPS), 1 - EPS)
            # the null puts q on the leader; the favourite is the leader in most cases
            p_null = q if r["lead_is_fav"] else (1 - q) / max(1, 3)
            mkt += -math.log(min(max(r["p_fav"] if r["fav_won"] else 1 - r["p_fav"], EPS), 1 - EPS))
            shp += -math.log(min(max(r["p_sharp"] if r["fav_won"] else 1 - r["p_sharp"], EPS), 1 - EPS))
            nul += -math.log(min(max(p_null if r["fav_won"] else 1 - p_null, EPS), 1 - EPS))
        n = len(sub)
        v = ("NULL BEATS MARKET -- audit the checkpoint" if nul < mkt
             else ("sharpening helps" if shp < mkt else "SHARPENING HURTS"))
        print(f"{lab:14s} {n:>4} {mkt/n:>10.4f} {shp/n:>9.4f} {nul/n:>8.4f}   {v}")

    print("\n### today's live boards placed on this table\n")
    live = json.load(open(f"{os.path.dirname(os.path.dirname(os.path.abspath(__file__)))}"
                          f"/data/live-analysis-2026-07-26.json"))
    today = json.load(open(f"{os.path.dirname(os.path.dirname(os.path.abspath(__file__)))}"
                           f"/data/arena-2026-07-26/parsed.json"))
    SL = {"text_overall_nosc_1_chinese": ("text/overall-no-style-control", 1, "chinese"),
          "text_overall_nosc_2": ("text/overall-no-style-control", 2, None),
          "text_overall_nosc_3": ("text/overall-no-style-control", 3, None),
          "text_math_1": ("text/math-no-style-control", 1, None),
          "text_overall_sc_1": ("text/overall", 1, None),
          "text_overall_sc_2": ("text/overall", 2, None),
          "text_overall_sc_3": ("text/overall", 3, None)}
    print(f"{'board':16s} {'leader':11s} {'margin':>7} {'mkt p_fav':>10} {'our p':>7} "
          f"{'hist fav-won @margin':>21}")
    for key, v in live.items():
        sp = SL.get(v["board_type"])
        if not sp or sp[0] not in today:
            continue
        lead, marg = board_margin(today[sp[0]]["rows"], sp[1], "rank", sp[2])
        band = next((b for b in MBANDS if b[0] <= (marg or 0) < b[1]), None)
        hist = [r for r in rows if band[0] <= r["margin"] < band[1]] if band else []
        hw = (f"{sum(r['fav_won'] for r in hist)}/{len(hist)} = "
              f"{sum(r['fav_won'] for r in hist)/len(hist):.3f}") if hist else "--"
        print(f"{key:16s} {lead or '--':11s} {(f'{marg:+.0f}' if marg is not None else '--'):>7} "
              f"{v['dv'][v['fav']]:>10.3f} {v['p_model']:>7.3f} {hw:>21}")

    json.dump(rows, open(f"{os.path.dirname(os.path.dirname(os.path.abspath(__file__)))}"
                         f"/data/margin_null.json", "w"), indent=1)


if __name__ == "__main__":
    main(sys.argv[1])
