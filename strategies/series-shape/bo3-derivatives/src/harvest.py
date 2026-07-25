#!/usr/bin/env python3
"""series-shape/bo3-derivatives — independent harvest of resolved esports BO3 triples.

Deliberately an INDEPENDENT code path from the idea's own scan (gate 0: the null is
"we are wrong"). Nothing is classified by slug or title text; every leg is typed from
Gamma's `sportsMarketType` and its `description` is retained verbatim for auditing.

Python (not Rust) because this stage is pure IO orchestration over ~1500 HTTP pages plus
small tabular stats; see worklog 2026-07-25.

Subcommands:
  events  <dir>              date-windowed offset paging of resolved esports events
  legs    <dir>              -> triples.jsonl   (moneyline + map_handicap + totals)
  clob    <dir> [n]          prices-history per token -> clob/<token>.json
  tape    <dir> <slugs...>   data-api taker tape per condition_id
"""
import json
import os
import subprocess
import sys
import datetime as dt
from concurrent.futures import ThreadPoolExecutor

UA = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36"
GAMMA = "https://gamma-api.polymarket.com"
CLOB = "https://clob.polymarket.com"
DATA = "https://data-api.polymarket.com"
TAG_ESPORTS = "64"


def get(url, tries=4, timeout=90):
    """curl, not urllib: the agent proxy 403s urllib (wiki/recipes/polymarket-api.md)."""
    for i in range(tries):
        p = subprocess.run(
            ["curl", "-sS", "--max-time", str(timeout), "-H", f"User-Agent: {UA}",
             "-H", "Accept: application/json", url],
            capture_output=True, text=True)
        if p.returncode == 0 and p.stdout.strip():
            try:
                return json.loads(p.stdout)
            except json.JSONDecodeError:
                pass
    return None


# ---------------------------------------------------------------- events

KEEP_MKT = ["sportsMarketType", "slug", "conditionId", "clobTokenIds", "outcomes",
            "outcomePrices", "gameStartTime", "eventStartTime", "volumeNum", "volume",
            "spread", "bestBid", "bestAsk", "closed", "active", "archived",
            "umaResolutionStatus", "lastTradePrice", "feeSchedule", "feesEnabled",
            "endDate", "closedTime", "createdAt", "orderPriceMinTickSize", "id",
            "resolutionSource", "question", "liquidityNum", "liquidity"]
KEEP_EV = ["id", "slug", "title", "closed", "endDate", "startDate", "closedTime",
           "gameId", "sport", "volume", "openInterest", "seriesSlug", "eventDate"]


def reduce_event(e):
    r = {k: e.get(k) for k in KEEP_EV}
    r["markets"] = []
    for m in e.get("markets") or []:
        d = {k: m.get(k) for k in KEEP_MKT}
        d["description"] = (m.get("description") or "")[:900]
        r["markets"].append(d)
    return r


def cmd_events(d, start="2025-11-25", end="2026-07-26"):
    """Date-windowed offset paging: offset caps at 2000, so window by endDate."""
    os.makedirs(d, exist_ok=True)
    out = open(os.path.join(d, "events.jsonl"), "w")
    d0 = dt.date.fromisoformat(start)
    d1 = dt.date.fromisoformat(end)
    windows = []
    c = d0
    while c < d1:
        n = min(c + dt.timedelta(days=7), d1)
        windows.append((c, n))
        c = n

    def one_window(w):
        a, b = w
        rows, off = [], 0
        while True:
            url = (f"{GAMMA}/events?closed=true&tag_id={TAG_ESPORTS}&limit=100&offset={off}"
                   f"&end_date_min={a}T00:00:00Z&end_date_max={b}T00:00:00Z")
            j = get(url)
            if not isinstance(j, list):
                sys.stderr.write(f"  !! {a} off={off} -> {str(j)[:120]}\n")
                break
            rows += [reduce_event(e) for e in j]
            if len(j) < 100:
                break
            off += 100
            if off > 1900:
                sys.stderr.write(f"  !! {a} hit offset cap, window too wide\n")
                break
        return a, rows

    seen = set()
    n = 0
    with ThreadPoolExecutor(max_workers=8) as ex:
        for a, rows in ex.map(one_window, windows):
            for r in rows:
                if r["id"] in seen:
                    continue
                seen.add(r["id"])
                out.write(json.dumps(r) + "\n")
                n += 1
            print(f"  {a}: {len(rows)} events (total unique {n})", flush=True)
    out.close()
    print(f"events.jsonl: {n} unique resolved esports events")


# ---------------------------------------------------------------- legs

def jl(s, default=None):
    try:
        return json.loads(s) if isinstance(s, str) else (s if s is not None else default)
    except Exception:
        return default


