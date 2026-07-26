"""Head-to-head persistence CONDITIONED on Preliminary / low-vote rows.

The migrated Chinese application justifies its edge with:

    "a 3-point gap between two Preliminary rows, where empirical one-refresh
     persistence is 0.976-0.982"

That 0.98 comes from backtest-2026-07-25.md S5's top-30 pooled table. The SAME section
says Preliminary rows are the exception: sd(delta score) = 6.98 for Preliminary vs 1.19
for established, and 5.50 for <5k votes vs 0.79 for >20k. A pooled persistence figure
dominated by established rows cannot be applied to a pair of Preliminary sub-5k-vote rows
without measuring it -- that is the published-CI-vs-printed error
(wiki/reference/published-ci-vs-printed.md) in the other direction.

This measures persistence directly on the vintage archive, split by the pair's Preliminary
and vote status, for the resolving slice.
"""

import json
import math
import os
import sys
from collections import defaultdict
from datetime import datetime

SAT = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__)))), "satellites")
V = json.load(open(f"{SAT}/data/vintages.json"))
GAPS = [(0, 3, "0-2"), (3, 6, "3-5"), (6, 11, "6-10"), (11, 999, "11+")]


def dkey(m):
    try:
        return datetime.strptime(m["data_date"], "%b %d, %Y")
    except Exception:                                            # noqa: BLE001
        return None


def series(path_filter):
    """One row-table per distinct data_date, ordered in time."""
    byd = {}
    for v in V:
        if not v.get("rows") or not v.get("meta", {}).get("data_date"):
            continue
        if path_filter not in v["path"]:
            continue
        d = dkey(v["meta"])
        if d and d not in byd:
            byd[d] = v["rows"]
    return [byd[k] for k in sorted(byd)], sorted(byd)


def main():
    for pf in ["text/overall-no-style-control", "text"]:
        tabs, dates = series(pf)
        if len(tabs) < 5:
            continue
        print(f"\n=== slice '{pf}': {len(tabs)} distinct data-dates "
              f"{dates[0]:%Y-%m-%d} .. {dates[-1]:%Y-%m-%d} ===")
        # buckets: (gap band) x (pair status)
        cnt = defaultdict(lambda: [0, 0])
        sds = defaultdict(list)
        for a, b in zip(tabs, tabs[1:]):
            nb = {r["model"]: r for r in b}
            top = [r for r in a if r["rank"] <= 30]
            for r in top:
                if r["model"] in nb:
                    sds[("prelim" if r["preliminary"] else "estab")].append(
                        nb[r["model"]]["score"] - r["score"])
                    if r.get("votes") is not None:
                        sds["<5k" if r["votes"] < 5000 else ">=5k"].append(
                            nb[r["model"]]["score"] - r["score"])
            for i, x in enumerate(top):
                for y in top[i + 1:]:
                    if x["model"] not in nb or y["model"] not in nb:
                        continue
                    gap = abs(x["score"] - y["score"])
                    lead = x if x["score"] >= y["score"] else y
                    trail = y if lead is x else x
                    stays = nb[lead["model"]]["score"] >= nb[trail["model"]]["score"]
                    both_p = x["preliminary"] and y["preliminary"]
                    any_p = x["preliminary"] or y["preliminary"]
                    lowv = ((x.get("votes") or 1e9) < 5000 and (y.get("votes") or 1e9) < 5000)
                    for lo, hi, gl in GAPS:
                        if lo <= gap < hi:
                            for tag in (["all"] + (["BOTH preliminary"] if both_p else [])
                                        + (["any preliminary"] if any_p else [])
                                        + (["neither prelim"] if not any_p else [])
                                        + (["both <5k votes"] if lowv else [])):
                                cnt[(gl, tag)][0] += stays
                                cnt[(gl, tag)][1] += 1
        print("\n  sd(delta score) over one refresh, top-30:")
        for k in ["estab", "prelim", ">=5k", "<5k"]:
            v = sds.get(k)
            if v and len(v) > 2:
                mu = sum(v) / len(v)
                sd = math.sqrt(sum((z - mu) ** 2 for z in v) / (len(v) - 1))
                print(f"    {k:8s} n={len(v):5d}  sd={sd:5.2f}")
        print("\n  P(leader stays ahead) over one refresh, by score gap and pair status:")
        tags = ["all", "neither prelim", "any preliminary", "BOTH preliminary",
                "both <5k votes"]
        print(f"    {'gap':>6} " + " ".join(f"{t:>18}" for t in tags))
        for lo, hi, gl in GAPS:
            cells = []
            for t in tags:
                k, n = cnt[(gl, t)]
                cells.append(f"{k/n:.3f} (n={n})" if n >= 8 else
                             (f"  n={n}" if n else "      --"))
            print(f"    {gl:>6} " + " ".join(f"{c:>18}" for c in cells))

        if pf.endswith("no-style-control"):
            print("\n  --- implied for the LIVE Chinese board (gap 3, both Preliminary,")
            print("      both <5k votes; one refresh is due, last data_date Jul 21) ---")
            for t in ["all", "BOTH preliminary", "both <5k votes"]:
                k, n = cnt[("3-5", t)]
                if n:
                    print(f"    gap 3-5, {t:18s}: {k}/{n} = {k/n:.3f}")


if __name__ == "__main__":
    main()
