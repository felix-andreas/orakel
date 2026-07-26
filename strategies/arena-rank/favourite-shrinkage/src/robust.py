"""Robustness of the fundable-band result, and the breakeven arithmetic that decides it.

The band split says the favourite-longshot gain is LARGER in the fundable 0.60-0.90 band
than at 0.93-1.00. Before acting on that, four things have to hold:

  1. it survives the leg-sum gate (a board that is not yet priced manufactures edge);
  2. it is not one cohort-month or one board type (jackknife, and by board_type);
  3. the 0.93-1.00 band's apparent +2.8c is priced against a LOSS RATE WE CANNOT ESTIMATE:
     at cost c the favourite must win c/(1) of the time just to break even, and 18
     observations cannot bound a 3% tail. Rule of three on 0/n.
  4. the win rate needed is compared with the win rate observed, per band, with a
     Clopper-Pearson-style bound rather than a point estimate.
"""

import json
import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from band_split import (BANDS, FEE_RATE, ADVERSE, build, clustered, economics,  # noqa: E402
                        fit_alpha_loo, ll_market, ll_sharp)


def beta_lo(k, n, conf=0.95):
    """Lower confidence bound on a binomial rate; exact for k == n (rule-of-three family)."""
    if n == 0:
        return float("nan")
    if k == n:
        return (1 - conf) ** (1.0 / n)
    lo, hi = 0.0, 1.0
    for _ in range(200):
        mid = (lo + hi) / 2
        # P(X >= k | p=mid)
        tail = sum(math.comb(n, i) * mid ** i * (1 - mid) ** (n - i) for i in range(k, n + 1))
        if tail > 1 - conf:
            hi = mid
        else:
            lo = mid
    return (lo + hi) / 2


def main(clob_dir):
    cases = build(clob_dir)
    months = sorted({c["month"] for c in cases})
    alpha = {m: fit_alpha_loo(cases, m) for m in months}

    print("## 3. Robustness of the fundable-band result\n")
    print("### a) after the leg-sum gate (priced books only, leg-sum <= 1.05)\n")
    print(f"{'band':22s} {'n':>4} {'mnth':>5} {'gap pp':>8} {'t':>6} {'pnl c':>8} {'t':>6} {'ANN%':>8}")
    for lo, hi, name in BANDS:
        sub = [c for c in cases if lo <= c["p_fav"] < hi and c["legsum"] <= 1.05]
        if not sub:
            continue
        g, gp, ga = defaultdict(list), defaultdict(list), defaultdict(list)
        for c in sub:
            g[c["month"]].append(float(c["fav_won"]) - c["p_fav"])
            x = economics(c, alpha[c["month"]])
            if x:
                gp[c["month"]].append(100 * x["pnl"])
                ga[c["month"]].append(100 * x["ann"])
        st, sp, sa = clustered(g), clustered(gp), clustered(ga)
        print(f"{name:22s} {len(sub):>4} {st['n_months']:>5} {100*st['mean']:>+8.1f} "
              f"{st['t']:>+6.2f} {sp['mean']:>+8.2f} {sp['t']:>+6.2f} {sa['mean']:>+8.1f}")

    print("\n### b) fundable band 0.60-0.90, leave-one-month-out jackknife (pnl c/trade)\n")
    sub = [c for c in cases if 0.60 <= c["p_fav"] < 0.90]
    g = defaultdict(list)
    for c in sub:
        x = economics(c, alpha[c["month"]])
        if x:
            g[c["month"]].append(100 * x["pnl"])
    ms = sorted(g)
    print(f"   full sample: {clustered(g)['mean']:+.2f}c over {len(ms)} months")
    for m in ms:
        sub2 = {k: v for k, v in g.items() if k != m}
        s = clustered(sub2)
        own = sum(g[m]) / len(g[m])
        print(f"   drop {m}: {s['mean']:+7.2f}c (t={s['t']:+.2f})   [{m} alone "
              f"{own:+7.2f}c on n={len(g[m])}]")

    print("\n### c) fundable band by board type\n")
    bt = defaultdict(list)
    for c in sub:
        bt[c["board_type"]].append(c)
    for k, v in sorted(bt.items(), key=lambda z: -len(z[1])):
        g2 = defaultdict(list)
        for c in v:
            g2[c["month"]].append(float(c["fav_won"]) - c["p_fav"])
        s = clustered(g2)
        wins = sum(c["fav_won"] for c in v)
        print(f"   {k:28s} n={len(v):3d} months={s['n_months']:2d} fav {wins}/{len(v)} "
              f"gap={100*s['mean']:+6.1f}pp")

    print("\n## 4. Breakeven arithmetic -- what each band needs to be true\n")
    print("buy favourite at cost c (= raw mid + 2c adverse), fee 0.04*c*(1-c) once.")
    print("break-even win rate q* solves (1-q*)*(-c-fee) + q*(1-c-fee) = 0\n")
    print(f"{'band':22s} {'n':>4} {'mean c':>7} {'q* need':>8} {'q obs':>7} "
          f"{'q 95% lo':>9} {'margin':>8}  verdict")
    for lo, hi, name in BANDS:
        s = [c for c in cases if lo <= c["p_fav"] < hi]
        e = [(c, economics(c, alpha[c["month"]])) for c in s]
        e = [(c, x) for c, x in e if x]
        if not e:
            continue
        mc = sum(x["cost"] for _, x in e) / len(e)
        fee = FEE_RATE * mc * (1 - mc)
        qstar = mc + fee
        k = sum(c["fav_won"] for c, _ in e)
        n = len(e)
        qobs = k / n
        qlo = beta_lo(k, n)
        margin = qlo - qstar
        v = "SURVIVES a 95% bound" if margin > 0 else "cannot be shown profitable"
        print(f"{name:22s} {n:>4} {mc:>7.3f} {qstar:>8.3f} {qobs:>7.3f} {qlo:>9.3f} "
              f"{margin:>+8.3f}  {v}")

    print("\n   (q 95% lo is the one-sided lower bound on the favourite's win rate from the")
    print("   observed k/n. A band whose LOWER BOUND sits below its break-even rate has not")
    print("   been shown to make money, however good the point estimate looks.)")

    print("\n### d) how many losses each band can absorb per 100 trades before it is flat\n")
    for lo, hi, name in BANDS:
        s = [c for c in cases if lo <= c["p_fav"] < hi]
        e = [(c, economics(c, alpha[c["month"]])) for c in s]
        e = [(c, x) for c, x in e if x]
        if not e:
            continue
        mc = sum(x["cost"] for _, x in e) / len(e)
        fee = FEE_RATE * mc * (1 - mc)
        win_pnl = 1 - mc - fee
        lose_pnl = -(mc + fee)
        n_break = 100 * win_pnl / (win_pnl - lose_pnl)
        print(f"   {name:22s} win +{100*win_pnl:5.2f}c / loss {100*lose_pnl:7.2f}c "
              f"-> {100-n_break:5.2f} losses per 100 wipes the band out "
              f"(observed losses in sample: {sum(1 for c, _ in e if not c['fav_won'])}/{len(e)})")


if __name__ == "__main__":
    main(sys.argv[1])
