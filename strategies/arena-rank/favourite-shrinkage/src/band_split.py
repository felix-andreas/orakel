"""DAY-3 PRE-REGISTERED KILL TEST: does the favourite-longshot gain live in a FUNDABLE band?

strategy.toml [trial].success_guideline commits us, in writing:

    the favourite-longshot gain must concentrate in a FUNDABLE band (favourite priced
    0.60-0.90). If the edge exists only on 0.93-0.99 favourites, return on locked capital
    after spread is too thin to justify a slot -> retire.

The variant's headline (+0.111 OOS log-loss, +9.2pp favourite underpricing) was measured
POOLED across every favourite price. This script decomposes it by the favourite's de-vigged
price at the checkpoint and prices each band as a business, per execution/DESIGN.md:

  * S3: return on locked capital, annualised = pnl / (capital_locked * days) * 365.
        Buying a favourite at c locks c per share for the whole hold.
  * S4: the venue taker fee, fee = shares * rate * p * (1-p), charged on entry (and on a
        market exit, never at resolution). These boards read feeType=tech_fees, rate=0.04.

Plus the two mandatory re-checks carried in memory/MEMORY.md:
  * leg-sum / checkpoint-artifact gate (wiki/reference/checkpoint-artifact.md) + null model
  * phantom-midpoint split (wiki/reference/phantom-midpoints.md): did the book actually move?

Usage:  python3 band_split.py <clob_dir>
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
from gate2_market import board_snapshot, load_hist, price_at  # noqa: E402
from gate_flb import sharpen, ALPHAS  # noqa: E402
from resolve import norm  # noqa: E402
from vintage import et_to_utc  # noqa: E402

EPS = 5e-3
CHECKPOINTS = [30, 14, 7]
FEE_RATE = 0.04          # tech_fees, read off the live feeSchedule 2026-07-26
ADVERSE = 0.02           # gate-4 convention: fill at raw mid + 2c

# The pre-registered bands. FUNDABLE is the band strategy.toml names.
BANDS = [(0.00, 0.60, "<0.60"), (0.60, 0.90, "FUNDABLE 0.60-0.90"),
         (0.90, 0.93, "0.90-0.93"), (0.93, 1.01, "0.93-1.00")]
FINE = [(0.00, 0.50, "[0.00,0.50)"), (0.50, 0.60, "[0.50,0.60)"),
        (0.60, 0.75, "[0.60,0.75)"), (0.75, 0.90, "[0.75,0.90)"),
        (0.90, 0.95, "[0.90,0.95)"), (0.95, 0.98, "[0.95,0.98)"),
        (0.98, 1.01, "[0.98,1.00]")]


def band_of(p, bands):
    for lo, hi, name in bands:
        if lo <= p < hi:
            return name
    return bands[-1][2]


def clustered(vals_by_month):
    """Mean and se treating each cohort-month as ONE observation.

    Within a month the boards mostly resolve to the same company, so board-level n is fake
    (backtest-2026-07-25.md S4). Every headline in this file is clustered this way.
    """
    per = [sum(v) / len(v) for v in vals_by_month.values() if v]
    n = len(per)
    if n == 0:
        return None
    mu = sum(per) / n
    if n == 1:
        return dict(mean=mu, se=float("nan"), t=float("nan"), n_months=1, pos=int(mu > 0))
    sd = math.sqrt(sum((x - mu) ** 2 for x in per) / (n - 1))
    se = sd / math.sqrt(n)
    return dict(mean=mu, se=se, t=(mu / se if se else float("nan")), n_months=n,
                pos=sum(1 for x in per if x > 0))


def build(clob_dir):
    """Satellite board-checkpoints with the de-vigged distribution, plus book diagnostics."""
    boards = json.load(open(f"{SAT}/data/poly/boards.json"))
    out = []
    for b in boards:
        if not (b["closed"] and b["board_type"] and b["check_et"]):
            continue
        if not isinstance(b["winner"], str):
            continue
        if b["board_type"] == "text_overall_nosc_1":   # the deep anchor is not a satellite
            continue
        T = et_to_utc(b["check_et"])
        tok = {l["company"]: l["token_id"] for l in b["legs"] if l["token_id"]}
        for d in CHECKPOINTS:
            t = T - timedelta(days=d)
            raw, dv = board_snapshot(b, clob_dir, t)
            if not dv:
                continue
            win = next((k for k in dv if norm(k) == norm(b["winner"])), None)
            if win is None:
                continue
            raw2, dv2 = board_snapshot(b, clob_dir, t + timedelta(hours=24))
            fav = max(dv, key=dv.get)
            # phantom / dead-book diagnostic: total variation of the FAVOURITE's own price
            # series over the 14 days before the checkpoint. A book that never moves is
            # not quoting a price (wiki/reference/phantom-midpoints.md).
            tv, npts = 0.0, 0
            h = load_hist(clob_dir, tok.get(fav, "")) if tok.get(fav) else None
            if h:
                lo, hi = int((t - timedelta(days=14)).timestamp()), int(t.timestamp())
                seg = [x["p"] for x in h if lo <= x["t"] <= hi]
                npts = len(seg)
                tv = sum(abs(seg[i] - seg[i - 1]) for i in range(1, len(seg)))
            out.append(dict(slug=b["slug"], board_type=b["board_type"],
                            month=b["check_et"][:7], d=d, dv=dv, raw=raw, raw2=raw2,
                            win=win, fav=fav, p_fav=dv[fav], fav_won=(fav == win),
                            legsum=sum(raw.values()), n_legs=len(dv),
                            volume=b["volume"], fav_tv=tv, fav_pts=npts))
    return out


def ll_market(c):
    return -math.log(min(max(c["dv"][c["win"]], EPS), 1 - EPS))


def ll_sharp(c, a):
    return -math.log(min(max(sharpen(c["dv"], a)[c["win"]], EPS), 1 - EPS))


def fit_alpha_loo(cases, month):
    sub = [c for c in cases if c["month"] != month]
    return min(ALPHAS, key=lambda a: sum(ll_sharp(c, a) for c in sub) / len(sub))


def economics(c, alpha, hold_days=None):
    """Price the favourite buy as a business, execution/DESIGN.md S3+S4.

    Signal: sharpened p on the de-vigged book. Fill: the RAW mid + adverse (de-vigging the
    fill shaves the overround off the cost basis and flatters every buy -- gate_flb.py).
    Capital locked buying YES at c is c per share, for the whole hold; the taker fee is
    charged on entry and NOT at resolution, so a held position pays it once.
    """
    c_raw = c["raw"].get(c["fav"])
    if c_raw is None:
        return None
    cost = c_raw + ADVERSE
    if not (0.01 < cost < 0.995):
        return None
    p_sharp = sharpen(c["dv"], alpha)[c["fav"]]
    fee = FEE_RATE * cost * (1 - cost)
    pay = 1.0 if c["fav_won"] else 0.0
    pnl = pay - cost - fee
    days = hold_days if hold_days is not None else c["d"]
    return dict(cost=cost, fee=fee, pnl=pnl, gross_edge=p_sharp - cost,
                mkt_edge=p_sharp - c["dv"][c["fav"]], capital=cost, days=days,
                rolc=pnl / cost, ann=(pnl / cost) * 365.0 / days)


def main(clob_dir):
    cases = build(clob_dir)
    months = sorted({c["month"] for c in cases})
    alpha = {m: fit_alpha_loo(cases, m) for m in months}
    print(f"# arena-rank/favourite-shrinkage -- day-3 fundable-band kill test")
    print(f"{len(cases)} satellite board-checkpoints, {len(months)} cohort-months "
          f"({months[0]} .. {months[-1]}), anchor board excluded\n")

    # ---------- 0. checkpoint-artifact gate ----------
    print("## 0. Checkpoint-artifact gate (wiki/reference/checkpoint-artifact.md)\n")
    ls = sorted(c["legsum"] for c in cases)
    print(f"leg-sum over all checkpoints: median {ls[len(ls)//2]:.3f}  "
          f"p10 {ls[int(.1*len(ls))]:.3f}  p90 {ls[int(.9*len(ls))]:.3f}  "
          f"share > 1.05: {sum(1 for x in ls if x > 1.05)}/{len(ls)}")
    for lo, hi, lab in [(0, 1.05, "leg-sum <= 1.05 (priced)"), (1.05, 9, "leg-sum > 1.05")]:
        sub = [c for c in cases if lo <= c["legsum"] < hi]
        if not sub:
            continue
        g = defaultdict(list)
        for c in sub:
            g[c["month"]].append(ll_market(c) - ll_sharp(c, alpha[c["month"]]))
        st = clustered(g)
        print(f"  {lab:26s} n={len(sub):3d}  LL gain={st['mean']:+.4f} "
              f"se={st['se']:.4f} t={st['t']:+.2f} months={st['n_months']}")

    print("\n### null models through the same pipeline (must LOSE to the market)")
    nulls = {}
    uni = defaultdict(list)
    fav1 = defaultdict(list)
    for c in cases:
        uni[c["month"]].append(math.log(c["n_legs"]) - ll_market(c))
        # "favourite wins, flat" null: p_fav=0.90, rest uniform
        q = 0.90 if c["fav"] == c["win"] else 0.10 / max(1, c["n_legs"] - 1)
        fav1[c["month"]].append(-math.log(max(q, EPS)) - ll_market(c))
    nulls["uniform over legs"] = clustered(uni)
    nulls["flat-0.90-on-favourite"] = clustered(fav1)
    for k, v in nulls.items():
        verdict = "NULL BEATS MARKET -- audit" if v["mean"] < 0 else "market wins (ok)"
        print(f"  {k:24s} market minus null LL = {-v['mean']:+.4f} -> {verdict}")

    print("\n### phantom-midpoint split: favourite's 14d total variation before checkpoint")
    for lo, hi, lab in [(0, 0.005, "DEAD (tv < 0.5c)"), (0.005, 0.05, "near-flat"),
                        (0.05, 99, "LIVE (tv >= 5c)")]:
        sub = [c for c in cases if lo <= c["fav_tv"] < hi]
        if not sub:
            continue
        g = defaultdict(list)
        gg = defaultdict(list)
        for c in sub:
            g[c["month"]].append(ll_market(c) - ll_sharp(c, alpha[c["month"]]))
            gg[c["month"]].append(float(c["fav_won"]) - c["p_fav"])
        st, sg = clustered(g), clustered(gg)
        print(f"  {lab:18s} n={len(sub):3d}  LL gain={st['mean']:+.4f} (t={st['t']:+.2f})"
              f"   fav gap={100*sg['mean']:+.1f}pp (t={sg['t']:+.2f})")

    # ---------- 1. the pre-registered band split ----------
    for title, bands in [("PRE-REGISTERED BANDS", BANDS), ("finer grid", FINE)]:
        print(f"\n## 1. Favourite-longshot gap by band -- {title}\n")
        print(f"{'band':22s} {'n':>4} {'mnth':>5} {'mean p_fav':>11} {'realised':>9} "
              f"{'gap pp':>8} {'se':>6} {'t':>6}  {'LLgain':>8} {'t':>6}")
        for lo, hi, name in bands:
            sub = [c for c in cases if lo <= c["p_fav"] < hi]
            if not sub:
                print(f"{name:22s} {0:>4}   --")
                continue
            g = defaultdict(list)
            gl = defaultdict(list)
            for c in sub:
                g[c["month"]].append(float(c["fav_won"]) - c["p_fav"])
                gl[c["month"]].append(ll_market(c) - ll_sharp(c, alpha[c["month"]]))
            st, sl = clustered(g), clustered(gl)
            mp = sum(c["p_fav"] for c in sub) / len(sub)
            rw = sum(c["fav_won"] for c in sub) / len(sub)
            print(f"{name:22s} {len(sub):>4} {st['n_months']:>5} {mp:>11.3f} {rw:>9.3f} "
                  f"{100*st['mean']:>+8.1f} {100*st['se']:>6.1f} {st['t']:>+6.2f}  "
                  f"{sl['mean']:>+8.4f} {sl['t']:>+6.2f}")

    print("\n### same split at T-7d only (the horizon this trial trades)\n")
    print(f"{'band':22s} {'n':>4} {'mnth':>5} {'mean p_fav':>11} {'realised':>9} {'gap pp':>8} {'t':>6}")
    for lo, hi, name in BANDS:
        sub = [c for c in cases if lo <= c["p_fav"] < hi and c["d"] == 7]
        if not sub:
            print(f"{name:22s} {0:>4}   --")
            continue
        g = defaultdict(list)
        for c in sub:
            g[c["month"]].append(float(c["fav_won"]) - c["p_fav"])
        st = clustered(g)
        mp = sum(c["p_fav"] for c in sub) / len(sub)
        rw = sum(c["fav_won"] for c in sub) / len(sub)
        print(f"{name:22s} {len(sub):>4} {st['n_months']:>5} {mp:>11.3f} {rw:>9.3f} "
              f"{100*st['mean']:>+8.1f} {st['t']:>+6.2f}")

    # ---------- 2. the business ----------
    print("\n## 2. The same bands priced as a business (execution/DESIGN.md S3+S4)\n")
    print("buy the favourite at raw mid +2c adverse, hold to check, taker fee 0.04*p*(1-p)")
    print("charged once (entry; settlement is a redemption, not a match)\n")
    print(f"{'band':22s} {'n':>4} {'cost':>6} {'fee c':>6} {'pnl c':>8} {'se':>6} {'t':>6} "
          f"{'RoLC%':>7} {'days':>5} {'ANN%':>9}")
    econ_rows = {}
    for lo, hi, name in BANDS:
        sub = [c for c in cases if lo <= c["p_fav"] < hi]
        e = [(c, economics(c, alpha[c["month"]])) for c in sub]
        e = [(c, x) for c, x in e if x]
        if not e:
            print(f"{name:22s} {0:>4}   --")
            continue
        g = defaultdict(list)
        ga = defaultdict(list)
        for c, x in e:
            g[c["month"]].append(100 * x["pnl"])
            ga[c["month"]].append(100 * x["ann"])
        st, sa = clustered(g), clustered(ga)
        mc = sum(x["cost"] for _, x in e) / len(e)
        mf = sum(x["fee"] for _, x in e) / len(e)
        md = sum(x["days"] for _, x in e) / len(e)
        rolc = st["mean"] / (100 * mc)
        econ_rows[name] = dict(n=len(e), cost=mc, fee=mf, pnl_c=st["mean"], se=st["se"],
                               t=st["t"], rolc=100 * rolc, days=md, ann=sa["mean"],
                               months=st["n_months"])
        print(f"{name:22s} {len(e):>4} {mc:>6.3f} {100*mf:>6.2f} {st['mean']:>+8.2f} "
              f"{st['se']:>6.2f} {st['t']:>+6.2f} {100*rolc:>+7.1f} {md:>5.1f} "
              f"{sa['mean']:>+9.1f}")

    print("\n### and with a 5-day hold (today -> 2026-07-31 check), same realised outcomes\n")
    print(f"{'band':22s} {'n':>4} {'pnl c':>8} {'RoLC%':>7} {'ANN% @5d':>9}")
    for lo, hi, name in BANDS:
        sub = [c for c in cases if lo <= c["p_fav"] < hi]
        e = [(c, economics(c, alpha[c["month"]], hold_days=5)) for c in sub]
        e = [(c, x) for c, x in e if x]
        if not e:
            continue
        g = defaultdict(list)
        ga = defaultdict(list)
        for c, x in e:
            g[c["month"]].append(100 * x["pnl"])
            ga[c["month"]].append(100 * x["ann"])
        st, sa = clustered(g), clustered(ga)
        mc = sum(x["cost"] for _, x in e) / len(e)
        print(f"{name:22s} {len(e):>4} {st['mean']:>+8.2f} {st['mean']/(100*mc):>+7.1%} "
              f"{sa['mean']:>+9.1f}")

    json.dump(dict(alpha_loo=alpha, econ=econ_rows,
                   cases=[{k: v for k, v in c.items() if k not in ("dv", "raw", "raw2")}
                          for c in cases]),
              open(f"{os.path.dirname(os.path.dirname(os.path.abspath(__file__)))}"
                   f"/data/band_split.json", "w"), indent=1)


if __name__ == "__main__":
    main(sys.argv[1])
