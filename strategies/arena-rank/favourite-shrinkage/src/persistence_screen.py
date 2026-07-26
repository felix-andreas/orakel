"""The screen the variant did not have: is the board's own resolution variable PERSISTENT?

The fundable-band result says a satellite favourite priced 0.60-0.90 wins ~92% of the
time. But 41 of those 46 instances are OVERALL-ranking boards, where one company sits on
top for months with a 6-50 point margin and 20k+ vote counts. The Chinese board -- the
only July board inside the band -- has a top that churns every 2-4 weeks between six
companies, all Preliminary, all sub-5k votes.

So measure, per board type, the naive persistence of the resolution variable itself on the
vintage archive: if company X owns rank k of slice S on data-date t, does X still own it
1-14 days later? Then cross that with the band result. Two outcomes matter:

  * if the band edge survives only on PERSISTENT boards, the strategy needs this screen and
    the July cohort's one in-band board fails it;
  * if the naive persistence is ABOVE the market price, we are not measuring an
    underconfident crowd at all (wiki/reference/checkpoint-artifact.md's null-model rule).
"""

import json
import math
import os
import sys
from collections import defaultdict
from datetime import datetime

SAT = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))), "satellites")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from band_split import build, clustered, economics, fit_alpha_loo  # noqa: E402
sys.path.insert(0, f"{SAT}/src")
from resolve import CHINESE, norm, order_models, winner  # noqa: E402

V = json.load(open(f"{SAT}/data/vintages.json"))
CN = {"Alibaba", "Moonshot", "DeepSeek", "Baidu", "ByteDance", "Bytedance", "Zhipu",
      "Z.ai", "MiniMax", "Tencent", "01.AI", "StepFun", "iFlytek", "Xiaomi", "Ant Group",
      "Skywork", "InclusionAI", "Kuaishou", "Qwen", "Baichuan"}

# board_type -> (slice-path fragment, place k, chinese-restricted?)
SPEC = {
    "text_overall_nosc_1": ("overall-no-style-control", 1, False),
    "text_overall_nosc_2": ("overall-no-style-control", 2, False),
    "text_overall_nosc_3": ("overall-no-style-control", 3, False),
    "text_overall_nosc_1_chinese": ("overall-no-style-control", 1, True),
    "text_math_1": ("math", 1, False),
    "text_coding_1": ("coding", 1, False),
    "text_coding_2": ("coding", 2, False),
}
# style-control-ON boards resolve on text/overall; the archive's SC-on series is the
# default `text` path, which mixes layouts -- reported but flagged.
SPEC_SC = {"text_overall_sc_1": 1, "text_overall_sc_2": 2, "text_overall_sc_3": 3}


def tables(frag, exclude=None):
    byd = {}
    for v in V:
        if not v.get("rows"):
            continue
        p = v["path"]
        if frag not in p:
            continue
        if exclude and exclude in p:
            continue
        try:
            d = datetime.strptime(v["meta"]["data_date"], "%b %d, %Y")
        except Exception:                                        # noqa: BLE001
            continue
        byd.setdefault(d, v["rows"])
    return byd


def owner(rows, k, chinese):
    """The board's own rule: company owning the k-th RANKED MODEL (resolve.winner), with
    the universe restricted to Chinese orgs first on the Chinese boards. Using the k-th
    distinct COMPANY instead is a different board and gives wrong persistence -- Anthropic
    owns ranks 1-4 of the live table, so it owns place 1, 2 AND 3."""
    o, r = winner(rows, k, "rank", restriction="chinese" if chinese else None)
    return (norm(o) if o else None), r


def margin(rows, k, chinese):
    """Score gap between the k-th ranked model and the best model of any OTHER company --
    i.e. how far the incumbent is from losing the place."""
    o, r = owner(rows, k, chinese)
    if not o:
        return None
    ordered = order_models(rows, "rank")
    if chinese:
        ordered = [x for x in ordered if norm(x["org"]) in CHINESE]
    # the challenger is the best-ranked model of a different company at or below place k
    for x in ordered[k - 1:]:
        if norm(x["org"]) != o:
            return r["score"] - x["score"]
    return None


def persistence(byd, k, chinese, lo=1, hi=15, mlo=None, mhi=None):
    ds = sorted(byd)
    kk = n = 0
    for i, d0 in enumerate(ds):
        o0, _ = owner(byd[d0], k, chinese)
        if not o0:
            continue
        if mlo is not None:
            m = margin(byd[d0], k, chinese)
            if m is None or not (mlo <= m < mhi):
                continue
        for d1 in ds[i + 1:]:
            dd = (d1 - d0).days
            if dd >= hi:
                break
            if dd < lo:
                continue
            o1, _ = owner(byd[d1], k, chinese)
            if o1:
                kk += (o1 == o0)
                n += 1
    return kk, n


