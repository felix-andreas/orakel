"""Company-level leadership persistence on the Chinese sub-ranking -- the one board that
is inside the fundable band, measured on its OWN resolution variable.

Everything upstream measures pairs of MODELS. The board resolves on which COMPANY owns the
highest-ranked Chinese model, which is a max over each company's fleet -- Alibaba's second
model is a backstop the head-to-head number ignores. So measure the thing itself: over the
vintage archive of the resolving slice, if company X leads the Chinese subset on data-date
t, does X still lead k days later?

This is the naive-persistence NULL for this board. If it comes out near the market's 0.80,
the crowd is right and there is nothing to sharpen; if it comes out near 0.93, the
sharpening rule is corroborated by something other than itself.
"""

import json
import math
import os
from collections import defaultdict
from datetime import datetime

SAT = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))), "satellites")
V = json.load(open(f"{SAT}/data/vintages.json"))

# Chinese-headquartered orgs as they appear in the Organization column.
CN = {"Alibaba", "Moonshot", "DeepSeek", "Baidu", "ByteDance", "Bytedance", "Zhipu",
      "Z.ai", "MiniMax", "Tencent", "01.AI", "StepFun", "iFlytek", "Xiaomi", "Ant Group",
      "Skywork", "InclusionAI", "Kuaishou", "Alibaba-Qwen", "Qwen", "Baichuan"}


def leader(rows):
    """The company owning the best-ranked Chinese model (rank ascending == better)."""
    best = None
    for r in rows:
        if r["org"] not in CN:
            continue
        key = (r["rank"], -r["score"])
        if best is None or key < best[0]:
            best = (key, r)
    return (best[1]["org"], best[1]) if best else (None, None)


def main():
    byd = {}
    for v in V:
        if not v.get("rows") or "no-style-control" not in v["path"]:
            continue
        try:
            d = datetime.strptime(v["meta"]["data_date"], "%b %d, %Y")
        except Exception:                                        # noqa: BLE001
            continue
        byd.setdefault(d, v["rows"])
    ds = sorted(byd)
    print(f"resolving slice, {len(ds)} distinct data-dates {ds[0]:%Y-%m-%d} .. {ds[-1]:%Y-%m-%d}")
    med = sorted((ds[i + 1] - ds[i]).days for i in range(len(ds) - 1))
    print(f"refresh cadence: median {med[len(med)//2]}d, p90 {med[int(.9*len(med))]}d, "
          f"max {med[-1]}d  -> a Jul-21 table before a Jul-31 check sees "
          f"{'1-2' if med[len(med)//2] <= 7 else '0-1'} more refreshes\n")

    print("Chinese leader by data-date (the board's resolution variable):")
    for d in ds:
        org, r = leader(byd[d])
        if org:
            nxt = [x for x in byd[d] if x["org"] in CN and x["org"] != org]
            nxt = min(nxt, key=lambda z: z["rank"]) if nxt else None
            print(f"  {d:%Y-%m-%d}  {org:10s} {r['model']:26s} rank {r['rank']:3d} "
                  f"score {r['score']:.0f} prelim={str(r['preliminary']):5s} "
                  f"votes={r['votes']}" +
                  (f"   | next: {nxt['org']} {nxt['score']:.0f} (gap {r['score']-nxt['score']:.0f})"
                   if nxt else ""))

    print("\nP(same company still leads) by horizon, all consecutive vintage pairs:")
    for lo, hi, lab in [(1, 8, "1-7 days"), (8, 15, "8-14 days"), (1, 15, "1-14 days"),
                        (5, 12, "5-11 days (our 5.2d hold + refresh slack)")]:
        k = n = 0
        for i, d0 in enumerate(ds):
            o0, _ = leader(byd[d0])
            if not o0:
                continue
            for d1 in ds[i + 1:]:
                dd = (d1 - d0).days
                if dd >= hi:
                    break
                if dd < lo:
                    continue
                o1, _ = leader(byd[d1])
                if o1:
                    k += (o1 == o0)
                    n += 1
        if n:
            se = math.sqrt(k / n * (1 - k / n) / n)
            print(f"  {lab:42s} {k:3d}/{n:3d} = {k/n:.3f} (se {se:.3f})")

    print("\nConditioned on the leader's margin over the next company, 1-14 day horizon:")
    for glo, ghi, lab in [(0, 4, "gap 0-3 pts"), (4, 8, "gap 4-7"), (8, 999, "gap 8+")]:
        k = n = 0
        for i, d0 in enumerate(ds):
            o0, r0 = leader(byd[d0])
            if not o0:
                continue
            oth = [x for x in byd[d0] if x["org"] in CN and x["org"] != o0]
            if not oth:
                continue
            gap = r0["score"] - min(oth, key=lambda z: z["rank"])["score"]
            if not (glo <= gap < ghi):
                continue
            for d1 in ds[i + 1:]:
                dd = (d1 - d0).days
                if dd >= 15:
                    break
                o1, _ = leader(byd[d1])
                if o1:
                    k += (o1 == o0)
                    n += 1
        if n:
            print(f"  {lab:20s} {k:3d}/{n:3d} = {k/n:.3f}"
                  + ("   <-- TODAY's state (Alibaba 1476 vs Moonshot 1473, gap 3)"
                     if glo == 0 else ""))

    print("\nAnd the direct evidence: the 3 resolved Chinese board instances")
    print("  2026-04  favourite Alibaba 0.838 at T-7d -> resolved BAIDU   (loss)")
    print("  2026-05  favourite Alibaba 0.881 at T-7d -> resolved Alibaba (win)")
    print("  2026-06  favourite Alibaba 0.899 at T-7d -> resolved Alibaba (win)")


if __name__ == "__main__":
    main()
