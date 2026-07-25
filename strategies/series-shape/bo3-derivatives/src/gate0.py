#!/usr/bin/env python3
"""Gate 0 — artifact hunt. Price-free checks that need only settled outcomes.

The null hypothesis is "we are wrong". Every check below is designed to FAIL loudly if
a leg is mislabelled, if the ledger is survivorship-filtered, or if the three books do
not describe the objects we think they do.

  A. description-derived semantics vs sportsMarketType   (no titles anywhere)
  B. the three-leg identity  HC_cover(X) <=> X wins match AND Under 2.5
  C. survivorship: what happens to the series that DON'T settle 1/0
  D. realised joint distribution + per-game / per-month supply
"""
import json
import os
import re
import sys
from collections import Counter, defaultdict

D = sys.argv[1]
ONLY_BO3 = "--all" not in sys.argv


def load():
    for line in open(os.path.join(D, "triples.jsonl")):
        r = json.loads(line)
        if ONLY_BO3 and not is_bo3(r):
            continue
        yield r


def winner(leg):
    """Settled winner outcome NAME, or None if not a clean 1/0 settle."""
    px = [float(x) for x in leg["px"]]
    if sorted(px) != [0.0, 1.0]:
        return None
    return leg["out"][0] if px[0] == 1.0 else leg["out"][1]


# ---------------------------------------------------------------- A. semantics
# Handicap description template (verified verbatim on live + resolved markets):
#   'This market will resolve to "<A>" if <A> wins 2 or more (games|maps) than <B> ...'
# Totals:
#   'This market will resolve to "Over" if <A> and <B> play 3 or more (games|maps) ...'
RE_HC = re.compile(r'resolve to "([^"]+)" if \1 wins (\d+) or more (?:games|maps) than ([^ ]+(?: [^ ]+)*?) in this match', re.S)
RE_HC_LOOSE = re.compile(r'resolve to "([^"]+)" if .{0,80}?wins (\d+) or more (?:games|maps)', re.S)
RE_TOT = re.compile(r'resolve to "Over" if .{0,120}? play (\d+) or more (?:games|maps)', re.S)
RE_ML = re.compile(r'resolve to "([^"]+)" if \1 win the match', re.S)


def hc_margin(r):
    m = RE_HC.search(r["hc"]["desc"] or "") or RE_HC_LOOSE.search(r["hc"]["desc"] or "")
    return int(m.group(2)) if m else None


def tot_thresh(r):
    m = RE_TOT.search(r["tot"]["desc"] or "")
    return int(m.group(1)) if m else None


def is_bo3(r):
    """BO3 is defined by the LEGS, not by the title: handicap margin 2 (= -1.5 maps)
    AND totals threshold 3 (= O/U 2.5).  In a BO5 the handicap is ALSO 'wins 2 or more'
    but totals is 'plays 4 or more', so the two-leg pair is what pins the format down.
    Getting this wrong silently imports 1,597 BO5/BO7 series whose HC/totals identity is
    different -- the single largest artifact found on day 1."""
    return hc_margin(r) == 2 and tot_thresh(r) == 3


def check_semantics(rows):
    st = Counter()
    bad = []
    for r in rows:
        hc, tot, ml = r["hc"], r["tot"], r["ml"]
        # --- handicap: outcomes[0] must be the team named as the -N.5 side
        m = RE_HC.search(hc["desc"] or "") or RE_HC_LOOSE.search(hc["desc"] or "")
        if not m:
            st["hc_desc_unparsed"] += 1
            bad.append(("hc_unparsed", r["event_slug"]))
        else:
            named, n = m.group(1), int(m.group(2))
            st[f"hc_margin_{n}"] += 1
            if named == hc["out"][0]:
                st["hc_out0_is_handicap_side"] += 1
            else:
                st["hc_OUT0_MISMATCH"] += 1
                bad.append(("hc_mismatch", r["event_slug"], named, hc["out"]))
        # --- totals: Over must be outcomes[0] and threshold must be 3+
        mt = RE_TOT.search(tot["desc"] or "")
        if not mt:
            st["tot_desc_unparsed"] += 1
        else:
            st[f"tot_thresh_{int(mt.group(1))}"] += 1
        if tot["out"][0] == "Over":
            st["tot_out0_is_over"] += 1
        else:
            st["tot_OUT0_NOT_OVER"] += 1
            bad.append(("tot_order", r["event_slug"], tot["out"]))
        # --- moneyline: outcomes[0] wins iff named
        mm = RE_ML.search(ml["desc"] or "")
        st["ml_desc_ok" if mm and mm.group(1) == ml["out"][0] else "ml_desc_odd"] += 1
        # --- the handicap side must be one of the two moneyline teams
        if hc["out"][0] in ml["out"] and hc["out"][1] in ml["out"]:
            st["hc_teams_match_ml"] += 1
        else:
            st["HC_TEAMS_DIFFER_FROM_ML"] += 1
            bad.append(("teams", r["event_slug"], hc["out"], ml["out"]))
    return st, bad