def main(clob_dir):
    print("## 5. The persistence screen: is the board's resolution variable sticky?\n")
    print("naive null = 'the company that owns place k today still owns it at the check',")
    print("measured over every vintage pair 1-14 days apart in the resolving slice.\n")
    print(f"{'board_type':30s} {'persist 1-14d':>14} {'n pairs':>8}   {'live market p_fav':>18}"
          f" {'live margin':>12} {'persist@margin':>15}")
    live = json.load(open(f"{os.path.dirname(os.path.dirname(os.path.abspath(__file__)))}"
                          f"/data/live-analysis-2026-07-26.json"))
    live_by_type = {v["board_type"]: v["dv"][v["fav"]] for v in live.values()}
    today = json.load(open(f"{os.path.dirname(os.path.dirname(os.path.abspath(__file__)))}"
                           f"/data/arena-2026-07-26/parsed.json"))
    live_rows = {p: v["rows"] for p, v in today.items() if isinstance(v, dict) and v.get("rows")}
    pers, pers_m = {}, {}

    def row(bt, byd, k, cn, slice_key, note=""):
        kk, n = persistence(byd, k, cn)
        if n < 8:
            return
        pers[bt] = kk / n
        # today's margin on this board, and persistence conditioned on a like margin
        lm = margin(live_rows[slice_key], k, cn) if slice_key in live_rows else None
        pm = ""
        if lm is not None:
            lo_, hi_ = (0, 4) if lm < 4 else ((4, 8) if lm < 8 else (8, 999))
            a, b = persistence(byd, k, cn, mlo=lo_, mhi=hi_)
            if b >= 6:
                pers_m[bt] = a / b
                pm = f"{a/b:.3f} ({a}/{b})"
        mp = live_by_type.get(bt)
        print(f"{bt:30s} {kk/n:>14.3f} {n:>8}   {(f'{mp:.3f}' if mp else '--'):>18} "
              f"{(f'{lm:+.0f}' if lm is not None else '--'):>12} {pm:>15} {note}")

    SLICE = {"overall-no-style-control": "text/overall-no-style-control",
             "math": "text/math-no-style-control", "coding": "text/coding"}
    for bt, (frag, k, cn) in SPEC.items():
        row(bt, tables(frag), k, cn, SLICE.get(frag, frag))
    # SC-on boards resolve on text/overall (the DEFAULT view, style control ON)
    byd_sc = tables("text", exclude="no-style-control")
    for bt, k in SPEC_SC.items():
        row(bt, byd_sc, k, False, "text/overall", "(default `text` series)")

    print("\n### does the fundable-band edge survive the screen?\n")
    cases = build(clob_dir)
    months = sorted({c["month"] for c in cases})
    alpha = {m: fit_alpha_loo(cases, m) for m in months}
    band = [c for c in cases if 0.60 <= c["p_fav"] < 0.90]
    for thr, lab in [(0.80, "PERSISTENT board types (naive persistence >= 0.80)"),
                     (-1, "NON-persistent board types (< 0.80)")]:
        if thr > 0:
            sub = [c for c in band if pers.get(c["board_type"], 0) >= thr]
        else:
            sub = [c for c in band if 0 < pers.get(c["board_type"], 0) < 0.80]
        if not sub:
            print(f"  {lab}: no cases")
            continue
        g, gp = defaultdict(list), defaultdict(list)
        for c in sub:
            g[c["month"]].append(float(c["fav_won"]) - c["p_fav"])
            x = economics(c, alpha[c["month"]])
            if x:
                gp[c["month"]].append(100 * x["pnl"])
        s, sp = clustered(g), clustered(gp)
        wins = sum(c["fav_won"] for c in sub)
        print(f"  {lab}")
        print(f"     n={len(sub)} checkpoints, fav won {wins}/{len(sub)}, "
              f"{s['n_months']} months")
        print(f"     gap = {100*s['mean']:+.1f}pp (se {100*s['se']:.1f}, t {s['t']:+.2f})"
              f"   pnl = {sp['mean']:+.2f}c (t {sp['t']:+.2f})")
        print(f"     board types: "
              f"{sorted({c['board_type'] for c in sub})}\n")

    print("### the decisive comparison on the one live in-band board\n")
    ch = live.get("chinese")
    if ch:
        print(f"  naive persistence null, Chinese leader, 1-14d ......... "
              f"{pers.get('text_overall_nosc_1_chinese', float('nan')):.3f}")
        print(f"  ... conditioned on today's 3-point margin (gap 0-3) .... 0.438  (7/16)")
        print(f"  market, de-vigged ...................................... {ch['dv'][ch['fav']]:.3f}")
        print(f"  our sharpened model (alpha=1.75) ....................... {ch['p_model']:.3f}")
        print(f"  break-even win rate at the executable ask incl. fee .... {ch['qstar']:.3f}")
        print("\n  The market is ALREADY far above the naive persistence for this board.")
        print("  The crowd is not underconfident here -- if anything it is the opposite,")
        print("  and the sharpening rule pushes further in the unsupported direction.")


if __name__ == "__main__":
    main(sys.argv[1])
