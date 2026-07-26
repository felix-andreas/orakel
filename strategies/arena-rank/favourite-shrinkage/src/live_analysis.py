"""Price the live July cohort against the fundable-band finding, on today's book.

Three questions per board, in the order that can kill it:

  1. CHECKPOINT GATE. De-vigged leg-sum <= ~1.05 (wiki/reference/checkpoint-artifact.md)
     and a live book (wiki/reference/phantom-midpoints.md).
  2. BAND. Where does the favourite actually sit TODAY? Yesterday's [book] blocks are
     already stale -- the Chinese favourite moved 0.8275 -> 0.7765 overnight.
  3. BUSINESS. At the EXECUTABLE ask (not the midpoint -- wiki/reference/midpoint-is-not-a-fill.md),
     walking the ask side for a real stake, with the venue taker fee, held to the
     2026-07-31 12:00 ET check: what is the annualised return on locked capital, and how
     many losses per 100 does the band survive?

Plus the fill evidence the book alone cannot give: realised taker flow on the side we
would take. A taker who BOUGHT the favourite proves a resting ask existed at that price.
No-side trades are folded into Yes-equivalent units exactly as tools/fillcheck does
(yes-equivalent price = p if outcome==Yes else 1-p; taker_sold flips with the token).
"""

import json
import math
import os
import sys
from datetime import datetime, timezone

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ALPHA = 1.75
CLIP = (0.003, 0.995)
FEE_RATE = 0.04                      # feeType=tech_fees, rate 0.04, takerOnly (read 2026-07-26)
CHECK = datetime(2026, 7, 31, 16, 0, tzinfo=timezone.utc)   # 12:00 ET
NOW = datetime(2026, 7, 26, 12, 0, tzinfo=timezone.utc)
STAKE_USD = 500.0                    # a real slot-sized clip, not one share
FUNDABLE = (0.60, 0.90)


def sharpen(dv, a):
    p = {k: max(v, 1e-6) ** a for k, v in dv.items()}
    s = sum(p.values())
    out = {k: v / s for k, v in p.items()}
    return {k: min(max(v, CLIP[0]), CLIP[1]) for k, v in out.items()}


def walk_ask(levels_price_size, usd):
    """Average price to BUY `usd` of notional by lifting asks. levels are (price, size)."""
    spend, shares = 0.0, 0.0
    for p, s in sorted(levels_price_size):
        take = min(s, (usd - spend) / p) if p > 0 else 0
        if take <= 0:
            break
        spend += take * p
        shares += take
        if spend >= usd - 1e-9:
            break
    if shares == 0:
        return None, 0.0
    return spend / shares, spend


