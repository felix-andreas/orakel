"""Wayback vintage reconstruction for the arena leaderboard.

The resolving table is recomputed over the whole vote history at every refresh, so today's
table is NOT what resolved a past market (wiki/reference/first-print-vintages.md). But the
refresh cadence is discrete (~weekly) and every rendered page stamps its own data date, so
a capture does not have to sit *at* the check instant: any capture whose stamped data date
equals the date live at the check instant carries the resolving table exactly.

pin_vintage() therefore brackets each check instant with the nearest capture before and
after; when both carry the same stamped data date, the vintage is proven (no refresh landed
inside the bracket) and the table is the one that resolved the market.
"""

import json
import os
import subprocess
import time
import sys
from datetime import datetime, timedelta, timezone

WB = "https://web.archive.org/web/{ts}id_/{url}"
UA = "orakel-research/1.0"


def et_to_utc(s):
    """'2026-06-30T12:00' ET -> aware UTC datetime (US DST: 2nd Sun Mar - 1st Sun Nov)."""
    d = datetime.strptime(s, "%Y-%m-%dT%H:%M")
    y = d.year
    mar = datetime(y, 3, 8)
    dst_start = mar + timedelta(days=(6 - mar.weekday()) % 7)  # 2nd Sunday of March
    nov = datetime(y, 11, 1)
    dst_end = nov + timedelta(days=(6 - nov.weekday()) % 7)  # 1st Sunday of November
    off = 4 if dst_start <= d < dst_end else 5
    return (d + timedelta(hours=off)).replace(tzinfo=timezone.utc)


def ts_to_dt(ts):
    return datetime.strptime(ts, "%Y%m%d%H%M%S").replace(tzinfo=timezone.utc)


def load_cdx(paths):
    """paths: list of json files produced by a CDX query with fl=timestamp,original,..."""
    rows = []
    for p in paths:
        rows += json.load(open(p))[1:]
    return rows


def norm_path(u):
    u = u.split("?")[0]
    if "/leaderboard/" not in u:
        return "ROOT"
    return u.split("/leaderboard/", 1)[1].rstrip("/")


def candidates(rows, wanted_paths):
    out = []
    for ts, url, *rest in rows:
        if norm_path(url) in wanted_paths:
            out.append((ts_to_dt(ts), ts, url))
    out.sort()
    return out


def fetch(ts, url, cache):
    os.makedirs(cache, exist_ok=True)
    key = f"{cache}/{ts}_{norm_path(url).replace('/', '-') or 'ROOT'}.html"
    if os.path.exists(key) and os.path.getsize(key) > 5000:
        return open(key, encoding="utf-8", errors="replace").read()
    for attempt in range(3):
        r = subprocess.run(
            ["curl", "-sL", "--compressed", "--max-time", "180", "-A", UA,
             WB.format(ts=ts, url=url)],
            capture_output=True,
        )
        raw = r.stdout
        if raw[:2] == b"\x1f\x8b":  # archived bytes were gzip and curl did not unwrap
            import gzip
            try:
                raw = gzip.decompress(raw)
            except OSError:
                raw = b""
        if r.returncode == 0 and len(raw) >= 5000:
            doc = raw.decode("utf-8", errors="replace")
            open(key, "w", encoding="utf-8").write(doc)
            return doc
        time.sleep(2 * (attempt + 1))
    return None


def bracket(cands, when, max_days=21):
    """Nearest capture before and after `when`, within max_days."""
    before = [c for c in cands if c[0] <= when]
    after = [c for c in cands if c[0] > when]
    b = before[-1] if before and (when - before[-1][0]).days <= max_days else None
    a = after[0] if after and (after[0][0] - when).days <= max_days else None
    return b, a


if __name__ == "__main__":
    print(json.dumps({"usage": "imported by backtest.py"}))
