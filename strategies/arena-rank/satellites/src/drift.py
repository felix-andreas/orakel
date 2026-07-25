"""Empirical refresh-to-refresh drift of the arena leaderboard.

The resolving quantity is the *published score at the check instant*, not the score today.
Two things move it between now and then:

  1. score drift  — the table is recomputed over the whole vote history at every refresh,
     so every model's published score moves even with no new models;
  2. new entrants — a model that did not exist at the checkpoint can land above the
     incumbents (the "release timing is an insider process" risk).

Both are measured here from the vintage panel rather than assumed, and the published
95% CI is checked against realised drift (the idea's kill-condition-3: is Rank Spread a
usable posterior or a bootstrap artifact?).
"""

import json
import math
import sys
from collections import defaultdict
from datetime import datetime


def parse_dd(s):
    for f in ("%b %d, %Y", "%B %d, %Y"):
        try:
            return datetime.strptime(s, f)
        except (ValueError, TypeError):
            continue
    return None


def build_panel(vintages, paths, min_rows=100):
    """data_date -> {model: row}, one snapshot per distinct published vintage."""
    panel = {}
    for v in vintages:
        if v["path"] not in paths or v["diag"]["n"] < min_rows:
            continue
        d = parse_dd(v["meta"].get("data_date"))
        if not d:
            continue
        # prefer the capture with the most rows for a given data date
        if d not in panel or len(panel[d]) < v["diag"]["n"]:
            panel[d] = {r["model"]: r for r in v["rows"]}
    return dict(sorted(panel.items()))


def drift_pairs(panel, max_gap_days=45):
    """(gap_days, model, s0, s1, ci0, votes0, prelim0, rank0, rank1) for consecutive-ish pairs."""
    dates = sorted(panel)
    out = []
    for i, d0 in enumerate(dates):
        for d1 in dates[i + 1:]:
            gap = (d1 - d0).days
            if gap <= 0 or gap > max_gap_days:
                continue
            a, b = panel[d0], panel[d1]
            for m, r0 in a.items():
                r1 = b.get(m)
                if not r1:
                    continue
                out.append(
                    dict(
                        gap=gap,
                        model=m,
                        d0=d0.strftime("%Y-%m-%d"),
                        d1=d1.strftime("%Y-%m-%d"),
                        s0=r0["score"],
                        s1=r1["score"],
                        ds=r1["score"] - r0["score"],
                        ci0=r0.get("ci"),
                        votes0=r0.get("votes"),
                        prelim0=r0.get("preliminary"),
                        rank0=r0["rank"],
                        rank1=r1["rank"],
                        spread_lo=r0.get("spread_lo"),
                        spread_hi=r0.get("spread_hi"),
                    )
                )
            break  # only the next available vintage, to keep pairs near-independent
    return out


def entrants(panel, top_n=10):
    """Per consecutive vintage pair: models newly present, and how high they landed."""
    dates = sorted(panel)
    out = []
    for d0, d1 in zip(dates, dates[1:]):
        gap = (d1 - d0).days
        if gap <= 0 or gap > 45:
            continue
        new = [r for m, r in panel[d1].items() if m not in panel[d0]]
        best_old = min((r["score"] for r in panel[d0].values()), default=None)
        top_old = sorted(panel[d0].values(), key=lambda r: r["rank"])[:top_n]
        cut = top_old[-1]["score"] if top_old else None
        out.append(
            dict(
                d0=d0.strftime("%Y-%m-%d"),
                d1=d1.strftime("%Y-%m-%d"),
                gap=gap,
                n_new=len(new),
                n_new_topN=sum(1 for r in new if r["rank"] <= top_n),
                new_top=[
                    dict(model=r["model"], org=r["org"], rank=r["rank"], score=r["score"])
                    for r in sorted(new, key=lambda z: z["rank"])[:5]
                    if r["rank"] <= top_n * 2
                ],
                topN_cut=cut,
                _unused=best_old,
            )
        )
    return out


def sd(xs):
    xs = [x for x in xs if x is not None]
    if len(xs) < 2:
        return None
    m = sum(xs) / len(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


def summarize(pairs):
    """Drift sd by horizon bucket and by vote count, plus CI coverage."""
    buckets = defaultdict(list)
    for p in pairs:
        h = p["gap"]
        b = "1-7d" if h <= 7 else "8-14d" if h <= 14 else "15-30d" if h <= 30 else "31-45d"
        buckets[b].append(p)
    rows = []
    for b, ps in buckets.items():
        top = [p for p in ps if p["rank0"] <= 25]
        rows.append(
            dict(
                horizon=b,
                n=len(ps),
                sd_all=sd([p["ds"] for p in ps]),
                n_top25=len(top),
                sd_top25=sd([p["ds"] for p in top]),
                mean_ci_top25=(
                    sum(p["ci0"] for p in top if p["ci0"]) / max(1, sum(1 for p in top if p["ci0"]))
                ),
            )
        )
    return sorted(rows, key=lambda r: r["n"], reverse=True)


if __name__ == "__main__":
    V = json.load(open(sys.argv[1]))
    for name, paths in [
        ("nosc", {"text/overall-no-style-control"}),
        ("sc", {"text/overall", "text"}),
    ]:
        panel = build_panel(V, paths)
        pairs = drift_pairs(panel)
        print(f"\n=== {name}: {len(panel)} vintages, {len(pairs)} model-pairs ===")
        for r in summarize(pairs):
            print(
                f"  {r['horizon']:7s} n={r['n']:5d} sd(dScore)={r['sd_all']:.2f}"
                f"   top25: n={r['n_top25']:4d} sd={r['sd_top25'] or float('nan'):.2f}"
                f"  mean published CI={r['mean_ci_top25']:.2f}"
            )