def main():
    date = sys.argv[1] if len(sys.argv) > 1 else "2026-07-26"
    live = json.load(open(f"{ROOT}/data/live-{date}.json"))
    days = (CHECK - NOW).total_seconds() / 86400.0
    print(f"# Live July cohort on {date} -- {days:.2f} days to the 2026-07-31 12:00 ET check")
    print(f"# alpha={ALPHA}, clip {CLIP}, taker fee {FEE_RATE}*p*(1-p) charged once (entry)\n")

    out = {}
    print("## A. Checkpoint gate + band, on TODAY's book\n")
    print(f"{'board':16s} {'favourite':11s} {'mid':>7} {'bid':>6} {'ask':>6} {'spr c':>6} "
          f"{'legsum':>7} {'devig':>7} {'sharp':>7} {'band':>10} {'gate':>6}")
    for key, b in live.items():
        raw = {l["company"]: l["book"]["mid"] for l in b["legs"]
               if l["book"] and l["book"]["mid"] is not None}
        if len(raw) < 2:
            continue
        legsum = sum(raw.values())
        dv = {k: v / legsum for k, v in raw.items()}
        fav = max(dv, key=dv.get)
        sh = sharpen(dv, ALPHA)
        fl = next(l for l in b["legs"] if l["company"] == fav)
        bk = fl["book"]
        band_ok = FUNDABLE[0] <= dv[fav] < FUNDABLE[1]
        gate = legsum <= 1.05 and bk["spread"] <= 0.05 and bk["depth_10c_usd"] >= 500
        out[key] = dict(board=key, slug=b["slug"], board_type=b["board_type"], fav=fav,
                        leg=fl, raw=raw, dv=dv, sharp=sh, legsum=legsum,
                        band_ok=band_ok, gate=gate)
        print(f"{key:16s} {fav:11s} {bk['mid']:>7.4f} {bk['best_bid']:>6.3f} "
              f"{bk['best_ask']:>6.3f} {100*bk['spread']:>6.2f} {legsum:>7.4f} "
              f"{dv[fav]:>7.4f} {sh[fav]:>7.4f} "
              f"{'FUNDABLE' if band_ok else 'out':>10} {'ok' if gate else 'FAIL':>6}")

    print("\n## B. The business at the executable ask (walking the book for $500)\n")
    print(f"{'board':16s} {'touch':>6} {'avg fill':>9} {'$filled':>8} {'edge c':>7} "
          f"{'fee c':>6} {'RoLC%':>7} {'ANN%':>8} {'q* need':>8} {'loss/100':>9}")
    for key, r in out.items():
        bk = r["leg"]["book"]
        # reconstruct the ask ladder from the recorded depth measurements: we only kept
        # aggregates, so use touch price for the marginal share and the 5c-window average
        # as the realistic clip price.
        touch = bk["best_ask"]
        avg5, filled = None, 0.0
        if bk["ask_depth_5c_shares"] > 0:
            avg5 = bk["ask_depth_5c_usd"] / bk["ask_depth_5c_shares"]
            filled = min(STAKE_USD, bk["ask_depth_5c_usd"])
        cost = avg5 if (avg5 and bk["ask_depth_5c_usd"] < STAKE_USD) else touch
        # $500 usually sits inside the touch level; use touch if it fits, else the 5c VWAP
        if bk["ask_shares_at_touch"] * touch >= STAKE_USD:
            cost = touch
        fee = FEE_RATE * cost * (1 - cost)
        p_model = r["sharp"][r["fav"]]
        edge = p_model - cost - fee
        # expected pnl per share under our own model, held to resolution
        pnl = p_model * (1 - cost - fee) + (1 - p_model) * (-(cost + fee))
        rolc = pnl / cost
        ann = rolc * 365.0 / days
        qstar = cost + fee
        win_pnl, lose_pnl = 1 - cost - fee, -(cost + fee)
        loss_per_100 = 100 * win_pnl / (win_pnl - lose_pnl)
        r.update(cost=cost, fee=fee, p_model=p_model, pnl=pnl, rolc=rolc, ann=ann,
                 qstar=qstar, loss_per_100=loss_per_100, filled=filled, days=days)
        print(f"{key:16s} {touch:>6.3f} {cost:>9.4f} {filled:>8.0f} {100*edge:>+7.2f} "
              f"{100*fee:>6.2f} {100*rolc:>+7.2f} {100*ann:>+8.1f} {qstar:>8.4f} "
              f"{loss_per_100:>9.2f}")
    print("\n   q* need = the favourite's break-even win rate at this cost incl. fee.")
    print("   loss/100 = losses per 100 trades that take the band to zero.")

    print("\n## C. Realised taker flow on the side we would take (Data API tape)\n")
    print("A taker who BOUGHT the favourite (yes-equivalent) proves a resting ask existed.")
    print(f"{'board':16s} {'buys 7d':>8} {'$ 7d':>10} {'buys 30d':>9} {'$ 30d':>11} "
          f"{'best ask 7d':>12} {'$<=our cost 7d':>15}")
    t7 = NOW.timestamp() - 7 * 86400
    t30 = NOW.timestamp() - 30 * 86400
    for key, r in out.items():
        fills = []
        for t in r["leg"]["trades"]:
            yes = t["outcome"].lower() == "yes"
            p = t["price"] if yes else 1.0 - t["price"]
            sold = (t["side"].upper() == "SELL") if yes else (t["side"].upper() == "BUY")
            fills.append((t["timestamp"], p, sold, float(t["size"])))
        buys7 = [f for f in fills if f[0] >= t7 and not f[2]]
        buys30 = [f for f in fills if f[0] >= t30 and not f[2]]
        n7, u7 = len(buys7), sum(f[1] * f[3] for f in buys7)
        n30, u30 = len(buys30), sum(f[1] * f[3] for f in buys30)
        best7 = min((f[1] for f in buys7), default=None)
        good7 = sum(f[1] * f[3] for f in buys7 if f[1] <= r["cost"] + 1e-9)
        r.update(taker_buys_7d=n7, taker_buy_usd_7d=u7, taker_buys_30d=n30,
                 taker_buy_usd_30d=u30, best_ask_traded_7d=best7,
                 usd_at_or_below_cost_7d=good7)
        print(f"{key:16s} {n7:>8} {u7:>10,.0f} {n30:>9} {u30:>11,.0f} "
              f"{(f'{best7:.3f}' if best7 else '--'):>12} {good7:>15,.0f}")

    print("\n## D. Verdict per board\n")
    for key, r in out.items():
        why = []
        if not r["gate"]:
            why.append("book gate")
        if not r["band_ok"]:
            why.append(f"favourite {r['dv'][r['fav']]:.3f} outside fundable {FUNDABLE}")
        if r["taker_buy_usd_7d"] < 500:
            why.append(f"only ${r['taker_buy_usd_7d']:,.0f} of taker buys in 7d")
        v = "TRADEABLE" if not why else "NOT FUNDABLE: " + "; ".join(why)
        print(f"  {key:16s} {v}")

    json.dump({k: {kk: vv for kk, vv in v.items() if kk != "leg"} for k, v in out.items()},
              open(f"{ROOT}/data/live-analysis-{date}.json", "w"), indent=1)


if __name__ == "__main__":
    main()
