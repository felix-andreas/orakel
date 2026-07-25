"""Classify frozen arena events into (arena slice, place k, restriction, check instant).

Every board in this family is "which company owns the k-th ranked model in arena slice S,
read at instant T". The rules text names the resolving URL explicitly, so the URL is the
authority for S — the slug is not (the same board type has been slugged four ways since
2025, and the site rebranded lmarena.ai -> arena.ai mid-family).

Two resolution-variable regimes exist and must not be mixed:
  * 2025 boards resolve on the **Arena Score** ("highest arena score")
  * 2026 boards resolve on the **Rank column** ("highest arena rank")
They coincide except at ties and when the displayed rank groups statistically-tied models.
"""

import glob
import json
import re
import sys
from datetime import datetime

# resolving URL path -> canonical slice id
PATH2SLICE = {
    "text": "text_overall",  # ambiguous alone; style-control phrase disambiguates
    "text/overall": "text_overall",
    "text/overall-no-style-control": "text_overall",
    "text/math": "text_math",
    "text/math-no-style-control": "text_math",
    "text/coding": "text_coding",
    "text/coding-no-style-control": "text_coding",
    "code/webdev": "code_webdev",
    "webdev": "code_webdev",
    "agent": "agent",
}

ORDINAL = [
    (r"highest[- ]ranked|highest arena (?:rank|score)|highest rank\b", 1),
    (r"second[- ]highest", 2),
    (r"third[- ]highest", 3),
    (r"fourth[- ]highest", 4),
]


def classify(desc, title):
    d = re.sub(r"\s+", " ", desc or "")
    dl = d.lower()
    tl = (title or "").lower()

    # --- resolving slice, from the URL(s) in the rules text ---
    paths = re.findall(r"(?:lmarena\.ai|arena\.ai)/leaderboard/([a-z0-9/\-]+)", dl)
    paths = [p.rstrip("/.,)") for p in paths]
    slice_ = None
    resolving_path = None
    for p in paths:
        if p in PATH2SLICE:
            slice_, resolving_path = PATH2SLICE[p], p
            break
    if slice_ is None:
        if "webdev" in dl or "webdev" in tl:
            slice_, resolving_path = "code_webdev", "code/webdev"
        elif "| math" in dl or "math arena" in dl:
            slice_, resolving_path = "text_math", "text/math"
        elif "| coding" in dl:
            slice_, resolving_path = "text_coding", "text/coding"
        elif "agent arena" in dl:
            slice_, resolving_path = "agent", "agent"
        elif "leaderboard" in dl:
            slice_, resolving_path = "text_overall", "text"

    # --- style control on/off (only meaningful for text slices) ---
    sc = None
    if slice_ and slice_.startswith("text"):
        if re.search(r"style control (?:is )?(?:on\b|checked)|set to default \(style control on\)", dl):
            sc = True
        elif re.search(r"style control (?:is )?(?:off\b|unchecked)|no-style-control", dl):
            sc = False
        elif "style control on" in tl:
            sc = True
        if resolving_path and resolving_path.endswith("no-style-control"):
            sc = False

    # --- place k ---
    k = None
    for pat, v in ORDINAL:
        if re.search(pat, dl):
            k = v
            break
    m = re.search(r"occupies (first|second|third|fourth) place", dl)
    if m:
        k = {"first": 1, "second": 2, "third": 3, "fourth": 4}[m.group(1)]
    if k is None:
        if re.search(r"\bthird\b|#3\b", tl):
            k = 3
        elif re.search(r"\bsecond\b|#2\b", tl):
            k = 2
        elif re.search(r"\bbest\b|\btop\b|#1\b", tl):
            k = 1

    # --- restriction ---
    restriction = "chinese" if ("chinese" in dl or "chinese" in tl) else None

    # --- resolution variable: rank column vs arena score ---
    if re.search(r'"rank" (?:column|section)|highest rank\b|arena rank', dl):
        resvar = "rank"
    elif re.search(r'"arena score"|highest arena score', dl):
        resvar = "score"
    else:
        resvar = None

    # --- check instant (ET) ---
    check = None
    m = re.search(r"checked on ([A-Z][a-z]+ \d{1,2}, \d{4}),? (\d{1,2}):(\d{2})\s*(AM|PM) ET", d)
    if m:
        day = datetime.strptime(m.group(1), "%B %d, %Y")
        h = int(m.group(2)) % 12 + (12 if m.group(4) == "PM" else 0)
        check = day.replace(hour=h, minute=int(m.group(3))).strftime("%Y-%m-%dT%H:%M")

    return dict(
        slice=slice_,
        resolving_path=resolving_path,
        style_control=sc,
        place=k,
        restriction=restriction,
        res_var=resvar,
        check_et=check,
    )


def board_type(c):
    """Stable id for a recurring board: slice + style-control + place + restriction."""
    if not c["slice"] or not c["place"]:
        return None
    s = c["slice"]
    if s == "text_overall":
        s += "_sc" if c["style_control"] else "_nosc"
    t = f"{s}_{c['place']}"
    if c["restriction"]:
        t += "_" + c["restriction"]
    return t


def main(root):
    out = []
    for p in sorted(glob.glob(f"{root}/events/*.json")):
        ev = json.load(open(p))
        if not ev:
            continue
        e = ev[0]
        c = classify(e.get("description"), e.get("title"))
        legs = []
        for m in e.get("markets", []):
            try:
                outcomes = json.loads(m["outcomes"])
                prices = [float(x) for x in json.loads(m["outcomePrices"])]
                toks = json.loads(m.get("clobTokenIds") or "[]")
            except Exception:
                continue
            yi = outcomes.index("Yes") if "Yes" in outcomes else 0
            legs.append(
                dict(
                    company=m.get("groupItemTitle") or m.get("question"),
                    slug=m.get("slug"),
                    condition_id=m.get("conditionId"),
                    token_id=toks[yi] if len(toks) > yi else None,
                    price=prices[yi] if len(prices) > yi else None,
                    closed=m.get("closed"),
                    volume=m.get("volumeNum"),
                    liquidity=m.get("liquidityNum"),
                    best_bid=m.get("bestBid"),
                    best_ask=m.get("bestAsk"),
                    spread=m.get("spread"),
                    last=m.get("lastTradePrice"),
                    start=m.get("startDate"),
                )
            )
        winner = None
        if e.get("closed"):
            won = [l["company"] for l in legs if l["price"] is not None and l["price"] > 0.99]
            winner = won[0] if len(won) == 1 else (won or None)
        out.append(
            dict(
                slug=e["slug"],
                title=e.get("title"),
                board_type=board_type(c),
                closed=e.get("closed"),
                volume=e.get("volume"),
                liquidity=e.get("liquidity"),
                start=e.get("startDate"),
                end=e.get("endDate"),
                n_legs=len(legs),
                winner=winner,
                legs=legs,
                **c,
            )
        )
    json.dump(out, open(f"{root}/boards.json", "w"), indent=1)
    print(f"{len(out)} boards -> {root}/boards.json")
    return out


if __name__ == "__main__":
    main(sys.argv[1])
