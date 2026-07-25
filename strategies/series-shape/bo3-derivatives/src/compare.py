#!/usr/bin/env python3
"""Match Pinnacle BO3 matchups to Polymarket events and print PM-minus-bookmaker on the
SAME leg (map handicap -1.5 and total maps O/U 2.5).  This is gate 5's evidence table.

Matching is by (start time within 2h) x (team-name token overlap), then the handicap is
re-oriented onto the SAME team as Polymarket's outcomes[0] before differencing -- an
orientation error here would fake the entire result, so both sides are printed.
"""
import json
import re
import sys
import datetime as dt

D = sys.argv[1]
SLATE = sys.argv[2] if len(sys.argv) > 2 else None

STOP = {"esports", "esport", "gaming", "team", "club", "the", "cs", "gg", "e",
        "sports", "pro", "academy"}
ALIAS = {"mouz": "mouz", "mousesports": "mouz", "tes": "topesports",
         "movistarkoi": "koi", "mkoi": "koi", "sen": "sentinels"}


def norm(s):
    s = (s or "").lower()
    s = re.sub(r"[^a-z0-9 ]", " ", s)
    toks = [t for t in s.split() if t and t not in STOP]
    return toks


def key(name):
    t = norm(name)
    return set(t) | {"".join(t)}


def score(a, b):
    ka, kb = key(a), key(b)
    if ka & kb:
        return 1.0
    # substring fallback for "G2" vs "G2 Esports", "Liquid" vs "Team Liquid"
    sa, sb = "".join(norm(a)), "".join(norm(b))
    if sa and sb and (sa in sb or sb in sa):
        return 0.9
    return 0.0


def am_norm(oa, ob):
    ia, ib = 1 / oa, 1 / ob
    return ia / (ia + ib), ia + ib


def power(oa, ob):
    qa, qb = 1 / oa, 1 / ob
    lo, hi = 0.3, 4.0
    for _ in range(300):
        k = (lo + hi) / 2
        if qa ** k + qb ** k > 1:
            lo = k
        else:
            hi = k
    k = (lo + hi) / 2
    return qa ** k / (qa ** k + qb ** k)


pin = json.load(open(f"{D}/pinnacle.json"))
slate = json.load(open(SLATE))


def parse(t):
    return dt.datetime.fromisoformat(t.replace("Z", "+00:00").replace("+00", "+00:00"))


rows = []
for pm in slate:
    pt = parse(pm["gst"])
    ml_o = pm["ml_out"]
    best, bs = None, 0
    for p in pin["rows"]:
        if not p.get("start") or not p.get("home"):
            continue
        if abs((parse(p["start"]) - pt).total_seconds()) > 7200:
            continue
        s1 = score(p["home"], ml_o[0]) + score(p["away"], ml_o[1])
        s2 = score(p["home"], ml_o[1]) + score(p["away"], ml_o[0])
        s = max(s1, s2)
        if s > bs:
            best, bs = (p, s1 >= s2), s
    if not best or bs < 1.8:
        continue
    p, same_order = best
    sp = p["spread"].get("1.5")
    to = p["total"].get("2.5")
    if not sp or p.get("bestOfX") != 3:
        continue
    # -- orient the handicap onto Polymarket's outcomes[0] (the -1.5 team)
    pm_hc_team = pm["hc_out"][0]
    ph, pa = p["home"], p["away"]
    hc_is_home = score(ph, pm_hc_team) >= score(pa, pm_hc_team)
    pts_home = sp["points_home"]
    # Pinnacle: the side with points -1.5 must win by 2+. Find which designation is -1.5.
    fav_desig = "home" if (pts_home is not None and pts_home < 0) else "away"
    o_fav = sp["odds"][fav_desig]
    o_dog = sp["odds"]["away" if fav_desig == "home" else "home"]
    pin_cover_fav, orr = am_norm(o_fav, o_dog)
    pin_cover_fav_pow = power(o_fav, o_dog)
    fav_is_home = fav_desig == "home"
    # is Pinnacle's -1.5 side the same team as Polymarket's outcomes[0]?
    aligned = (fav_is_home == hc_is_home)
    pin_hc = pin_cover_fav if aligned else None      # only comparable when aligned
    pin_hc_pow = pin_cover_fav_pow if aligned else None
    r = {"slug": pm["slug"], "start": pm["gst"], "pin_match": f"{ph} vs {pa}",
         "league": p["league"], "pm_hc_team": pm_hc_team,
         "pin_fav": ph if fav_is_home else pa, "aligned": aligned,
         "pm_hc": float(pm["hc_px"][0]), "pin_hc": pin_hc, "pin_hc_pow": pin_hc_pow,
         "hc_overround": orr, "hc_limit": (sp.get("limit") or [{}])[0].get("amount")
         if isinstance(sp.get("limit"), list) else None,
         "pm_hc_vol": pm["hc_vol"], "pm_hc_spread": pm["hc_spread"]}
    if to:
        oo, ou = to["odds"]["over"], to["odds"]["under"]
        r["pm_over"] = float(pm["t_px"][0])
        r["pin_over"], r["ou_overround"] = am_norm(oo, ou)
        r["pin_over_pow"] = power(oo, ou)
        r["pm_tot_vol"] = pm["t_vol"]
    if p.get("ml"):
        oh, oa2 = p["ml"]["home"], p["ml"]["away"]
        pmlh, mlorr = am_norm(oh, oa2)
        pm_ml0 = float(pm["ml_px"][0])
        same = score(ph, ml_o[0]) >= score(pa, ml_o[0])
        r["pm_ml"] = pm_ml0
        r["pin_ml"] = pmlh if same else 1 - pmlh
        r["pin_ml_pow"] = (power(oh, oa2) if same else 1 - power(oh, oa2))
        r["ml_overround"] = mlorr
    rows.append(r)

