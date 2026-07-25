"""The only surviving candidate: sharpen the crowd's own distribution.

Gate 2 killed the joint simulation — the market beats it at every checkpoint. But the
crowd's *modal calibration* is off in a consistent direction: on satellite boards the
favourite wins ~9pp more often than its de-vigged price implies (t=4.77 clustered by
cohort-month). That is the classic favourite-longshot bias
(wiki/reference/favorite-longshot-bias.md), and it is a transformation of the market's own
distribution, not an independent model.

    p_sharp(i) = p_mkt(i)^alpha / sum_j p_mkt(j)^alpha,   alpha > 1

alpha is fitted leave-one-month-out, so every scored month is out of sample. Then the
t+24h delayed-execution test with adverse selection is applied to the resulting trades.
"""

import json
import math
import os
import sys
from collections import defaultdict
from datetime import timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gate2_market import board_snapshot  # noqa: E402
from resolve import norm  # noqa: E402
from vintage import et_to_utc  # noqa: E402

CHECKPOINTS = [30, 14, 7]
ALPHAS = [1.0, 1.1, 1.2, 1.35, 1.5, 1.75, 2.0, 2.5]
EPS = 5e-3


def sharpen(dv, a):
    p = {k: max(v, 1e-6) ** a for k, v in dv.items()}
    s = sum(p.values())
    return {k: v / s for k, v in p.items()}


def build(root, clob_dir, exclude_anchor=True):
    boards = json.load(open(f"{root}/poly/boards.json"))
    out = []
    for b in boards:
        if not (b["closed"] and b["board_type"] and b["check_et"]):
            continue
        if not isinstance(b["winner"], str):
            continue
        if exclude_anchor and b["board_type"] == "text_overall_nosc_1":
            continue
        T = et_to_utc(b["check_et"])
        for d in CHECKPOINTS:
            t = T - timedelta(days=d)
            raw, dv = board_snapshot(b, clob_dir, t)
            if not dv:
                continue
            win = next((k for k in dv if norm(k) == norm(b["winner"])), None)
            if win is None:
                continue
            # price 24h later, for the delayed-execution test (inputs frozen at t)
            raw2, dv2 = board_snapshot(b, clob_dir, t + timedelta(hours=24))
            out.append(dict(slug=b["slug"], board_type=b["board_type"],
                            month=b["check_et"][:7], d=d, dv=dv, dv2=dv2,
                            raw=raw, raw2=raw2, win=win, volume=b["volume"]))
    return out


def ll(cases, a):
    return sum(-math.log(min(max(sharpen(c["dv"], a)[c["win"]], EPS), 1 - EPS))
               for c in cases) / len(cases)


def fit_alpha(cases, exclude_month=None):
    sub = [c for c in cases if c["month"] != exclude_month]
    return min(ALPHAS, key=lambda a: ll(sub, a))


def report(cases):
    months = sorted({c["month"] for c in cases})
    print(f"{len(cases)} satellite board-checkpoints over {len(months)} cohort-months\n")

    print("--- in-sample alpha sweep (log-loss) ---")
    for a in ALPHAS:
        print(f"   alpha={a:4.2f}  logloss={ll(cases, a):.4f}")

    print("\n--- leave-one-month-out ---")
    rows = []
    for m in months:
        a = fit_alpha(cases, exclude_month=m)
        sub = [c for c in cases if c["month"] == m]
        base = sum(-math.log(min(max(c["dv"][c["win"]], EPS), 1 - EPS)) for c in sub) / len(sub)
        shrp = ll(sub, a)
        rows.append(dict(month=m, alpha=a, n=len(sub), mkt=base, flb=shrp, d=base - shrp))
        print(f"   {m}  alpha*={a:4.2f} n={len(sub):2d}  market LL={base:.4f}"
              f"  sharpened LL={shrp:.4f}  gain={base-shrp:+.4f}")
    ds = [r["d"] for r in rows]
    n = len(ds)
    mu = sum(ds) / n
    sd = math.sqrt(sum((x - mu) ** 2 for x in ds) / (n - 1))
    se = sd / math.sqrt(n)
    print(f"\n   OOS mean log-loss gain = {mu:+.4f}  se={se:.4f}  t={mu/se:+.2f}"
          f"  months better = {sum(1 for x in ds if x > 0)}/{n}")

    print("\n--- by checkpoint (LOO alpha) ---")
    for d in CHECKPOINTS:
        rr = []
        for m in months:
            a = fit_alpha(cases, exclude_month=m)
            sub = [c for c in cases if c["month"] == m and c["d"] == d]
            if not sub:
                continue
            base = sum(-math.log(min(max(c["dv"][c["win"]], EPS), 1 - EPS)) for c in sub) / len(sub)
            rr.append(base - ll(sub, a))
        if len(rr) > 2:
            mu = sum(rr) / len(rr)
            sd = math.sqrt(sum((x - mu) ** 2 for x in rr) / (len(rr) - 1))
            print(f"   T-{d:2d}d  n_months={len(rr):2d} mean gain={mu:+.4f}"
                  f" se={sd/math.sqrt(len(rr)):.4f} t={mu/(sd/math.sqrt(len(rr))):+.2f}"
                  f"  better={sum(1 for x in rr if x>0)}/{len(rr)}")
    return rows


