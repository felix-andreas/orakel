#!/usr/bin/env python3
"""Day-1 backtest for climate-nowcast/gistemp-era5: the idea's 5 gates on resolved instances.

Point-in-time discipline:
  - ERA5 dailies available at time t: date <= t - 3 days (observed feed lag 2-3 d).
  - GISTEMP available at time t: the latest Wayback vintage captured <= t (never today's file).
  - All regressions fit on years strictly before the predicted month's year.
  - sigma per checkpoint estimated on a PRE-SAMPLE hindcast (2015-2023, before any
    backtested market existed), then held fixed across the 2024-2026 sample.

Usage: python3 backtest.py <pulldir> <vintages.csv> <era5.csv> <gistemp_current.csv> <outdir>
"""

import csv
import json
import math
import os
import re
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone

import numpy as np

ERA5_LAG_DAYS = 3
SPREAD_PROXY = 0.04  # assumed half-turn book width for resolved instances (no book history)
EDGE_THRESHOLD = 0.06  # |model - mid| must exceed spread proxy + 2c
ADVERSE_FILL = 0.02  # half the proxied spread, paid on top of the fill mid
FUNDABLE = (0.03, 0.50)  # price zone for the token actually bought
FLOOR = 0.005  # per-leg probability floor before renormalisation

MONTHS = {m: i + 1 for i, m in enumerate(
    ["January", "February", "March", "April", "May", "June", "July", "August",
     "September", "October", "November", "December"])}


# ---------------------------------------------------------------- data loading

def load_era5(path):
    """date -> anomaly (1991-2020 baseline). Includes preliminary rows."""
    out = {}
    for row in csv.DictReader(l for l in open(path) if not l.startswith("#")):
        out[datetime.strptime(row["date"], "%Y-%m-%d").date()] = float(row["ano_91-20"])
    return out


