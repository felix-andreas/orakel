"""Anchor calibration: fit the simulation's free parameters to the DEEP #1-overall board.

This is the idea's central mechanism. The $4-36M #1-overall board is the sharpest read of
the latent ranking available; the satellites are 10-250x thinner. So the simulation's
nuisance parameters (drift scale, entrant rate, incumbency) are fitted so the model
reproduces the ANCHOR board's de-vigged distribution, and are then applied unchanged to
the satellites — which are never used in the fit.

Leave-one-month-out: the parameters used to price month m are fitted on all other months,
so the satellite numbers are genuinely out of sample.
"""

import json
import math
import os
import sys
from collections import defaultdict
from datetime import timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gate2_market import board_snapshot  # noqa: E402
from resolve import norm, pin_vintage  # noqa: E402
from simulate import load_spec, simulate, to_legs  # noqa: E402
from vintage import et_to_utc  # noqa: E402

ANCHOR = "text_overall_nosc_1"
PATHS = {"text/overall-no-style-control"}
CHECKPOINTS = [30, 14, 7]
GRID_DRIFT = [0.15, 0.3, 0.5, 1.0, 1.5]
GRID_RATE = [0.05, 0.15, 0.3, 0.5, 0.8]
GRID_INC = [0.0, 0.25, 0.5, 0.85]


def anchor_cases(root, clob_dir):
    boards = json.load(open(f"{root}/poly/boards.json"))
    vint = json.load(open(f"{root}/vintages.json"))
    out = []
    for b in boards:
        if b["board_type"] != ANCHOR or not (b["closed"] and b["check_et"]):
            continue
        if not isinstance(b["winner"], str):
            continue
        T = et_to_utc(b["check_et"])
        for d in CHECKPOINTS:
            t = T - timedelta(days=d)
            cap, q = pin_vintage(vint, PATHS, t)
            if cap is None or cap["diag"]["n"] < 50 or cap["ts"] > t.strftime("%Y%m%d%H%M%S"):
                continue
            raw, dv = board_snapshot(b, clob_dir, t)
            if not dv:
                continue
            out.append(dict(month=b["check_et"][:7], slug=b["slug"], d=d,
                            rows=cap["rows"], dv=dv, winner=b["winner"]))
    return out, vint


def anchor_loss(cases, spec, ds, rs, inc, n_sims=4000):
    """Cross-entropy between the model's #1 distribution and the deep board's de-vigged one."""
    tot, n = 0.0, 0
    for c in cases:
        sim = simulate(c["rows"], c["d"], spec, n_sims=n_sims, drift_scale=ds,
                       rate_scale=rs, incumbency=inc, seed=hash((c["slug"], c["d"])) % 10**6)
        if not sim:
            continue
        mp = to_legs(sim[1], list(c["dv"]))
        s = sum(mp.values()) or 1
        mp = {k: v / s for k, v in mp.items()}
        for leg, q in c["dv"].items():
            p = min(max(mp.get(leg, 0.0), 1e-3), 1 - 1e-3)
            tot += -q * math.log(p)
        n += 1
    return tot / max(1, n)


def fit(cases, spec, exclude_month=None, n_sims=4000, verbose=False):
    sub = [c for c in cases if c["month"] != exclude_month]
    best, arg = float("inf"), None
    for ds in GRID_DRIFT:
        for rs in GRID_RATE:
            for inc in GRID_INC:
                L = anchor_loss(sub, spec, ds, rs, inc, n_sims)
                if verbose:
                    print(f"    ds={ds} rs={rs} inc={inc} anchor_xent={L:.4f}")
                if L < best:
                    best, arg = L, (ds, rs, inc)
    return arg, best


if __name__ == "__main__":
    root = "strategies/arena-rank/satellites/data"
    cases, _ = anchor_cases(root, sys.argv[1])
    spec = load_spec(sys.argv[2])
    print(f"{len(cases)} anchor board-checkpoints across "
          f"{len(set(c['month'] for c in cases))} months")
    arg, L = fit(cases, spec, verbose=True)
    print(f"\nGLOBAL best: drift_scale={arg[0]} rate_scale={arg[1]} incumbency={arg[2]} "
          f"anchor cross-entropy={L:.4f}")
    loo = {}
    for m in sorted(set(c["month"] for c in cases)):
        a, l = fit(cases, spec, exclude_month=m, n_sims=3000)
        loo[m] = a
        print(f"  LOO {m}: {a}  (xent {l:.4f})")
    json.dump(dict(global_fit=arg, loo=loo), open(f"{root}/calibration.json", "w"), indent=1)
