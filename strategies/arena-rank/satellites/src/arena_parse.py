"""Parse an arena.ai / lmarena.ai leaderboard HTML capture into rows.

The leaderboard is server-side rendered into a plain <table> (all rows, not virtualized),
but the column set changed three times across the family's life:

  2025-05 → 2025-1x  Rank (UB) | Model | Score | 95% CI (±) | Votes | Organization | License
  2025-1x → 2026-06  Rank | Rank Spread | Model | Score | 95% CI (±) | Votes | Org | License
  2026-06 → now      Rank | Rank Spread | Model(+"Org · License") | Score(±CI, Preliminary)
                     | Votes | Price $/M | Context

So the parser is header-driven: it reads the <th> row and maps columns by name. Fixed
column indices silently misparse older vintages (they yield a vote count where the score
should be), and a silently-misparsed vintage corrupts a backtest exactly the way revised
GISTEMP did — wiki/reference/first-print-vintages.md.
"""

import html
import json
import re
import sys

TAG = re.compile(r"<[^>]+>")
TR = re.compile(r"<tr\b", re.I)
TD = re.compile(r"<td\b", re.I)
TH = re.compile(r"<th\b", re.I)
SEP = re.compile(r"\s*[·|]\s*")


def _toks(cell_html):
    cell_html = cell_html.split(">", 1)[1] if ">" in cell_html else cell_html
    txt = html.unescape(TAG.sub("\x00", cell_html))
    return [t.strip() for t in txt.split("\x00") if t.strip()]


def _num(s):
    s = str(s).replace(",", "").replace("+", "").replace("±", "").strip()
    try:
        return float(s)
    except ValueError:
        return None


def _header(doc):
    """Column name -> index, from the first <tr> that contains <th> cells."""
    for ch in TR.split(doc)[1:]:
        ths = TH.split(ch)[1:]
        if len(ths) < 4:
            continue
        names = []
        for t in ths:
            tk = _toks(t)
            names.append(tk[0].lower() if tk else "")
        idx = {}
        for i, n in enumerate(names):
            if n.startswith("rank spread"):
                idx["spread"] = i
            elif n.startswith("rank"):
                idx["rank"] = i
            elif n.startswith("model"):
                idx["model"] = i
            elif n.startswith("score") or n.startswith("arena score"):
                idx["score"] = i
            elif "ci" in n:
                idx["ci"] = i
            elif n.startswith("votes"):
                idx["votes"] = i
            elif n.startswith("organization") or n.startswith("provider"):
                idx["org"] = i
            elif n.startswith("license"):
                idx["license"] = i
        if "rank" in idx and "model" in idx and "score" in idx:
            return idx, names
    return None, None


def parse_rows(doc):
    """Return (rows, diagnostics). Empty rows list if the header cannot be read."""
    idx, names = _header(doc)
    if not idx:
        return [], dict(n=0, skipped=0, layout=None, reason="no-header")

    rows, skipped = [], 0
    for ch in TR.split(doc)[1:]:
        if TH.search(ch):
            continue
        cells = [_toks(p) for p in TD.split(ch)[1:]]
        if len(cells) <= max(idx.values()):
            continue

        rank = _num(cells[idx["rank"]][0]) if cells[idx["rank"]] else None
        if rank is None:
            skipped += 1
            continue

        lo = hi = None
        if "spread" in idx:
            sp = [_num(t) for t in cells[idx["spread"]]]
            sp = [s for s in sp if s is not None]
            lo, hi = (sp + [None, None])[:2]

        mc = cells[idx["model"]]
        model = org = license_ = None
        for i, t in enumerate(mc):
            if SEP.search(t):  # 2026-06+ layout packs "Org · License" into the model cell
                parts = SEP.split(t)
                org, license_ = parts[0].strip(), parts[-1].strip()
                if i > 0:
                    model = mc[i - 1]
                break
        if model is None:
            # older layouts: the vendor icon's <title> may prefix the model name
            cand = [t for t in mc if not SEP.search(t)]
            model = cand[-1] if cand else None
        if "org" in idx and cells[idx["org"]]:
            org = cells[idx["org"]][0]
        if "license" in idx and cells[idx["license"]]:
            license_ = cells[idx["license"]][0]

        sc = cells[idx["score"]]
        prelim = any("Preliminary" in t for t in sc)
        score = ci = None
        for t in sc:
            if t.startswith("±"):
                ci = _num(t)
            elif score is None and _num(t) is not None:
                score = _num(t)
        if "ci" in idx and cells[idx["ci"]]:
            vals = [_num(t) for t in cells[idx["ci"]] if _num(t) is not None]
            if vals:
                ci = vals[0]

        votes = None
        if "votes" in idx and cells[idx["votes"]]:
            votes = _num(cells[idx["votes"]][0])

        if score is None or model is None or org is None:
            skipped += 1
            continue
        rows.append(
            dict(
                rank=int(rank),
                spread_lo=int(lo) if lo is not None else None,
                spread_hi=int(hi) if hi is not None else None,
                model=model,
                org=org,
                license=license_,
                score=score,
                ci=ci,
                preliminary=prelim,
                votes=int(votes) if votes is not None else None,
            )
        )
    return rows, dict(n=len(rows), skipped=skipped, layout="|".join(sorted(idx)))


def parse_meta(doc):
    """Vote total / model count / stamped data date rendered above the table."""
    txt = re.sub(r"\s+", " ", html.unescape(TAG.sub(" ", doc)))
    out = {}
    m = re.search(r"([\d,]{5,})\s+votes", txt, re.I)
    if m:
        out["total_votes"] = int(m.group(1).replace(",", ""))
    m = re.search(r"([\d,]+)\s+models", txt, re.I)
    if m:
        out["n_models"] = int(m.group(1).replace(",", ""))
    m = re.search(r"([A-Z][a-z]{2}\s+\d{1,2},\s+\d{4})", txt)
    if m:
        out["data_date"] = m.group(1)
    return out


if __name__ == "__main__":
    doc = open(sys.argv[1], encoding="utf-8", errors="replace").read()
    rows, diag = parse_rows(doc)
    print(json.dumps(dict(meta=parse_meta(doc), diag=diag, rows=rows), indent=1))
