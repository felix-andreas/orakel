"""Parse an arena.ai / lmarena.ai leaderboard HTML capture into rows.

The leaderboard is server-side rendered into a plain <table> (all rows, not virtualized).
Columns in the 2026 layout:

    Rank | Rank Spread (lo hi) | Model (name, "Org · License") | Score ±CI [Preliminary]
    | Votes | Price | Context

Org is taken from the "Org · License" text (the vendor SVG has a <title> for only ~20% of
orgs). Older (2025) layouts differ; parse_rows() counts what it could not read rather than
guessing — a silently-misparsed vintage corrupts a backtest (wiki first-print-vintages).
"""

import html
import json
import re
import sys

TAG = re.compile(r"<[^>]+>")
TR = re.compile(r"<tr\b", re.I)
TD = re.compile(r"<td\b", re.I)
SEP = re.compile(r"\s*[·|]\s*")


def _cells(tr_html):
    """Split one <tr> chunk into a list of plain-text token lists, one per <td>."""
    out = []
    for p in TD.split(tr_html)[1:]:
        p = p.split(">", 1)[1] if ">" in p else p  # drop the tag's own attributes
        txt = html.unescape(TAG.sub("\x00", p))
        out.append([t.strip() for t in txt.split("\x00") if t.strip()])
    return out


def _num(s):
    s = s.replace(",", "").replace("+", "").replace("±", "").strip()
    try:
        return float(s)
    except ValueError:
        return None


def parse_rows(doc):
    """Return (rows, diagnostics)."""
    rows, skipped = [], 0
    for ch in TR.split(doc)[1:]:
        cells = _cells(ch)
        if len(cells) < 4:
            continue
        flat = [t for c in cells for t in c]
        if "Rank" in flat and ("Model" in flat or "Score" in flat):
            continue  # header

        rank = _num(cells[0][0]) if cells[0] else None
        if rank is None:
            skipped += 1
            continue

        spread = [_num(t) for t in cells[1]]
        spread = [s for s in spread if s is not None]
        lo, hi = (spread + [None, None])[:2]

        # model cell: [model_name, "Org · License"] (sometimes preceded by an org label)
        model = org = license_ = None
        for i, t in enumerate(cells[2]):
            if SEP.search(t):
                parts = SEP.split(t)
                org = parts[0].strip()
                license_ = parts[-1].strip() if len(parts) > 1 else None
                if i > 0:
                    model = cells[2][i - 1]
                break
        if model is None:
            cand = [t for t in cells[2] if not SEP.search(t)]
            model = cand[-1] if cand else None
        if org is None:
            m = re.search(r"<title>([^<]{1,40})</title>", ch)
            org = html.unescape(m.group(1)).strip() if m else None

        sc = cells[3]
        prelim = any("Preliminary" in t for t in sc)
        score = ci = None
        for t in sc:
            if t.startswith("±"):
                ci = _num(t)
            elif score is None and _num(t) is not None:
                score = _num(t)

        votes = _num(cells[4][0]) if len(cells) > 4 and cells[4] else None

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
    return rows, dict(n=len(rows), skipped=skipped)


def parse_meta(doc):
    """Vote total / model count / data date rendered above the table."""
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
