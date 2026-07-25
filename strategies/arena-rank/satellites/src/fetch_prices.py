"""CLOB price history for every leg of every arena board we can score.

Gotchas (wiki/recipes/polymarket-api.md): interval=max silently caps at ~30d; for the full
series pass startTs alone with fidelity, and adding endTs 400s. Empty history happens —
retried without fidelity before giving up.
"""

import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone

UA = "orakel-research/1.0"
BASE = "https://clob.polymarket.com/prices-history"


def curl_json(url, tries=3):
    for i in range(tries):
        r = subprocess.run(
            ["curl", "-s", "--max-time", "90", "-A", UA, url], capture_output=True, text=True
        )
        if r.returncode == 0 and r.stdout.strip():
            try:
                return json.loads(r.stdout)
            except json.JSONDecodeError:
                pass
        time.sleep(1.5 * (i + 1))
    return None


def history(token_id, start_ts):
    for url in (
        f"{BASE}?market={token_id}&startTs={start_ts}&fidelity=60",
        f"{BASE}?market={token_id}&startTs={start_ts}",
        f"{BASE}?market={token_id}&interval=max&fidelity=60",
    ):
        d = curl_json(url)
        if d and d.get("history"):
            return d["history"]
    return []


def main(root, out_dir):
    boards = json.load(open(f"{root}/poly/boards.json"))
    os.makedirs(out_dir, exist_ok=True)
    jobs = []
    for b in boards:
        if not (b["board_type"] and b["check_et"]):
            continue
        start = b.get("start") or b.get("legs", [{}])[0].get("start")
        try:
            st = int(
                datetime.fromisoformat(start.replace("Z", "+00:00")).timestamp()
            ) - 86400
        except Exception:
            st = int(datetime(2025, 1, 1, tzinfo=timezone.utc).timestamp())
        for l in b["legs"]:
            if not l["token_id"]:
                continue
            jobs.append((b["slug"], l["company"], l["token_id"], st))

    print(f"{len(jobs)} legs to fetch")

    def job(j):
        slug, comp, tok, st = j
        p = f"{out_dir}/{tok}.json"
        if os.path.exists(p) and os.path.getsize(p) > 40:
            return 1
        h = history(tok, st)
        json.dump(dict(slug=slug, company=comp, token_id=tok, history=h), open(p, "w"))
        return 1 if h else 0

    ok = 0
    with ThreadPoolExecutor(max_workers=8) as ex:
        for i, r in enumerate(ex.map(job, jobs)):
            ok += r
            if (i + 1) % 100 == 0:
                print(f"  {i+1}/{len(jobs)} ok={ok}")
    print(f"done: {ok}/{len(jobs)} legs with history -> {out_dir}")


if __name__ == "__main__":
    main("strategies/arena-rank/satellites/data", sys.argv[1])
