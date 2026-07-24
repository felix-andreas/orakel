#!/usr/bin/env python3
"""Parse Wayback captures of GISTEMP GLB.Ts+dSST.{txt,csv} into a vintage matrix.

Input: a directory of files named <wayback_ts>.txt / <wayback_ts>.csv
Output: vintages.csv with columns capture_ts, year, month, anom_c

txt format: GISTEMP integer hundredths (e.g. 118 = 1.18 C), rows per year, months
Jan..Dec in columns 1..12, missing = ****. Two header blocks repeat; lines starting
with 'Year' or non-numeric are skipped.
csv format: Year,Jan,...,Dec,... with decimal degrees, missing = ***.
"""

import csv
import os
import re
import sys

MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]


def parse_txt(path):
    out = {}
    for line in open(path, errors="replace"):
        m = re.match(r"^\s*(\d{4})\s+(.*)$", line)
        if not m:
            continue
        year = int(m.group(1))
        if not (1880 <= year <= 2030):
            continue
        fields = m.group(2).split()
        if len(fields) < 12:
            continue
        for i in range(12):
            f = fields[i]
            if re.match(r"^-?\d+$", f):
                out[(year, i + 1)] = int(f) / 100.0
    return out


def parse_csv(path):
    out = {}
    rows = list(csv.reader(open(path, errors="replace")))
    header_idx = None
    for i, r in enumerate(rows):
        if r and r[0] == "Year":
            header_idx = i
            break
    if header_idx is None:
        return out
    for r in rows[header_idx + 1 :]:
        if not r or not re.match(r"^\d{4}$", r[0]):
            continue
        year = int(r[0])
        for i in range(12):
            v = r[i + 1].strip() if i + 1 < len(r) else ""
            try:
                out[(year, i + 1)] = float(v)
            except ValueError:
                pass
    return out


def main():
    vdir, outpath = sys.argv[1], sys.argv[2]
    rows = []
    for fn in sorted(os.listdir(vdir)):
        ts, ext = fn.rsplit(".", 1)
        if ext == "txt":
            d = parse_txt(os.path.join(vdir, fn))
        elif ext == "csv":
            d = parse_csv(os.path.join(vdir, fn))
        else:
            continue
        if not d:
            print(f"WARN empty parse {fn}", file=sys.stderr)
            continue
        for (y, m), v in sorted(d.items()):
            if y >= 2012:  # enough history for 10y seasonal-delta fits; keeps the csv small
                rows.append({"capture_ts": ts, "fmt": ext, "year": y, "month": m, "anom_c": v})
    with open(outpath, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=["capture_ts", "fmt", "year", "month", "anom_c"])
        w.writeheader()
        w.writerows(rows)
    print(f"{len(rows)} vintage cells -> {outpath}")


if __name__ == "__main__":
    main()