# ---------------------------------------------------------------- B. identity
def check_identity(rows):
    st = Counter()
    viol = []
    dist = Counter()
    for r in rows:
        if not r["resolved"]:
            continue
        w_ml, w_hc, w_tot = winner(r["ml"]), winner(r["hc"]), winner(r["tot"])
        if None in (w_ml, w_hc, w_tot):
            st["skip_unsettled"] += 1
            continue
        hc_side = r["hc"]["out"][0]              # the -1.5 team
        cover = (w_hc == hc_side)                # handicap leg paid the -1.5 side
        hc_side_won_match = (w_ml == hc_side)
        under = (w_tot == "Under")
        st["n"] += 1
        if cover == (hc_side_won_match and under):
            st["identity_ok"] += 1
        else:
            st["IDENTITY_VIOLATION"] += 1
            viol.append((r["event_slug"], r["title"], f"cover={cover}",
                         f"ml_winner={w_ml}", f"hc_side={hc_side}", f"tot={w_tot}"))
        # realised series score, from the -1.5 side's perspective
        other = r["hc"]["out"][1]
        if hc_side_won_match:
            dist["hcside 2-0" if under else "hcside 2-1"] += 1
        else:
            dist["other 2-0" if under else "other 2-1"] += 1
    return st, viol, dist


# ---------------------------------------------------------------- C. survivorship
def check_survivorship(rows):
    st = Counter()
    ex = defaultdict(list)
    for r in rows:
        pxs = {t: [float(x) for x in r[t]["px"]] for t in ("ml", "hc", "tot")}
        clean = {t: sorted(pxs[t]) == [0.0, 1.0] for t in pxs}
        if all(clean.values()):
            st["all_clean"] += 1
            continue
        # classify the non-clean settle
        for t in ("ml", "hc", "tot"):
            if clean[t]:
                continue
            p = pxs[t]
            if p == [0.5, 0.5]:
                st[f"{t}_50_50"] += 1
                ex[f"{t}_50_50"].append(r["event_slug"])
            elif r[t].get("uma") not in ("resolved",):
                st[f"{t}_uma_{r[t].get('uma')}"] += 1
                ex[f"{t}_uma"].append(r["event_slug"])
            else:
                st[f"{t}_other_{p}"] += 1
                ex[f"{t}_other"].append(r["event_slug"])
        st["any_dirty"] += 1
    return st, ex


# ---------------------------------------------------------------- D. supply
def supply(rows):
    g, mo, bo = Counter(), Counter(), Counter()
    for r in rows:
        if not r["resolved"]:
            continue
        t = r["title"] or ""
        g[(t.split(":")[0] if ":" in t else "?").strip()] += 1
        mo[(r["end"] or "")[:7]] += 1
        m = re.search(r"\((BO\d)\)", t)
        bo[m.group(1) if m else "none"] += 1
    return g, mo, bo


if __name__ == "__main__":
    rows = list(load())
    print(f"# triples: {len(rows)}  resolved: {sum(r['resolved'] for r in rows)}\n")

    print("== A. leg semantics parsed from descriptions ==")
    st, bad = check_semantics(rows)
    for k, v in sorted(st.items()):
        print(f"  {k:34s} {v}")
    for b in bad[:15]:
        print("   !!", b)

    print("\n== B. three-leg identity  HC_cover(X) <=> X wins match AND Under 2.5 ==")
    st, viol, dist = check_identity(rows)
    for k, v in sorted(st.items()):
        print(f"  {k:34s} {v}")
    if st["n"]:
        print(f"  identity rate: {st['identity_ok']/st['n']:.5f}")
    print("  violations:")
    for v in viol[:40]:
        print("   !!", v)
    tot = sum(dist.values())
    print("  realised series-score distribution (-1.5 side = 'hcside'):")
    for k, v in sorted(dist.items()):
        print(f"    {k:12s} {v:6d}  {v/tot:.4f}")

    print("\n== C. survivorship: series that do NOT settle 1/0 ==")
    st, ex = check_survivorship(rows)
    for k, v in sorted(st.items()):
        print(f"  {k:34s} {v}")
    for k, v in ex.items():
        print(f"   e.g. {k}: {v[:6]}")

    print("\n== D. supply ==")
    g, mo, bo = supply(rows)
    print("  by game:  ", dict(g.most_common()))
    print("  by month: ", dict(sorted(mo.items())))
    print("  by BO:    ", dict(bo.most_common()))
