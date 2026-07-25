"""Fetch + parse Wayback captures of the arena leaderboard around each board check instant.

Two capture series are used together:
  * the DENSE default series (lmarena.ai|arena.ai /leaderboard/text, 7k captures) dates each
    refresh to the hour — it stamps the same data date as every other slice of the same
    refresh, so it says *which vintage was live* at a check instant;
  * the SPARSE resolving-slice series (…/text/overall-no-style-control etc.) carries the
    actual table that resolved the market.

A vintage is "pinned" when a capture of the resolving slice carries the same stamped data
date as the vintage the dense series shows was live at the check instant.

Writes <out>/vintages.json: one record per (slice, capture) with the parsed table.
"""

import json
import os
import sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from arena_parse import parse_meta, parse_rows  # noqa: E402
from vintage import candidates, et_to_utc, fetch, load_cdx, norm_path  # noqa: E402

SLICE_PATHS = {
    "text_overall_nosc": ["text/overall-no-style-control"],
    "text_overall_sc": ["text/overall"],
    "text_math": ["text/math-no-style-control", "text/math"],
    "text_coding": ["text/coding-no-style-control", "text/coding"],
}
DENSE = ["text"]


def near(cands, when, n_before=3, n_after=3, max_days=45):
    b = [c for c in cands if c[0] <= when and (when - c[0]).days <= max_days][-n_before:]
    a = [c for c in cands if c[0] > when and (c[0] - when).days <= max_days][:n_after]
    return b + a


def main(cdx_files, boards_json, cache, out):
    rows = load_cdx(cdx_files)
    boards = json.load(open(boards_json))
    checks = sorted(
        {b["check_et"] for b in boards if b["check_et"] and b["board_type"]}
    )

    want = {}  # (ts, url) -> set of tags
    for name, paths in list(SLICE_PATHS.items()) + [("dense", DENSE)]:
        cands = candidates(rows, set(paths))
        for chk in checks:
            when = et_to_utc(chk)
            nb, na = (6, 6) if name == "dense" else (3, 3)
            for c in near(cands, when, nb, na):
                want.setdefault((c[1], c[2]), set()).add(name)

    items = sorted(want)
    print(f"{len(items)} distinct captures to fetch")

    def job(it):
        ts, url = it
        doc = fetch(ts, url, cache)
        if not doc:
            return None
        rws, diag = parse_rows(doc)
        meta = parse_meta(doc)
        return dict(
            ts=ts,
            url=url,
            path=norm_path(url),
            tags=sorted(want[it]),
            meta=meta,
            diag=diag,
            rows=rws,
        )

    got = []
    with ThreadPoolExecutor(max_workers=6) as ex:
        for i, r in enumerate(ex.map(job, items)):
            if r:
                got.append(r)
            if (i + 1) % 25 == 0:
                print(f"  {i+1}/{len(items)}  ok={len(got)}")
    json.dump(got, open(out, "w"))
    print(f"{len(got)}/{len(items)} captures parsed -> {out}")


if __name__ == "__main__":
    S = sys.argv[1]
    main(
        [f"{S}/cdxfull_l_text.json", f"{S}/cdxfull_a_text.json"],
        "strategies/arena-rank/satellites/data/poly/boards.json",
        f"{S}/wb_cache",
        "strategies/arena-rank/satellites/data/vintages.json",
    )
