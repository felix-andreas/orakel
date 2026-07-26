"""Archive today's live resolving tables and read the Chinese board's head-to-head.

memory duty (1): no forward vintage record of the resolving slices exists unless we make
it, and a refresh can land on the check morning (backtest-2026-07-25.md S4 found exactly
one such case, and the venue used the FRESHER table). Fetch, stamp, archive.

Slices that resolve the live July cohort:
  text/overall-no-style-control   -> #1/#2/#3 nosc, Chinese
  text/overall                    -> #1/#2/#3 sc
  text/math-no-style-control      -> math #1
"""

import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone

sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))), "satellites", "src"))
import arena_parse  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
UA = ("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) "
      "Chrome/126.0 Safari/537.36")
SLICES = ["text/overall-no-style-control", "text/overall", "text/math-no-style-control",
          "text/math"]
CHINESE_ORGS = {"Alibaba", "Moonshot", "DeepSeek", "Baidu", "ByteDance", "Bytedance",
                "Zhipu", "Z.ai", "MiniMax", "Tencent", "01.AI", "StepFun", "iFlytek",
                "Xiaomi", "Ant Group", "Skywork", "InclusionAI", "Kuaishou", "Meituan"}


def fetch(path, tries=3):
    url = f"https://arena.ai/leaderboard/{path}"
    for i in range(tries):
        r = subprocess.run(["curl", "-sL", "--max-time", "90", "-A", UA, url],
                           capture_output=True, text=True)
        if r.returncode == 0 and len(r.stdout) > 5000:
            return r.stdout
        time.sleep(2 * (i + 1))
    return None


def main():
    date = sys.argv[1] if len(sys.argv) > 1 else "2026-07-26"
    d = f"{ROOT}/data/arena-{date}"
    os.makedirs(d, exist_ok=True)
    out = {}
    for path in SLICES:
        doc = fetch(path)
        if not doc:
            print(f"{path}: FETCH FAILED")
            continue
        fn = path.replace("/", "_")
        open(f"{d}/{fn}.html", "w").write(doc)
        rows, diag = arena_parse.parse_rows(doc)          # returns (rows, diagnostics)
        meta = arena_parse.parse_meta(doc)
        if not rows:
            print(f"{path}: parse produced no rows ({diag})")
            continue
        out[path] = dict(meta=meta, diag=diag, rows=rows,
                         fetched_at=datetime.now(timezone.utc).isoformat())
        print(f"\n=== {path}  data_date={meta.get('data_date')}  "
              f"({diag['n']} rows, {diag['skipped']} skipped, layout {diag['layout']}) ===")
        for r in rows[:6]:
            print(f"    rank {r['rank']:3d}  {r['org']:12s} {r['model']:30s} "
                  f"{r['score']:.0f} +/-{r['ci']} prelim={r['preliminary']} "
                  f"votes={r['votes']}")
        if "no-style-control" in path and "overall" in path:
            print("    --- Chinese-restricted ordering (the Chinese board's variable) ---")
            n = 0
            for r in rows:
                if r["org"] in CHINESE_ORGS:
                    print(f"    rank {r['rank']:3d}  {r['org']:12s} {r['model']:30s} "
                          f"{r['score']:.0f} +/-{r['ci']} prelim={r['preliminary']} "
                          f"votes={r['votes']}")
                    n += 1
                if n >= 6:
                    break
    json.dump(out, open(f"{d}/parsed.json", "w"), indent=1, default=str)
    print(f"\nwrote {d}/parsed.json")


if __name__ == "__main__":
    main()
