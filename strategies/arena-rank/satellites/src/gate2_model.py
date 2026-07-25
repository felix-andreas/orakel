"""Gate 2b/3 — model vs market, paired, on resolved satellite instances.

Three forecasters are compared on the same board-checkpoints:

  market      de-vigged CLOB mids
  model       the joint order-statistic simulation (simulate.py), anchor-calibrated
  naive       "the company owning the k-th model right now wins", shrunk to the base rate

Paired log-loss, clustered by cohort-month (within a month the boards resolve to the same
company, so board-level n is fake). Gate 2 kills the idea if the market beats the model at
every checkpoint.
"""

import json
import math
import os
import sys
from collections import defaultdict
from datetime import timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gate2_market import board_snapshot  # noqa: E402
from resolve import CHINESE, norm, order_models, pin_vintage  # noqa: E402
from simulate import load_spec, simulate, to_legs  # noqa: E402
from vintage import et_to_utc  # noqa: E402

SLICE_PATHS = {
    "text_overall_nosc": {"text/overall-no-style-control"},
    "text_overall_sc": {"text/overall", "text"},
    "text_math": {"text/math-no-style-control", "text/math"},
    "text_coding": {"text/coding-no-style-control", "text/coding"},
}
CHECKPOINTS = [30, 14, 7]
EPS = 5e-3


def slice_key(b):
    s = b["slice"]
    if s == "text_overall":
        return "text_overall_sc" if b["style_control"] else "text_overall_nosc"
    return s


def clip(p):
    return min(max(p, EPS), 1 - EPS)


def main(root, clob_dir, spec_path, drift_scale=1.0, rate_scale=1.0, n_sims=8000,
         use_loo=True):
    boards = json.load(open(f"{root}/poly/boards.json"))
    vint = json.load(open(f"{root}/vintages.json"))
    spec = load_spec(spec_path)
    loo = {}
    if use_loo and os.path.exists(f"{root}/calibration.json"):
        cal = json.load(open(f"{root}/calibration.json"))
        loo = cal.get("loo", {})
        gdef = cal.get("global_fit")
    else:
        gdef = None
    rows = []
    for b in boards:
        if not (b["closed"] and b["board_type"] and b["check_et"]):
            continue
        if not isinstance(b["winner"], str) or b["place"] not in (1, 2, 3):
            continue
        paths = SLICE_PATHS.get(slice_key(b))
        if not paths:
            continue
        T = et_to_utc(b["check_et"])
        legs = [l["company"] for l in b["legs"]]
        for d in CHECKPOINTS:
            t = T - timedelta(days=d)
            cap, qual = pin_vintage(vint, paths, t)
            if cap is None or cap["diag"]["n"] < 50:
                continue
            # only use a vintage that actually predates the checkpoint
            if cap["ts"] > t.strftime("%Y%m%d%H%M%S"):
                continue
            raw, dv = board_snapshot(b, clob_dir, t)
            if not dv:
                continue
            win = next((k for k in dv if norm(k) == norm(b["winner"])), None)
            if win is None:
                continue

            # leave-one-month-out anchor calibration: the params pricing month m were
            # fitted on the OTHER months' deep #1 boards only, never on satellites
            ds, rs_, inc = loo.get(b["check_et"][:7], gdef or
                                   (drift_scale, rate_scale, 0.0))
            sim = simulate(cap["rows"], d, spec, n_sims=n_sims,
                           drift_scale=ds, rate_scale=rs_, incumbency=inc,
                           seed=hash((b["slug"], d)) % 10**6)
            if not sim:
                continue
            key = "chinese1" if b["restriction"] == "chinese" else b["place"]
            mp = to_legs(sim[key], list(dv))
            s = sum(mp.values()) or 1
            mp = {k: v / s for k, v in mp.items()}

            # naive: current k-th place company gets 1-eps
            ordered = order_models(cap["rows"], b["res_var"])
            if b["restriction"] == "chinese":
                ordered = [r for r in ordered if norm(r["org"]) in CHINESE]
            nv = {k: 0.0 for k in dv}
            if len(ordered) >= b["place"]:
                cur = ordered[b["place"] - 1]["org"]
                hit = next((k for k in dv if norm(k) == norm(cur)), None)
                for k in nv:
                    nv[k] = 0.80 if k == hit else 0.20 / max(1, len(nv) - 1)
                if hit is None:
                    nv = {k: 1.0 / len(nv) for k in nv}

            rows.append(
                dict(
                    slug=b["slug"], board_type=b["board_type"], check=b["check_et"],
                    month=b["check_et"][:7], d=d, quality=qual,
                    vintage=cap["meta"].get("data_date"), n_legs=len(dv),
                    p_mkt=clip(dv[win]), p_model=clip(mp.get(win, 0.0)),
                    p_naive=clip(nv.get(win, 0.0)),
                    p_fav_mkt=max(dv.values()),
                    fav_mkt=max(dv, key=dv.get),
                    fav_model=max(mp, key=mp.get) if mp else None,
                    winner=b["winner"], volume=b["volume"],
                )
            )
    json.dump(rows, open(f"{root}/gate2_model.json", "w"), indent=1)
    return rows


def paired(rows, a, b_, label):
    """Monthly-clustered paired difference in log-loss (positive = `a` better)."""
    g = defaultdict(list)
    for r in rows:
        g[r["month"]].append(-math.log(r[b_]) + math.log(r[a]))
    months = sorted(g)
    diffs = [sum(v) / len(v) for m, v in sorted(g.items())]
    n = len(diffs)
    mu = sum(diffs) / n
    sd = math.sqrt(sum((x - mu) ** 2 for x in diffs) / (n - 1)) if n > 1 else float("nan")
    se = sd / math.sqrt(n) if n > 1 else float("nan")
    print(f"  {label:34s} n_months={n:2d} mean dLL={mu:+.4f} se={se:.4f} "
          f"t={mu/se if se else float('nan'):+.2f}  months better={sum(1 for x in diffs if x>0)}/{n}")
    return mu, se


if __name__ == "__main__":
    clob = sys.argv[1]
    spec = sys.argv[2]
    ds = float(sys.argv[3]) if len(sys.argv) > 3 else 1.0
    rs = float(sys.argv[4]) if len(sys.argv) > 4 else 1.0
    rows = main("strategies/arena-rank/satellites/data", clob, spec, ds, rs)
    print(f"{len(rows)} scored board-checkpoints (drift_scale={ds}, rate_scale={rs})\n")
    for scope, sel in [("ALL", lambda r: True),
                       ("SATELLITES", lambda r: r["board_type"] != "text_overall_nosc_1")]:
        rs_ = [r for r in rows if sel(r)]
        if not rs_:
            continue
        print(f"=== {scope} (n={len(rs_)}) ===")
        for nm, key in [("market", "p_mkt"), ("model", "p_model"), ("naive", "p_naive")]:
            ll = sum(-math.log(r[key]) for r in rs_) / len(rs_)
            br = sum((1 - r[key]) ** 2 for r in rs_) / len(rs_)
            print(f"  {nm:8s} logloss={ll:.4f}  brier(winner-leg)={br:.4f}")
        paired(rs_, "p_model", "p_mkt", "model vs market")
        paired(rs_, "p_naive", "p_mkt", "naive  vs market")
        print()
        for d in CHECKPOINTS:
            sub = [r for r in rs_ if r["d"] == d]
            if len(sub) > 3:
                print(f"  -- T-{d}d (n={len(sub)})")
                paired(sub, "p_model", "p_mkt", "    model vs market")
