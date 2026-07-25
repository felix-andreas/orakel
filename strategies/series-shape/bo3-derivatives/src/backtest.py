#!/usr/bin/env python3
"""series-shape/bo3-derivatives — checkpoint extraction + the gate-0 artifact hunt.

Reproduces the idea's headline tables on an independently harvested sample, then
decomposes them by BOOK QUALITY.  The decomposition is the point: the idea's edge is
measured against `outcomePrices`/CLOB midpoints, and on Polymarket a derivative leg with
no orders quotes a ~0.50 midpoint (the mean of a 0.01 bid and a 0.99 ask).  Averaging
"market price" over a sample containing such legs manufactures an arbitrarily large
apparent mispricing that never existed as a tradeable quote.

  checkpoints <dir>     -> cp.jsonl   (T-24h / T-6h / T-1h / T-15m per leg)
  artifact    <dir>     -> the decomposition
"""
import json
import math
import os
import sys
import datetime as dt
from collections import defaultdict

CHECKPOINTS = [("T-24h", 86400), ("T-6h", 21600), ("T-1h", 3600), ("T-15m", 900)]


def is_bo3(r):
    import re
    m = re.search(r'wins (\d+) or more (?:games|maps)', r["hc"]["desc"] or "")
    t = re.search(r'play (\d+) or more (?:games|maps)', r["tot"]["desc"] or "")
    return bool(m and t and int(m.group(1)) == 2 and int(t.group(1)) == 3)


def load_hist(d, tok):
    p = os.path.join(d, "clob", f"{tok}.json")
    if not os.path.exists(p):
        return None
    try:
        return json.load(open(p)).get("history") or None
    except Exception:
        return None


def price_at(hist, t):
    """Last observation at or before t. Returns (price, staleness_seconds) or (None,None).
    NEVER interpolates forward -- a forward-looking read is the classic backtest bug."""
    lo, hi, best = 0, len(hist) - 1, None
    while lo <= hi:
        mid = (lo + hi) // 2
        if hist[mid]["t"] <= t:
            best = mid
            lo = mid + 1
        else:
            hi = mid - 1
    if best is None:
        return None, None
    return hist[best]["p"], t - hist[best]["t"]


def prematch_stats(hist, t0, t1):
    """Shape of the pre-match price path in [t0, t1] -- the dead-book detector.
    A book with no orders prints a flat 0.50 line; a real book moves."""
    v = [h["p"] for h in hist if t0 <= h["t"] <= t1]
    if not v:
        return None
    n = len(v)
    m = sum(v) / n
    sd = (sum((x - m) ** 2 for x in v) / n) ** 0.5
    return {"n": n, "mean": m, "sd": sd, "distinct": len(set(v)),
            "min": min(v), "max": max(v)}


def cmd_checkpoints(d):
    out = open(os.path.join(d, "cp.jsonl"), "w")
    n = kept = 0
    for line in open(os.path.join(d, "triples.jsonl")):
        r = json.loads(line)
        n += 1
        if not r["resolved"] or not is_bo3(r):
            continue
        gst = r["ml"]["gst"] or r["hc"]["gst"]
        if not gst:
            continue
        T = int(dt.datetime.fromisoformat(gst.replace("+00", "+00:00")).timestamp())
        rec = {"slug": r["event_slug"], "title": r["title"], "end": r["end"], "T": T,
               "month": (r["end"] or "")[:7],
               "game": (r["title"] or "?").split(":")[0].strip()}
        ok = True
        for tag in ("ml", "hc", "tot"):
            hist = load_hist(d, r[tag]["tok"][0])
            if not hist:
                ok = False
                break
            leg = {"vol": r[tag]["vol"], "out": r[tag]["out"], "px": r[tag]["px"],
                   "cid": r[tag]["cid"], "tok": r[tag]["tok"][0]}
            for nm, off in CHECKPOINTS:
                p, st = price_at(hist, T - off)
                leg[nm] = p
                leg[nm + "_stale"] = st
            leg["pre"] = prematch_stats(hist, T - 21600, T - 900)   # T-6h .. T-15m
            leg["post"] = prematch_stats(hist, T + 900, T + 21600)  # sanity: in-play
            leg["first_t"] = hist[0]["t"] - T
            leg["last_t"] = hist[-1]["t"] - T
            # settled winner
            px = [float(x) for x in r[tag]["px"]]
            leg["win0"] = 1 if px[0] == 1.0 else 0
            rec[tag] = leg
        if not ok:
            continue
        # -1.5 side's index inside the moneyline outcomes (name match, with a fallback)
        hs = r["hc"]["out"][0]
        rec["hc_side_ml_idx"] = (0 if hs == r["ml"]["out"][0]
                                 else 1 if hs == r["ml"]["out"][1] else None)
        out.write(json.dumps(rec) + "\n")
        kept += 1
    out.close()
    print(f"{kept} BO3 triples with full CLOB history (of {n} harvested triples)")