def trade_test(cases, edge_c=0.03, adverse_c=0.02, zone=(0.03, 0.50), delayed=True):
    """t+24h delayed execution: signal from prices at t, fill at t+24h mid + adverse.

    Reports PnL per trade in cents for legs the rule wants to BUY, split by half-sample.
    `zone` restricts to the fundable price band; the FLB trade also has a favourite side
    outside that band, reported separately.
    """
    out = []
    for c in cases:
        a = 1.35
        sh = sharpen(c["dv"], a)
        # fills use RAW mids, not de-vigged: de-vigging shaves the overround off the
        # cost basis and flatters every buy. The signal uses de-vigged prices, the fill
        # pays what the book actually quotes.
        fills = (c["raw2"] if (delayed and c.get("raw2")) else c["raw"])
        for leg, p_model in sh.items():
            p_now = c["dv"][leg]
            p_fill_mid = fills.get(leg)
            if p_fill_mid is None:
                continue
            if p_model - p_now < edge_c:
                continue
            cost = p_fill_mid + adverse_c
            if not (0.01 < cost < 0.99):
                continue
            pay = 1.0 if leg == c["win"] else 0.0
            out.append(dict(month=c["month"], slug=c["slug"], d=c["d"], leg=leg,
                            in_zone=(zone[0] <= cost <= zone[1]),
                            cost=cost, pnl=100 * (pay - cost)))
    return out


if __name__ == "__main__":
    root = "strategies/arena-rank/satellites/data"
    cases = build(root, sys.argv[1])
    rows = report(cases)
    json.dump(dict(loo=rows), open(f"{root}/gate_flb.json", "w"), indent=1)

    print("\n=== Gate 4: t+24h delayed execution, +2c adverse (alpha=1.35) ===")
    for label, delayed in [("instant fill at t", False), ("t+24h delayed fill", True)]:
        tr = trade_test(cases, delayed=delayed)
        if not tr:
            print(f"  {label}: no trades")
            continue
        for scope, sel in [("all buys", lambda x: True),
                           ("fundable 3-50c", lambda x: x["in_zone"])]:
            t = [x for x in tr if sel(x)]
            if not t:
                print(f"  {label:20s} {scope:16s}: no trades")
                continue
            g = defaultdict(list)
            for x in t:
                g[x["month"]].append(x["pnl"])
            per = [sum(v) / len(v) for v in g.values()]
            n = len(per)
            mu = sum(per) / n
            sd = math.sqrt(sum((y - mu) ** 2 for y in per) / (n - 1)) if n > 1 else float("nan")
            se = sd / math.sqrt(n) if n > 1 else float("nan")
            ms = sorted(g)
            h1 = [g[m] for m in ms[: len(ms) // 2]]
            h2 = [g[m] for m in ms[len(ms) // 2:]]
            f1 = sum(sum(x) for x in h1) / max(1, sum(len(x) for x in h1))
            f2 = sum(sum(x) for x in h2) / max(1, sum(len(x) for x in h2))
            print(f"  {label:20s} {scope:16s}: n={len(t):4d} trades over {n} months"
                  f"  mean={mu:+.2f}c se={se:.2f} t={mu/se if se else float('nan'):+.2f}"
                  f"  | 1st half {f1:+.2f}c  2nd half {f2:+.2f}c")