def era5_monthly(era5):
    """(y,m) -> full-month mean anomaly, complete months only."""
    acc = defaultdict(list)
    for d, v in era5.items():
        acc[(d.year, d.month)].append(v)
    out = {}
    for (y, m), vs in acc.items():
        ndays = ((datetime(y + m // 12, m % 12 + 1, 1) - datetime(y, m, 1)).days)
        if len(vs) == ndays:
            out[(y, m)] = float(np.mean(vs))
    return out


def load_vintages(path):
    """capture datetime -> {(y,m): anom}. txt and csv captures merged per timestamp."""
    caps = defaultdict(dict)
    for row in csv.DictReader(open(path)):
        ts = datetime.strptime(row["capture_ts"], "%Y%m%d%H%M%S").replace(tzinfo=timezone.utc)
        caps[ts][(int(row["year"]), int(row["month"]))] = float(row["anom_c"])
    return dict(sorted(caps.items()))


def load_gistemp_current(path):
    out = {}
    rows = list(csv.reader(open(path)))
    hdr = next(i for i, r in enumerate(rows) if r and r[0] == "Year")
    for r in rows[hdr + 1:]:
        if not r or not re.match(r"^\d{4}$", r[0]):
            continue
        for i in range(12):
            try:
                out[(int(r[0]), i + 1)] = float(r[i + 1])
            except (ValueError, IndexError):
                pass
    return out


def vintage_at(caps, t):
    """GISTEMP table as knowable at time t (latest capture <= t). None if none."""
    best = None
    for ts, table in caps.items():
        if ts <= t:
            best = table
    return best


def first_prints(caps):
    """(y,m) -> value in the earliest capture containing that month."""
    out = {}
    for ts in sorted(caps):
        for key, v in caps[ts].items():
            out.setdefault(key, (ts, v))
    return {k: v for k, (ts, v) in out.items()}, {k: ts for k, (ts, v) in out.items()}


# ---------------------------------------------------------------- market parsing

BUCKET_RES = [
    (re.compile(r"less than (\d+\.\d+)"), "lt"),
    (re.compile(r"(?:more|greater) than (\d+\.\d+)"), "gt"),
    (re.compile(r"between (\d+\.\d+)[^\d]{1,6}(\d+\.\d+)"), "between"),
]
MONTH_RE = re.compile(
    r"(January|February|March|April|May|June|July|August|September|October|November|December)"
    r"\s+(\d{4})")


def parse_bucket(question):
    """-> (lo, hi) continuous bounds; lo=None => -inf, hi=None => +inf.

    Bucket 'between 1.20 and 1.24' wins for prints 1.20..1.24 -> continuous
    [1.195, 1.245). 'less than 1.10' -> (-inf, 1.095). 'more than 1.29' -> [1.295, inf).
    """
    q = question.replace("–", "-").replace("ºC", "C").replace("°C", "C")
    for rx, kind in BUCKET_RES:
        m = rx.search(q)
        if m:
            if kind == "lt":
                return None, float(m.group(1)) - 0.005
            if kind == "gt":
                return float(m.group(1)) + 0.005, None
            return float(m.group(1)) - 0.005, float(m.group(2)) + 0.005
    raise ValueError(f"unparseable bucket: {question}")


def parse_month(questions, slug):
    """Month/year from any question text; fall back to the event slug.

    NOTE the -394 trap: event slug says july-2025 but every question says August 2025 —
    questions are authoritative, slug is the fallback only."""
    for q in questions:
        m = MONTH_RE.search(q)
        if m:
            return int(m.group(2)), MONTHS[m.group(1)]
    s = slug.lower()
    for name, num in MONTHS.items():
        if name.lower() in s:
            ym = re.search(r"(20\d\d)", s)
            if ym:
                return int(ym.group(1)), num
    raise ValueError(f"no month in questions or slug: {slug}")


def load_instances(pulldir, monthly_slugs):
    """One instance per event: legs with buckets, winner, checkpoint scaffolding."""
    legs = list(csv.DictReader(open(os.path.join(pulldir, "legs.csv"))))
    ev = defaultdict(list)
    for l in legs:
        ev[l["event_slug"]].append(l)
    instances = []
    for slug in monthly_slugs:
        rows = ev[slug]
        if not rows:
            continue
        y, m = parse_month([r["question"] for r in rows], slug)
        buckets = []
        for l in rows:
            lo, hi = parse_bucket(l["question"])
            buckets.append({
                "lo": lo, "hi": hi,
                "question": l["question"],
                "market_slug": l["market_slug"],
                "condition_id": l["condition_id"],
                "token_yes": l["token_yes"],
                "winner": l["price_yes"] == "1",
                "volume": float(l["volume"] or 0),
            })
        buckets.sort(key=lambda b: (-1e9 if b["lo"] is None else b["lo"]))
        closed = rows[0]["event_closedTime"]
        closed_dt = (datetime.strptime(closed[:19], "%Y-%m-%dT%H:%M:%S")
                     .replace(tzinfo=timezone.utc) if closed else None)
        start = rows[0]["event_start"]
        start_dt = (datetime.strptime(start[:19], "%Y-%m-%dT%H:%M:%S")
                    .replace(tzinfo=timezone.utc) if start else None)
        instances.append({
            "slug": slug, "year": y, "month": m, "buckets": buckets,
            "closed_dt": closed_dt, "start_dt": start_dt,
            "resolved": rows[0]["event_closed"] == "True",
        })
    instances.sort(key=lambda i: (i["year"], i["month"], i["slug"]))
    return instances


def load_price_history(pulldir, token):
    h = json.load(open(os.path.join(pulldir, "prices_history", f"{token}.json")))["history"]
    return [(datetime.fromtimestamp(p["t"], tz=timezone.utc), p["p"]) for p in h]


def mid_at(hist, t, forward_grace_days=5):
    """Last mid <= t; else first within grace (flagged conservative substitute)."""
    past = [p for ts, p in hist if ts <= t]
    if past:
        return past[-1], False
    fut = [(ts, p) for ts, p in hist if ts > t]
    if fut and (fut[0][0] - t).days <= forward_grace_days:
        return fut[0][1], True
    return None, False


# ---------------------------------------------------------------- nowcast model

def era5_projection(era5, y, m, t):
    """Project full-month ERA5 anomaly for (y,m) from dailies available at t.

    Regression full_mean ~ mean(first k days), fit on same calendar month
    1979..y-1. Returns (mu, resid_sd, k)."""
    cutoff = (t - timedelta(days=ERA5_LAG_DAYS)).date()
    avail = [era5[d] for d in sorted(era5)
             if d.year == y and d.month == m and d <= cutoff]
    k = len(avail)
    if k == 0:
        return None, None, 0
    ndays = (datetime(y + m // 12, m % 12 + 1, 1) - datetime(y, m, 1)).days
    if k >= ndays:
        return float(np.mean(avail)), 0.005, k  # month complete (tiny final-revision noise)
    X, Y = [], []
    for yy in range(1979, y):
        days = [era5.get(datetime(yy, m, dd).date())
                for dd in range(1, (datetime(yy + m // 12, m % 12 + 1, 1)
                                    - datetime(yy, m, 1)).days + 1)]
        if any(v is None for v in days):
            continue
        X.append(np.mean(days[:k]))
        Y.append(np.mean(days))
    X, Y = np.array(X), np.array(Y)
    b, a = np.polyfit(X, Y, 1)
    resid = Y - (a + b * X)
    return float(a + b * np.mean(avail)), float(np.std(resid, ddof=2)), k


def offset_prediction(gistemp, era5m, y, m, n_seasonal_years=10, n_drift_months=6):
    """Predict GISTEMP-ERA5 offset for (y,m) from a point-in-time GISTEMP table.

    est1: latest known offset walked forward with mean month-over-month seasonal deltas.
    est2: same month last year plus mean yearly drift of the last n_drift_months.
    Returns (mu, details) or (None, reason)."""
    offsets = {}
    for (yy, mm), g in gistemp.items():
        e = era5m.get((yy, mm))
        if e is not None:
            offsets[(yy, mm)] = g - e

    def prev_month(yy, mm):
        return (yy - 1, 12) if mm == 1 else (yy, mm - 1)

    def seasonal_delta(mm):
        """mean over recent years of offset(mm) - offset(mm-1)."""
        ds = []
        for yy in range(y - n_seasonal_years, y + 1):
            a = offsets.get((yy, mm))
            b = offsets.get(prev_month(yy, mm))
            if a is not None and b is not None:
                ds.append(a - b)
        return float(np.mean(ds)) if ds else 0.0

    known = [k for k in offsets if k < (y, m)]
    if not known:
        return None, "no offsets known"
    latest = max(known)
    # est1: walk from latest known month to target
    est1 = offsets[latest]
    yy, mm = latest
    steps = 0
    while (yy, mm) < (y, m) and steps < 6:
        yy, mm = (yy + 1, 1) if mm == 12 else (yy, mm + 1)
        est1 += seasonal_delta(mm)
        steps += 1
    # est2: same month previous year + yearly drift
    est2 = None
    if (y - 1, m) in offsets:
        drifts = []
        cy, cm = latest
        for _ in range(n_drift_months):
            if (cy, cm) in offsets and (cy - 1, cm) in offsets:
                drifts.append(offsets[(cy, cm)] - offsets[(cy - 1, cm)])
            cy, cm = prev_month(cy, cm)
        if drifts:
            est2 = offsets[(y - 1, m)] + float(np.mean(drifts))
    mu = est1 if est2 is None else (est1 + est2) / 2.0
    return mu, {"est1": est1, "est2": est2, "latest_known": latest, "steps": steps}


def nowcast(era5, era5m_final, gistemp_table, y, m, t):
    """Full nowcast of the (y,m) GISTEMP print at time t. Returns (mu, parts) or None."""
    proj, proj_sd, k = era5_projection(era5, y, m, t)
    if proj is None or gistemp_table is None:
        return None
    # era5 monthly means usable for offsets: only months fully final at t
    era5m_avail = {key: v for key, v in era5m_final.items()
                   if datetime(key[0] + key[1] // 12, key[1] % 12 + 1, 1,
                               tzinfo=timezone.utc) + timedelta(days=ERA5_LAG_DAYS) <= t}
    off, det = offset_prediction(gistemp_table, era5m_avail, y, m)
    if off is None:
        return None
    return proj + off, {"era5_proj": proj, "era5_proj_sd": proj_sd, "k": k,
                        "offset": off, "offset_detail": det}


def bucket_probs(buckets, mu, sigma):
    from math import erf

    def cdf(x):
        return 0.5 * (1 + erf((x - mu) / (sigma * math.sqrt(2))))

    ps = []
    for b in buckets:
        lo = -np.inf if b["lo"] is None else b["lo"]
        hi = np.inf if b["hi"] is None else b["hi"]
        ps.append(max(cdf(hi) - cdf(lo), 0.0) + FLOOR)
    ps = np.array(ps)
    return ps / ps.sum()


# ---------------------------------------------------------------- sigma pre-sample

def presample_sigma(era5, era5m_final, gistemp_current, first_print_noise_sd):
    """Hindcast 2015-2023 with the same pipeline (current-file GISTEMP as both the
    point-in-time table cut at each t and the target), residual sd per checkpoint."""
    resids = defaultdict(list)
    for y in range(2015, 2024):
        for m in range(1, 13):
            target = gistemp_current.get((y, m))
            if target is None:
                continue
            ckpts = checkpoint_times(y, m, closed_dt=None)
            for name, t in ckpts.items():
                # emulate the point-in-time table: months whose print existed at t
                # (print ~ day 12 of following month)
                table = {k: v for k, v in gistemp_current.items()
                         if datetime(k[0] + k[1] // 12, k[1] % 12 + 1, 12,
                                     tzinfo=timezone.utc) <= t}
                nc = nowcast(era5, era5m_final, table, y, m, t)
                if nc is None:
                    continue
                mu, _ = nc
                resids[name].append(mu - target)
    out = {}
    for name, rs in resids.items():
        rs = np.array(rs)
        out[name] = {
            "n": len(rs),
            "bias": float(np.mean(rs)),
            "sd": float(np.std(rs, ddof=1)),
            "sigma_used": float(math.sqrt(np.var(rs, ddof=1) + first_print_noise_sd ** 2)),
        }
    return out


def checkpoint_times(y, m, closed_dt):
    nxt = datetime(y + m // 12, m % 12 + 1, 1, tzinfo=timezone.utc)
    cks = {
        "day15": datetime(y, m, 15, 12, tzinfo=timezone.utc),
        "day21": datetime(y, m, 21, 12, tzinfo=timezone.utc),
        "month_end": nxt + timedelta(hours=12),
        "preprint": (closed_dt - timedelta(hours=72)) if closed_dt
                    else nxt + timedelta(days=9),
    }
    return cks


# ---------------------------------------------------------------- main

def main():
    pulldir, vintages_csv, era5_csv, gistemp_csv, outdir = sys.argv[1:6]
    os.makedirs(outdir, exist_ok=True)

    era5 = load_era5(era5_csv)
    era5m = era5_monthly(era5)
    caps = load_vintages(vintages_csv)
    gcur = load_gistemp_current(gistemp_csv)
    fprints, fp_capdates = first_prints(caps)

    monthly_slugs = [
        "how-hot-will-april-2024-be",
        "may-2024-temperature-increase-c", "june-2024-temperature-increase-c",
        "august-2024-temperature-increase-c", "september-2024-temperature-increase-c",
        "october-2024-temperature-increase-c", "november-2024-temperature-increase-c",
        "december-2024-temperature-increase-c", "january-2025-temperature-increase-c",
        "february-2025-temperature-increase-c", "march-2025-temperature-increase-c",
        "april-2025-temperature-increase-c4", "april-2025-temperature-increase-c-lower-ranges",
        "may-2025-temperature-increase-c", "june-2025-temperature-increase-c-549",
        "july-2025-temperature-increase-c-513", "july-2025-temperature-increase-c-394",
        "september-2025-temperature-increase-c", "october-2025-temperature-increase-c",
        "october-2025-temperature-increase-c-577", "november-2025-temperature-increase-c",
        "december-2025-temperature-increase-c", "january-2026-temperature-increase-c",
        "february-2026-temperature-increase-c", "march-2026-temperature-increase-c",
        "april-2026-temperature-increase-c", "may-2026-temperature-increase-c",
        "june-2026-temperature-increase-c",
    ]
    instances = [i for i in load_instances(pulldir, monthly_slugs) if i["resolved"]]
    print(f"{len(instances)} resolved instances")

    # ---- Gate 4 first: vintage integrity ------------------------------------
    print("\n=== GATE 4: GISTEMP first print vs current file ===")
    fp_noise = []
    g4_rows = []
    for (y, m) in sorted(k for k in fprints if k >= (2024, 3)):
        fp, cur = fprints[(y, m)], gcur.get((y, m))
        if cur is None:
            continue
        fp_noise.append(fp - cur)
        g4_rows.append((y, m, fp, cur, round(fp - cur, 2)))
    fp_noise_sd = float(np.std(fp_noise, ddof=1))
    fp_noise_mean = float(np.mean(fp_noise))
    print(f"first-print - current: mean {fp_noise_mean:+.3f}, sd {fp_noise_sd:.3f}, "
          f"n {len(fp_noise)}")

    # winner-bucket consistency: does the first print fall in the winning bucket?
    mismatches, flips = [], []
    for inst in instances:
        key = (inst["year"], inst["month"])
        fp, cur = fprints.get(key), gcur.get(key)
        win = next((b for b in inst["buckets"] if b["winner"]), None)
        if fp is None or win is None:
            continue

        def in_bucket(x, b):
            lo = -np.inf if b["lo"] is None else b["lo"]
            hi = np.inf if b["hi"] is None else b["hi"]
            return lo <= x < hi

        if not in_bucket(fp, win):
            mismatches.append((inst["slug"], fp, win["question"]))
        cur_bucket = next((b for b in inst["buckets"] if in_bucket(cur, b)), None)
        if cur_bucket is not None and not cur_bucket["winner"]:
            flips.append((inst["slug"], fp, cur, win["question"][-45:]))
    print(f"winner vs first-print mismatches: {len(mismatches)}")
    for s in mismatches:
        print("  MISMATCH", s)
    print(f"instances where TODAY'S value lands in a NON-winning bucket (backtest-on-"
          f"current would be wrong): {len(flips)}")
    for s in flips:
        print("  FLIP", s)

    # ---- sigma from pre-sample hindcast -------------------------------------
    print("\n=== sigma calibration (pre-sample 2015-2023 hindcast) ===")
    sigmas = presample_sigma(era5, era5m, gcur, fp_noise_sd)
    for name, s in sigmas.items():
        print(f"  {name:9s} n={s['n']:3d} bias {s['bias']:+.3f} sd {s['sd']:.3f} "
              f"sigma_used {s['sigma_used']:.3f}")

    # ---- per-instance nowcasts + market at checkpoints ----------------------
    print("\n=== per-instance checkpoint table ===")
    records = []  # one row per instance x checkpoint
    for inst in instances:
        y, m = inst["year"], inst["month"]
        cks = checkpoint_times(y, m, inst["closed_dt"])
        hists = {b["token_yes"]: load_price_history(pulldir, b["token_yes"])
                 for b in inst["buckets"]}
        win_idx = next(i for i, b in enumerate(inst["buckets"]) if b["winner"])
        for name, t in cks.items():
            if inst["closed_dt"] and t >= inst["closed_dt"]:
                continue
            table = vintage_at(caps, t)
            nc = nowcast(era5, era5m, table, y, m, t)
            if nc is None:
                continue
            mu, parts = nc
            sigma = sigmas[name]["sigma_used"]
            bias = sigmas[name]["bias"]
            probs = bucket_probs(inst["buckets"], mu - bias, sigma)
            mids, approx = [], 0
            for b in inst["buckets"]:
                p, was_fwd = mid_at(hists[b["token_yes"]], t)
                approx += was_fwd
                mids.append(p)
            if any(p is None for p in mids):
                mkt = None
            else:
                s = sum(mids)
                mkt = [p / s for p in mids] if s > 0.5 else None
            records.append({
                "slug": inst["slug"], "year": y, "month": m, "ckpt": name, "t": t,
                "mu_raw": mu, "mu": mu - bias, "sigma": sigma, "k": parts["k"],
                "model_probs": probs.tolist(), "market_probs": mkt,
                "mids": mids, "win_idx": win_idx,
                "buckets": inst["buckets"], "closed_dt": inst["closed_dt"],
                "n_fwd_mids": approx, "vig_sum": (sum(mids) if mkt else None),
                "first_print": fprints.get((y, m)),
            })

    # dump full records for reproducibility
    with open(os.path.join(outdir, "checkpoint_records.json"), "w") as f:
        json.dump([{k: (v.isoformat() if isinstance(v, datetime) else
                        [str(x) for x in v] if k == "buckets" else v)
                    for k, v in r.items() if k != "buckets"} for r in records], f, indent=1)

    # ---- GATE 1: log-loss model vs market -----------------------------------
    print("\n=== GATE 1: log-loss (winner leg), model vs de-vigged market ===")
    print(f"{'ckpt':9s} {'n':>3s} {'LL_mkt':>7s} {'LL_mod':>7s} {'diff':>7s} "
          f"{'se':>6s} {'mod_wins':>8s}")
    gate1 = {}
    for name in ["day15", "day21", "month_end", "preprint"]:
        rs = [r for r in records if r["ckpt"] == name and r["market_probs"]]
        if not rs:
            continue
        ll_m = np.array([-math.log(max(r["market_probs"][r["win_idx"]], 1e-4)) for r in rs])
        ll_o = np.array([-math.log(max(r["model_probs"][r["win_idx"]], 1e-4)) for r in rs])
        d = ll_m - ll_o  # positive = model better
        gate1[name] = {"n": len(rs), "ll_market": float(ll_m.mean()),
                       "ll_model": float(ll_o.mean()), "diff": float(d.mean()),
                       "se": float(d.std(ddof=1) / math.sqrt(len(d))),
                       "model_wins": int((d > 0).sum())}
        g = gate1[name]
        print(f"{name:9s} {g['n']:3d} {g['ll_market']:7.3f} {g['ll_model']:7.3f} "
              f"{g['diff']:+7.3f} {g['se']:6.3f} {g['model_wins']:3d}/{g['n']}")

    # ---- GATE 2: modal-bucket calibration -----------------------------------
    print("\n=== GATE 2: market modal bucket, priced vs realized ===")
    gate2 = {}
    for name in ["day21", "month_end", "preprint"]:
        rs = [r for r in records if r["ckpt"] == name and r["market_probs"]]
        if not rs:
            continue
        prices, hits, model_at_modal = [], [], []
        for r in rs:
            mi = int(np.argmax(r["market_probs"]))
            prices.append(r["market_probs"][mi])
            hits.append(1.0 if mi == r["win_idx"] else 0.0)
            model_at_modal.append(r["model_probs"][mi])
        n = len(rs)
        hit = float(np.mean(hits))
        se = math.sqrt(hit * (1 - hit) / n) if 0 < hit < 1 else 1.0 / n
        gate2[name] = {"n": n, "mean_modal_price": float(np.mean(prices)),
                       "hit_rate": hit, "hit_se": se,
                       "model_mean_at_market_modal": float(np.mean(model_at_modal))}
        g = gate2[name]
        print(f"{name:9s} n={n:3d} priced {g['mean_modal_price']:.3f} "
              f"realized {hit:.3f} +/- {se:.3f} (model would price it "
              f"{g['model_mean_at_market_modal']:.3f})")

    # ---- GATE 3: delayed-execution PnL --------------------------------------
    print("\n=== GATE 3: t+24h delayed-execution PnL, fundable zone "
          f"{FUNDABLE}, threshold {EDGE_THRESHOLD:.2f} ===")
    trades = []
    for r in records:
        if r["ckpt"] == "day15":
            continue  # market often not yet listed / first hours; day21 onwards
        if not r["market_probs"]:
            continue
        hists = {}
        for i, b in enumerate(r["buckets"]):
            model_p = r["model_probs"][i]
            mid_t = r["mids"][i]
            if mid_t is None:
                continue
            edge = model_p - mid_t
            if abs(edge) <= EDGE_THRESHOLD:
                continue
            t_fill = r["t"] + timedelta(hours=24)
            if r["closed_dt"] and t_fill > r["closed_dt"] - timedelta(hours=12):
                continue
            if b["token_yes"] not in hists:
                hists[b["token_yes"]] = load_price_history(pulldir, b["token_yes"])
            mid24, fwd = mid_at(hists[b["token_yes"]], t_fill)
            if mid24 is None:
                continue
            if edge > 0:  # buy YES
                fill = mid24 + ADVERSE_FILL
                instant = mid_t + ADVERSE_FILL
                pnl = (1.0 if b["winner"] else 0.0) - fill
                pnl_instant = (1.0 if b["winner"] else 0.0) - instant
                token_price = fill
                side = "BUY_YES"
                model_edge_at_fill = model_p - fill
            else:  # buy NO
                fill = (1 - mid24) + ADVERSE_FILL
                instant = (1 - mid_t) + ADVERSE_FILL
                pnl = (0.0 if b["winner"] else 1.0) - fill
                pnl_instant = (0.0 if b["winner"] else 1.0) - instant
                token_price = fill
                side = "BUY_NO"
                model_edge_at_fill = (1 - model_p) - fill
            if not (FUNDABLE[0] <= token_price <= FUNDABLE[1]):
                continue
            trades.append({
                "slug": r["slug"], "ckpt": r["ckpt"], "t": r["t"].isoformat(),
                "market_slug": b["market_slug"], "side": side,
                "model_p": round(model_p, 4), "mid_t": mid_t, "mid_t24": mid24,
                "fill": round(fill, 4), "pnl": round(pnl, 4),
                "pnl_instant": round(pnl_instant, 4),
                "model_edge_at_fill": round(model_edge_at_fill, 4),
                "won": b["winner"] if side == "BUY_YES" else (not b["winner"]),
            })
    with open(os.path.join(outdir, "trades.json"), "w") as f:
        json.dump(trades, f, indent=1)

    def pnl_summary(ts, label):
        if not ts:
            print(f"  {label}: no trades")
            return None
        p = np.array([t["pnl"] for t in ts])
        pi = np.array([t["pnl_instant"] for t in ts])
        print(f"  {label}: n={len(p)} delayed {100*p.mean():+.1f}c/trade "
              f"(se {100*p.std(ddof=1)/math.sqrt(len(p)):.1f}) | instant "
              f"{100*pi.mean():+.1f}c | hit {np.mean([t['won'] for t in ts]):.2f}")
        return {"n": len(p), "delayed_c": float(100 * p.mean()),
                "se_c": float(100 * p.std(ddof=1) / math.sqrt(len(p))),
                "instant_c": float(100 * pi.mean())}

    gate3 = {"all": pnl_summary(trades, "ALL")}
    for name in ["day21", "month_end", "preprint"]:
        gate3[name] = pnl_summary([t for t in trades if t["ckpt"] == name], name)
    slugs_sorted = [i["slug"] for i in instances]
    half = len(slugs_sorted) // 2
    first_half = set(slugs_sorted[:half])
    gate3["first_half"] = pnl_summary([t for t in trades if t["slug"] in first_half],
                                      "first half")
    gate3["second_half"] = pnl_summary([t for t in trades if t["slug"] not in first_half],
                                       "second half")
    for side in ["BUY_YES", "BUY_NO"]:
        gate3[side] = pnl_summary([t for t in trades if t["side"] == side], side)

    with open(os.path.join(outdir, "gates.json"), "w") as f:
        json.dump({"gate1": gate1, "gate2": gate2, "gate3": gate3,
                   "gate4": {"first_print_minus_current_mean": fp_noise_mean,
                             "first_print_minus_current_sd": fp_noise_sd,
                             "mismatches": mismatches, "current_file_flips": flips,
                             "table": g4_rows},
                   "sigmas": sigmas}, f, indent=1, default=str)
    print("\nwrote", outdir)


if __name__ == "__main__":
    main()