json.dump(rows, open(f"{D}/gate5_pairs.json", "w"), indent=1)

print(f"matched {len(rows)} Polymarket BO3 events to Pinnacle\n")
hdr = (f"{'slug':30s} {'HCteam':14s} {'PM_hc':>6} {'Pin_hc':>7} {'Pin^':>6} {'d_hc':>6} "
       f"{'PM_ov':>6} {'Pin_ov':>7} {'d_ov':>6} {'PM_ml':>6} {'Pin_ml':>7} {'d_ml':>6} {'hcVol':>8}")
print(hdr)
print("-" * len(hdr))
dh, do, dm = [], [], []
for r in sorted(rows, key=lambda x: x["start"]):
    if not r["aligned"]:
        print(f"{r['slug']:30s}  !! handicap sides not aligned "
              f"(PM {r['pm_hc_team']} / Pin fav {r['pin_fav']}) -- skipped")
        continue
    d1 = r["pm_hc"] - r["pin_hc"]
    dh.append(d1)
    d2 = r.get("pm_over", float("nan")) - r.get("pin_over", float("nan"))
    if r.get("pin_over") is not None:
        do.append(d2)
    d3 = r.get("pm_ml", float("nan")) - r.get("pin_ml", float("nan"))
    if r.get("pin_ml") is not None:
        dm.append(d3)
    print(f"{r['slug']:30s} {r['pm_hc_team'][:14]:14s} {r['pm_hc']:6.3f} {r['pin_hc']:7.3f} "
          f"{r['pin_hc_pow']:6.3f} {d1:+6.3f} {r.get('pm_over', float('nan')):6.3f} "
          f"{r.get('pin_over', float('nan')):7.3f} {d2:+6.3f} {r.get('pm_ml', float('nan')):6.3f} "
          f"{r.get('pin_ml', float('nan')):7.3f} {d3:+6.3f} {(r['pm_hc_vol'] or 0):8.0f}")


def stat(v, nm):
    if not v:
        return
    v = sorted(v)
    n = len(v)
    m = sum(v) / n
    sd = (sum((x - m) ** 2 for x in v) / (n - 1)) ** 0.5 if n > 1 else float("nan")
    med = v[n // 2]
    amed = sorted(abs(x) for x in v)[n // 2]
    print(f"  {nm}: n={n} mean={m:+.4f} (se {sd/n**0.5:.4f}) median={med:+.4f} "
          f"median|d|={amed:.4f}  within3pp={sum(abs(x)<=0.03 for x in v)}/{n}")


print("\nPolymarket minus Pinnacle (de-vig = normalisation):")
stat(dh, "map handicap -1.5")
stat(do, "total maps Over 2.5")
stat(dm, "moneyline       ")
