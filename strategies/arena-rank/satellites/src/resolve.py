"""Apply a board's resolution rule to a leaderboard table, and pin the resolving vintage.

Board rule (2026 regime): order the models of the resolving arena slice by the Rank column
(ties broken by unrounded score, then alphabetically by company); optionally restrict to a
company subset; the winner is the company owning the k-th model in that order.
2025 regime: same but ordered by Arena Score directly ("highest arena score").
"""

import re
import unicodedata

# arena org name -> Polymarket leg name, where normalisation alone does not bridge them
ALIAS = {
    "spacexai": "xai",
    "moonshotai": "moonshot",
    "zhipuai": "zai",
    "zhipu": "zai",
    "01ai": "01ai",
    "mistralai": "mistral",
    "allenaiuw": "ai2",
    "googledeepmind": "google",
}


def norm(name):
    if not name:
        return ""
    s = unicodedata.normalize("NFKD", str(name)).lower()
    s = re.sub(r"[^a-z0-9]", "", s)
    return ALIAS.get(s, s)


# companies the "Best Chinese AI company" boards treat as primarily Chinese
# (union of the leg lists across every Chinese board in the family)
CHINESE = {
    "alibaba", "moonshot", "deepseek", "zai", "baidu", "bytedance", "tencent",
    "minimax", "xiaomi", "stepfun", "meituan", "01ai", "antgroup", "iflytek",
    "kuaishou", "inclusionai", "skywork", "internlm", "tsinghua", "openbmb",
    "shanghaiai", "sensetime", "yi",
}


def order_models(rows, res_var):
    """Sort a parsed table into resolution order. Lower rank / higher score first."""
    if res_var == "score":
        return sorted(rows, key=lambda r: (-r["score"], norm(r["org"])))
    return sorted(rows, key=lambda r: (r["rank"], -r["score"], norm(r["org"])))


def winner(rows, place, res_var, restriction=None, legs=None):
    """Company that owns the `place`-th ranked model. Returns (poly_leg_name, model_row)."""
    ordered = order_models(rows, res_var)
    if restriction == "chinese":
        ordered = [r for r in ordered if norm(r["org"]) in CHINESE]
    if len(ordered) < place:
        return None, None
    row = ordered[place - 1]
    o = norm(row["org"])
    if legs:
        for leg in legs:
            if norm(leg) == o:
                return leg, row
        return "Other", row
    return row["org"], row


def pin_vintage(vintages, slice_paths, when, dense_paths=("text",), max_lag_h=None):
    """Return (capture, quality) for the table live at `when`.

    quality: 'pinned'   dense series proves which data date was live and the resolving
                        slice has a capture with exactly that date
             'bracketed' captures either side of `when` carry the same data date
             'nearest'  best-effort: closest capture before `when`
             None       nothing usable
    """
    from vintage import ts_to_dt

    def sub(paths):
        return sorted(
            [v for v in vintages if v["path"] in paths and v["diag"]["n"] >= 20],
            key=lambda z: z["ts"],
        )

    res = sub(slice_paths)
    if not res:
        return None, None

    dense = sub(set(dense_paths) | set(slice_paths))
    live_date = None
    before = [v for v in dense if ts_to_dt(v["ts"]) <= when]
    after = [v for v in dense if ts_to_dt(v["ts"]) > when]
    if before:
        lag_h = (when - ts_to_dt(before[-1]["ts"])).total_seconds() / 3600
        if max_lag_h is None or lag_h <= max_lag_h:
            live_date = before[-1]["meta"].get("data_date")
    # a refresh between the last capture and `when` would invalidate live_date; the next
    # capture after `when` carrying the same date proves none landed.
    proven = bool(after and live_date and after[0]["meta"].get("data_date") == live_date)

    if live_date:
        same = [v for v in res if v["meta"].get("data_date") == live_date]
        if same:
            # prefer a capture taken before the check instant
            pre = [v for v in same if ts_to_dt(v["ts"]) <= when]
            return (pre[-1] if pre else same[0]), ("pinned" if proven else "bracketed")

    pre = [v for v in res if ts_to_dt(v["ts"]) <= when]
    return (pre[-1] if pre else res[0]), "nearest"