# ---------------------------------------------------------------- artifact hunt

def wilson_se(p, n):
    return math.sqrt(max(p * (1 - p), 1e-9) / max(n, 1))


def tbl(rows, key, label, price_fn, real_fn, buckets):
    print(f"\n  -- {label} --")
    print(f"    {'bucket':>22s} {'n':>6s} {'mean mkt':>9s} {'realised':>9s} {'gap pp':>8s} {'se pp':>7s}")
    for name, pred in buckets:
        sub = [r for r in rows if pred(r)]
        if not sub:
            continue
        p = [price_fn(r) for r in sub]
        y = [real_fn(r) for r in sub]
        mp, my = sum(p) / len(p), sum(y) / len(y)
        print(f"    {name:>22s} {len(sub):6d} {mp:9.4f} {my:9.4f} {100*(my-mp):+8.2f} "
              f"{100*wilson_se(my, len(sub)):7.2f}")


def cmd_artifact(d, cp="T-1h"):
    rows = []
    for line in open(os.path.join(d, "cp.jsonl")):
        r = json.loads(line)
        if r["hc_side_ml_idx"] is None:
            continue
        if any(r[t][cp] is None for t in ("ml", "hc", "tot")):
            continue
        # moneyline probability of the -1.5 side
        p0 = r["ml"][cp]
        r["ml_hcside"] = p0 if r["hc_side_ml_idx"] == 0 else 1 - p0
        r["hc_px"] = r["hc"][cp]
        r["ov_px"] = r["tot"][cp]
        r["hc_win"] = r["hc"]["win0"]           # did the -1.5 side cover
        r["ov_win"] = r["tot"]["win0"]          # did the series go 3 maps
        r["ml_win"] = 1 if ((r["ml"]["win0"] == 1) == (r["hc_side_ml_idx"] == 0)) else 0
        rows.append(r)
    print(f"# {len(rows)} BO3 series with a {cp} price on all three legs")

    def deadness(r, tag):
        """A leg is 'dead' if its pre-match path never moved -- no orders, midpoint is
        the mean of a 1c bid and a 99c ask."""
        pre = r[tag]["pre"]
        if not pre:
            return "no_pre_data"
        if pre["sd"] < 1e-9:
            return "flat"
        if pre["sd"] < 0.005:
            return "near_flat"
        return "moving"

    for tag, lbl in (("hc", "map handicap -1.5"), ("tot", "totals Over 2.5")):
        for r in rows:
            r[f"{tag}_dead"] = deadness(r, tag)

    # ---- 1. the idea's headline table, reproduced as-is
    bands = [(f"ML {a:.1f}-{b:.1f}", (lambda a=a, b=b: (lambda r: a <= r["ml_hcside"] < b))())
             for a, b in [(0.5, 0.6), (0.6, 0.7), (0.7, 0.8), (0.8, 0.9), (0.9, 1.0)]]
    print("\n[1] REPRODUCTION of the idea's banded table (no book filter at all)")
    tbl(rows, None, "moneyline leg (-1.5 side)", lambda r: r["ml_hcside"],
        lambda r: r["ml_win"], bands)
    tbl(rows, None, "handicap leg", lambda r: r["hc_px"], lambda r: r["hc_win"], bands)
    tbl(rows, None, "totals Over leg", lambda r: r["ov_px"], lambda r: r["ov_win"], bands)

    # ---- 2. the same numbers decomposed by book quality
    print("\n[2] SAME EDGE, DECOMPOSED BY BOOK QUALITY (the artifact hunt)")
    qb = [("hc book flat (dead)", lambda r: r["hc_dead"] == "flat"),
          ("hc near-flat", lambda r: r["hc_dead"] == "near_flat"),
          ("hc moving", lambda r: r["hc_dead"] == "moving")]
    tbl(rows, None, "handicap: market vs realised by liveness",
        lambda r: r["hc_px"], lambda r: r["hc_win"], qb)
    vb = [("hc vol = 0", lambda r: (r["hc"]["vol"] or 0) == 0),
          ("hc vol 0-1k", lambda r: 0 < (r["hc"]["vol"] or 0) <= 1000),
          ("hc vol 1k-5k", lambda r: 1000 < (r["hc"]["vol"] or 0) <= 5000),
          ("hc vol 5k-20k", lambda r: 5000 < (r["hc"]["vol"] or 0) <= 20000),
          ("hc vol > 20k", lambda r: (r["hc"]["vol"] or 0) > 20000)]
    tbl(rows, None, "handicap: market vs realised by leg volume",
        lambda r: r["hc_px"], lambda r: r["hc_win"], vb)
    tb = [("tot vol = 0", lambda r: (r["tot"]["vol"] or 0) == 0),
          ("tot vol 0-1k", lambda r: 0 < (r["tot"]["vol"] or 0) <= 1000),
          ("tot vol 1k-5k", lambda r: 1000 < (r["tot"]["vol"] or 0) <= 5000),
          ("tot vol > 5k", lambda r: (r["tot"]["vol"] or 0) > 5000)]
    tbl(rows, None, "totals Over: market vs realised by leg volume",
        lambda r: r["ov_px"], lambda r: r["ov_win"], tb)
    mb = [("ml vol < 5k", lambda r: (r["ml"]["vol"] or 0) < 5000),
          ("ml vol 5k-20k", lambda r: 5000 <= (r["ml"]["vol"] or 0) < 20000),
          ("ml vol 20k-50k", lambda r: 20000 <= (r["ml"]["vol"] or 0) < 50000),
          ("ml vol >= 50k", lambda r: (r["ml"]["vol"] or 0) >= 50000)]
    tbl(rows, None, "moneyline: market vs realised by leg volume",
        lambda r: r["ml_hcside"], lambda r: r["ml_win"], mb)

    # ---- 3. the tradeable subset: live book AND fundable band
    live = [r for r in rows if r["hc_dead"] == "moving" and (r["hc"]["vol"] or 0) > 5000]
    print(f"\n[3] TRADEABLE SUBSET (handicap book moving AND vol > $5k): n={len(live)}")
    tbl(live, None, "handicap, live books only", lambda r: r["hc_px"],
        lambda r: r["hc_win"], bands + [("all", lambda r: True),
                                        ("band 0.20-0.60", lambda r: 0.20 <= r["hc_px"] <= 0.60)])
    livet = [r for r in rows if r["tot_dead"] == "moving" and (r["tot"]["vol"] or 0) > 2000]
    print(f"\n    totals live subset (moving AND vol > $2k): n={len(livet)}")
    tbl(livet, None, "totals Over, live books only", lambda r: r["ov_px"],
        lambda r: r["ov_win"], [("all", lambda r: True)] + bands)

    # ---- 4. monthly clustering on the live subset
    print("\n[4] MONTHLY CLUSTERING on the live subsets (gate 3 sign-stability)")
    for nm, sub, pf, rf in (("handicap", live, lambda r: r["hc_px"], lambda r: r["hc_win"]),
                            ("totals Over", livet, lambda r: r["ov_px"], lambda r: r["ov_win"])):
        by = defaultdict(list)
        for r in sub:
            by[r["month"]].append(rf(r) - pf(r))
        ms = []
        print(f"    {nm}:")
        for m in sorted(by):
            v = by[m]
            if len(v) < 5:
                continue
            mu = sum(v) / len(v)
            ms.append(mu)
            print(f"      {m}  n={len(v):4d}  mean gap {100*mu:+6.2f}pp")
        if len(ms) > 1:
            mm = sum(ms) / len(ms)
            sd = (sum((x - mm) ** 2 for x in ms) / (len(ms) - 1)) ** 0.5
            se = sd / len(ms) ** 0.5
            print(f"      -> monthly-clustered mean {100*mm:+.2f}pp  se {100*se:.2f}pp  "
                  f"t={mm/se if se else float('nan'):+.2f}  positive {sum(x>0 for x in ms)}/{len(ms)}")

    # ---- 5. timestamp sanity: is the checkpoint really pre-match?
    print("\n[5] TIMESTAMP ALIGNMENT: is the checkpoint really pre-match?")
    coll = sum(1 for r in rows if r["ml"][cp] is not None
               and (r["ml"][cp] <= 0.002 or r["ml"][cp] >= 0.998))
    print(f"    moneyline price already collapsed (<=0.002 or >=0.998) at {cp}: "
          f"{coll}/{len(rows)} = {coll/len(rows):.4f}")
    stales = sorted((r["ml"][cp + "_stale"] or 0) for r in rows)
    print(f"    staleness of the {cp} read (s): median {stales[len(stales)//2]}, "
          f"p90 {stales[int(0.9*len(stales))]}, max {stales[-1]}")
    pre_sd = sorted((r["hc"]["pre"] or {}).get("sd", 0) for r in rows)
    post_sd = sorted((r["hc"]["post"] or {}).get("sd", 0) for r in rows)
    print(f"    handicap pre-match path sd: median {pre_sd[len(pre_sd)//2]:.4f}; "
          f"post-start sd median {post_sd[len(post_sd)//2]:.4f} (should be much larger)")
    firsts = sorted(r["hc"]["first_t"] for r in rows)
    print(f"    handicap book first print vs gameStartTime (s): median "
          f"{firsts[len(firsts)//2]}, p10 {firsts[int(0.1*len(firsts))]}")


if __name__ == "__main__":
    {"checkpoints": cmd_checkpoints, "artifact": cmd_artifact}[sys.argv[1]](*sys.argv[2:])
