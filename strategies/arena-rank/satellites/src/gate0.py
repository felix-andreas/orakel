"""Gate 0 — can we reproduce past resolutions from the archived leaderboard?

Kill if <90% of resolved instances with a usable vintage reproduce: it would mean we cannot
read the resolution variable, and everything downstream is noise.
"""

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from resolve import norm, pin_vintage, winner  # noqa: E402
from vintage import et_to_utc  # noqa: E402

SLICE_PATHS = {
    "text_overall_nosc": {"text/overall-no-style-control"},
    "text_overall_sc": {"text/overall", "text"},
    "text_math": {"text/math-no-style-control", "text/math"},
    "text_coding": {"text/coding-no-style-control", "text/coding"},
}


def slice_key(b):
    s = b["slice"]
    if s == "text_overall":
        return "text_overall_sc" if b["style_control"] else "text_overall_nosc"
    return s


def is_company_board(b):
    bad = ("claude-", "gemini-", "gpt-", "grok-", "qwen", "kimi", "glm-", "muse-",
           "dola-", "Model ")
    return b["n_legs"] > 0 and not any(
        (l["company"] or "").startswith(bad) for l in b["legs"]
    )


def main(root):
    boards = json.load(open(f"{root}/poly/boards.json"))
    vint = json.load(open(f"{root}/vintages.json"))
    out = []
    for b in boards:
        if not (b["closed"] and b["board_type"] and b["check_et"] and is_company_board(b)):
            continue
        if not isinstance(b["winner"], str):
            continue
        sk = slice_key(b)
        paths = SLICE_PATHS.get(sk)
        if not paths:
            continue
        when = et_to_utc(b["check_et"])
        cap, qual = pin_vintage(vint, paths, when)
        if cap is None:
            out.append(dict(slug=b["slug"], board_type=b["board_type"],
                            check=b["check_et"], quality=None, ok=None))
            continue
        legs = [l["company"] for l in b["legs"]]
        pred, row = winner(cap["rows"], b["place"], b["res_var"], b["restriction"], legs)
        out.append(
            dict(
                slug=b["slug"],
                board_type=b["board_type"],
                check=b["check_et"],
                quality=qual,
                capture_ts=cap["ts"],
                capture_path=cap["path"],
                data_date=cap["meta"].get("data_date"),
                n_rows=cap["diag"]["n"],
                predicted=pred,
                actual=b["winner"],
                model=row["model"] if row else None,
                margin=None,
                ok=(norm(pred) == norm(b["winner"])),
            )
        )
    json.dump(out, open(f"{root}/gate0.json", "w"), indent=1)
    return out


if __name__ == "__main__":
    res = main(sys.argv[1] if len(sys.argv) > 1 else
               "strategies/arena-rank/satellites/data")
    got = [r for r in res if r["ok"] is not None]
    print(f"{len(res)} resolved company boards; {len(got)} with a usable vintage")
    for q in ("pinned", "bracketed", "nearest"):
        s = [r for r in got if r["quality"] == q]
        if s:
            print(f"  {q:10s} n={len(s):3d} reproduced {sum(r['ok'] for r in s)}/{len(s)}"
                  f" = {100*sum(r['ok'] for r in s)/len(s):.0f}%")
    print(f"  {'TOTAL':10s} n={len(got):3d} reproduced {sum(r['ok'] for r in got)}/{len(got)}"
          f" = {100*sum(r['ok'] for r in got)/len(got):.0f}%")
    print("\n--- misses ---")
    for r in got:
        if not r["ok"]:
            print(f"  {r['check'][:10]} {r['board_type']:26s} pred={str(r['predicted'])[:14]:14s}"
                  f" actual={str(r['actual'])[:14]:14s} q={r['quality']:9s}"
                  f" cap={r['capture_ts']} dd={r['data_date']} rows={r['n_rows']}"
                  f" model={r['model']}")
