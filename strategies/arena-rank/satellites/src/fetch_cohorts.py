"""Discover every Polymarket arena-leaderboard board (open + closed) and freeze the raw JSON.

Gamma /public-search over the family's title words, then /events?slug= for each hit
(&closed=true is implied by the events-by-slug endpoint returning resolved events too;
/markets?slug= needs it explicitly — see wiki/recipes/polymarket-api.md).

Output: <out>/events/<slug>.json  and  <out>/index.json
"""

import json
import os
import subprocess
import sys
import time

QUERIES = [
    "best AI model",
    "best AI company",
    "arena rank",
    "chatbot arena",
    "chinese AI company",
    "webdev arena",
    "math arena",
    "coding arena",
    "best code model",
    "style control",
    "second best AI model",
    "third best AI model",
]

UA = "orakel-research/1.0"


def curl(url, tries=3):
    for i in range(tries):
        r = subprocess.run(
            ["curl", "-s", "--max-time", "90", "-A", UA, url],
            capture_output=True,
            text=True,
        )
        if r.returncode == 0 and r.stdout.strip():
            try:
                return json.loads(r.stdout)
            except json.JSONDecodeError:
                pass
        time.sleep(1.5 * (i + 1))
    return None


def main(out):
    os.makedirs(f"{out}/events", exist_ok=True)
    seen = {}
    for q in QUERIES:
        d = curl(
            "https://gamma-api.polymarket.com/public-search?q="
            + q.replace(" ", "%20")
            + "&limit_per_type=40"
        )
        if not d:
            print(f"  ! search failed: {q}", file=sys.stderr)
            continue
        for e in d.get("events", []):
            seen.setdefault(e["slug"], e)
        print(f"  {q}: {len(d.get('events', []))} events (total {len(seen)})")

    # keep only events whose description points at the arena leaderboard
    kept = []
    for slug in sorted(seen):
        path = f"{out}/events/{slug}.json"
        if os.path.exists(path):
            ev = json.load(open(path))
        else:
            ev = curl(f"https://gamma-api.polymarket.com/events?slug={slug}")
            if not ev:
                continue
            json.dump(ev, open(path, "w"))
        if not ev:
            continue
        e = ev[0]
        desc = (e.get("description") or "").lower()
        if "arena" not in desc or "leaderboard" not in desc:
            os.remove(path)
            continue
        kept.append(
            dict(
                slug=slug,
                title=e.get("title"),
                closed=e.get("closed"),
                volume=e.get("volume"),
                liquidity=e.get("liquidity"),
                startDate=e.get("startDate"),
                endDate=e.get("endDate"),
                n_markets=len(e.get("markets", [])),
            )
        )
        print(f"  KEEP {slug} closed={e.get('closed')} vol={e.get('volume')}")
    json.dump(kept, open(f"{out}/index.json", "w"), indent=1)
    print(f"\n{len(kept)} arena events frozen -> {out}")


if __name__ == "__main__":
    main(sys.argv[1])
