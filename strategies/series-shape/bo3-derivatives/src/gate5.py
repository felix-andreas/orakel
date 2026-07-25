#!/usr/bin/env python3
"""Gate 5 — the incumbent. Fetch an external bookmaker map-handicap / total-maps line
and measure Polymarket-minus-bookmaker on the SAME leg.

KILL condition (pre-registered in ideas/2026-07-25-esports-series-shape-2.md):
  the bookmaker line agrees with Polymarket's handicap within 3pp.

Sources that work read-only from this box (see results/backtest-2026-07-25.md for the
full route inventory):
  - Pinnacle guest arcadia API (the sharp book), public guest key, no account.
  - Smarkets v3 public API (an exchange -> back/lay mid, no vig at all).

De-vig: simple normalisation p_A = (1/oA)/(1/oA + 1/oB), plus a power de-vig for
robustness (favourite-longshot-aware).  Both are reported: the kill must not depend on
which de-vig you pick.
"""
import json
import os
import subprocess
import sys
import datetime as dt

PIN_KEY = "CmX2KcMrXuFmNg6YFbmTxE0y9CIrOi0R"
UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
PIN = "https://guest.api.arcadia.pinnacle.com/0.1"


def get(url, hdrs=()):
    cmd = ["curl", "-sS", "--max-time", "60", "-H", f"User-Agent: {UA}",
           "-H", "Accept: application/json"]
    for h in hdrs:
        cmd += ["-H", h]
    p = subprocess.run(cmd + [url], capture_output=True, text=True)
    try:
        return json.loads(p.stdout)
    except Exception:
        return None


def pin(url):
    return get(PIN + url, hdrs=[f"X-API-Key: {PIN_KEY}"])


def am2dec(a):
    a = float(a)
    return 1 + (a / 100 if a > 0 else 100 / -a)


def devig_norm(oa, ob):
    ia, ib = 1 / oa, 1 / ob
    return ia / (ia + ib), ia + ib


def devig_power(oa, ob, tol=1e-10):
    """p_i = q_i^k with sum p = 1; k solved by bisection. Shrinks longshots harder than
    normalisation, so it is the CONSERVATIVE choice when testing 'is Polymarket's
    favourite leg too cheap'."""
    qa, qb = 1 / oa, 1 / ob
    lo, hi = 0.5, 3.0
    for _ in range(200):
        k = (lo + hi) / 2
        s = qa ** k + qb ** k
        if s > 1:
            lo = k
        else:
            hi = k
        if abs(s - 1) < tol:
            break
    k = (lo + hi) / 2
    return qa ** k / (qa ** k + qb ** k)


def fetch_pinnacle():
    """-> list of dicts: one per esports matchup with its ±1.5 spread + 2.5 total."""
    ms = pin("/sports/12/matchups?withSpecials=false")
    if not isinstance(ms, list):
        raise SystemExit(f"pinnacle matchups failed: {str(ms)[:200]}")
    out = []
    for m in ms:
        if m.get("parentId") or m.get("type") == "special":
            continue
        parts = {p["alignment"]: p["name"] for p in m.get("participants", [])}
        rec = {"id": m["id"], "league": (m.get("league") or {}).get("name"),
               "home": parts.get("home"), "away": parts.get("away"),
               "start": m.get("startTime"), "bestOfX": m.get("bestOfX"),
               "spread": {}, "total": {}, "ml": None, "limits": {}}
        mk = pin(f"/matchups/{m['id']}/markets/related/straight")
        if not isinstance(mk, list):
            out.append(rec)
            continue
        for k in mk:
            if k.get("period") != 0 or k.get("status") != "open":
                continue
            prices = k.get("prices") or []
            typ = k.get("type")
            if typ == "moneyline" and len(prices) == 2:
                d = {p["designation"]: am2dec(p["price"]) for p in prices}
                if "home" in d and "away" in d:
                    rec["ml"] = d
                    rec["limits"]["ml"] = k.get("limits")
            elif typ == "spread" and len(prices) == 2:
                pts = prices[0].get("points")
                d = {p["designation"]: am2dec(p["price"]) for p in prices}
                rec["spread"][f"{abs(pts)}{'-alt' if k.get('isAlternate') else ''}"] = {
                    "points_home": next((p.get("points") for p in prices
                                         if p["designation"] == "home"), None),
                    "odds": d, "alt": bool(k.get("isAlternate")), "limit": k.get("limits")}
            elif typ == "total" and len(prices) == 2:
                pts = prices[0].get("points")
                d = {p["designation"]: am2dec(p["price"]) for p in prices}
                rec["total"][str(pts)] = {"odds": d, "alt": bool(k.get("isAlternate")),
                                          "limit": k.get("limits")}
        out.append(rec)
    return out


def fetch_smarkets():
    """Exchange: back/lay mid has NO vig, so it is the cleanest possible comparator."""
    ev = get("https://api.smarkets.com/v3/events/?type=esports&state=upcoming&limit=200")
    if not ev or "events" not in ev:
        return []
    out = []
    for e in ev["events"]:
        mks = get(f"https://api.smarkets.com/v3/events/{e['id']}/markets/")
        for m in (mks or {}).get("markets", []):
            nm = (m.get("name") or "").lower()
            if "1.5" not in nm and "2.5" not in nm:
                continue
            cs = get(f"https://api.smarkets.com/v3/markets/{m['id']}/contracts/")
            q = get(f"https://api.smarkets.com/v3/markets/{m['id']}/quotes/")
            out.append({"event": e.get("name"), "start": e.get("start_datetime"),
                        "market": m.get("name"), "market_id": m["id"],
                        "contracts": [{"id": c["id"], "name": c.get("name")}
                                      for c in (cs or {}).get("contracts", [])],
                        "quotes": q})
    return out


if __name__ == "__main__":
    d = sys.argv[1] if len(sys.argv) > 1 else "."
    os.makedirs(d, exist_ok=True)
    src = sys.argv[2] if len(sys.argv) > 2 else "pinnacle"
    if src == "pinnacle":
        rows = fetch_pinnacle()
        json.dump({"fetched_at": dt.datetime.now(dt.timezone.utc).isoformat(),
                   "rows": rows}, open(os.path.join(d, "pinnacle.json"), "w"), indent=1)
        print(f"{len(rows)} matchups")
        for r in rows:
            if not r["spread"]:
                continue
            main = [k for k in r["spread"] if not r["spread"][k]["alt"]]
            print(f"{r['start']} [{r['league']}] BO{r['bestOfX']} {r['home']} vs {r['away']}"
                  f"  spreads={sorted(r['spread'])} totals={sorted(r['total'])} main={main}")
    else:
        rows = fetch_smarkets()
        json.dump({"fetched_at": dt.datetime.now(dt.timezone.utc).isoformat(),
                   "rows": rows}, open(os.path.join(d, "smarkets.json"), "w"), indent=1)
        print(f"{len(rows)} smarkets handicap/total markets")