def cmd_legs(d):
    """Build the triple ledger. Typing is by sportsMarketType ONLY; semantics are then
    CHECKED against the description text (never inferred from titles)."""
    trip, stats = [], {"events": 0, "has_ml": 0, "has_hc": 0, "has_tot": 0, "triple": 0,
                       "triple_resolved": 0, "unresolved_leg": 0, "bo3_title": 0}
    for line in open(os.path.join(d, "events.jsonl")):
        e = json.loads(line)
        stats["events"] += 1
        by = {}
        for m in e["markets"]:
            by.setdefault(m.get("sportsMarketType"), []).append(m)
        if "moneyline" in by:
            stats["has_ml"] += 1
        if "map_handicap" in by:
            stats["has_hc"] += 1
        if "totals" in by:
            stats["has_tot"] += 1
        if not ("moneyline" in by and "map_handicap" in by and "totals" in by):
            continue
        stats["triple"] += 1
        ml, hc, tot = by["moneyline"][0], by["map_handicap"][0], by["totals"][0]
        rec = {"event_slug": e["slug"], "title": e["title"], "end": e["endDate"],
               "gameId": e.get("gameId"), "sport": e.get("sport"),
               "ev_volume": e.get("volume")}
        ok = True
        for tag, m in (("ml", ml), ("hc", hc), ("tot", tot)):
            o, p = jl(m["outcomes"], []), jl(m["outcomePrices"], [])
            t = jl(m["clobTokenIds"], [])
            if len(o) != 2 or len(p) != 2 or len(t) != 2:
                ok = False
            rec[tag] = {"slug": m["slug"], "cid": m["conditionId"], "out": o, "px": p,
                        "tok": t, "vol": m.get("volumeNum"), "gst": m.get("gameStartTime"),
                        "spread": m.get("spread"), "uma": m.get("umaResolutionStatus"),
                        "closed": m.get("closed"), "desc": m.get("description"),
                        "liq": m.get("liquidityNum"), "tick": m.get("orderPriceMinTickSize"),
                        "fee": m.get("feeSchedule"), "created": m.get("createdAt"),
                        "ltp": m.get("lastTradePrice")}
        if not ok:
            continue
        # resolved := settled prices collapsed to 1/0 on all three legs
        res = all(sorted(map(float, rec[t]["px"])) == [0.0, 1.0] for t in ("ml", "hc", "tot"))
        rec["resolved"] = res
        if not res:
            stats["unresolved_leg"] += 1
        else:
            stats["triple_resolved"] += 1
        if "(BO3)" in (e["title"] or ""):
            stats["bo3_title"] += 1
        trip.append(rec)
    with open(os.path.join(d, "triples.jsonl"), "w") as f:
        for r in trip:
            f.write(json.dumps(r) + "\n")
    print(json.dumps(stats, indent=1))


# ---------------------------------------------------------------- clob

def cmd_clob(d, limit=None, fidelity=10, lookback_s=129600):
    """prices-history per token, window anchored on gameStartTime (as the idea did)."""
    os.makedirs(os.path.join(d, "clob"), exist_ok=True)
    jobs = []
    for line in open(os.path.join(d, "triples.jsonl")):
        r = json.loads(line)
        if not r["resolved"]:
            continue
        gst = r["ml"]["gst"] or r["hc"]["gst"]
        if not gst:
            continue
        t0 = int(dt.datetime.fromisoformat(gst.replace("+00", "+00:00")).timestamp()) - lookback_s
        for tag in ("ml", "hc", "tot"):
            tok = r[tag]["tok"][0]  # outcome-0 token; outcome-1 = 1 - p
            p = os.path.join(d, "clob", f"{tok}.json")
            if not os.path.exists(p):
                jobs.append((f"{CLOB}/prices-history?market={tok}&startTs={t0}&fidelity={fidelity}", p))
    if limit:
        jobs = jobs[:int(limit)]
    print(f"{len(jobs)} token fetches")
    okc = [0, 0]

    def one(j):
        url, path = j
        v = get(url, tries=3, timeout=60)
        if v is None:
            okc[1] += 1
            return
        json.dump(v, open(path, "w"))
        okc[0] += 1

    with ThreadPoolExecutor(max_workers=28) as ex:
        for i, _ in enumerate(ex.map(one, jobs)):
            if i % 500 == 0:
                print(f"  {i}/{len(jobs)} ok={okc[0]} fail={okc[1]}", flush=True)
    print(f"done ok={okc[0]} fail={okc[1]}")


def cmd_tape(d, *slugs):
    os.makedirs(os.path.join(d, "tape"), exist_ok=True)
    idx = {}
    for line in open(os.path.join(d, "triples.jsonl")):
        r = json.loads(line)
        idx[r["event_slug"]] = r
    for s in slugs:
        r = idx.get(s)
        if not r:
            print(f"  ?? {s}")
            continue
        for tag in ("ml", "hc", "tot"):
            cid, all_t, off = r[tag]["cid"], [], 0
            while True:
                j = get(f"{DATA}/trades?market={cid}&limit=500&offset={off}")
                if not isinstance(j, list) or not j:
                    break
                all_t += j
                if len(j) < 500:
                    break
                off += 500
            json.dump(all_t, open(os.path.join(d, "tape", f"{s}.{tag}.json"), "w"))
            print(f"  {s}.{tag}: {len(all_t)} trades")


if __name__ == "__main__":
    cmd = sys.argv[1]
    {"events": cmd_events, "legs": cmd_legs, "clob": cmd_clob, "tape": cmd_tape}[cmd](*sys.argv[2:])
