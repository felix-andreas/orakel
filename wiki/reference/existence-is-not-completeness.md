# A file that exists is not a file that is finished

> **In plain English:** most caches ask "is the file there?" and stop. That is the right
> question for a photograph and the wrong one for a tape measure that is still being unrolled.
> If you save a record of something that had not finished happening yet, every later run will
> hand you the truncated version and call it cached — forever, and without an error.

The right question is never *is the file there*. It is:

> **Was this file written after the thing it describes stopped changing?**

## Why it is nearly invisible

The truncated file is **valid**. It parses, it has the right schema, it has plausible values.
Nothing throws. Downstream code that guards on freshness — a max-age check, a "no data past
T" filter — then does the damage quietly, by *dropping* rows rather than complaining about
them. You lose sample, not correctness, so the symptom is a number that is merely smaller
than you expected, which nobody investigates.

And the bias is not a wash. A record cut short mid-window is missing **the end** of the
window, which is where resolution drama, volume and volatility live. It is the same shape of
error as [lifetime-volume-is-look-ahead](lifetime-volume-is-look-ahead.md), pointing the
other way: that page is about a covariate that knows too much about the future, this one is
about a record that knows too little.

## Measured — four instances in one crate in three days

`barrier-touch/ladder-rv`, 2026-07-26 → 07-29:

| where | what it cached | what it cost |
|---|---|---|
| `cmd_candles` | a day-file written mid-day | 07-27 WTIU6 held at **21.9 KB of a true 69.7 KB**; SPY/NVDA held as 52-byte `no_data` — a whole RTH session missing from σ. Live four days; also made an earlier run log RV14 48.8% against a true 51.7%. |
| `cmd_clob` | CLOB `prices-history`, which grows until the market closes | would have scored a trial against a price history **stopping three days before resolution** — silently, one dropped leg at a time |
| `cmd_tape` | the trade tape, likewise growing | turns "any taker trade in the **last 7 days**" into "…in the 7 days before whenever we happened to fetch" — a different and unanswerable question |
| **the daily freeze itself** | a `tar` line that omitted a directory | the per-leg record a **pre-registration** was to be scored from existed in **one container**, in neither git nor object storage |

The fix in code is three lines and identical every time:

```
fn complete_through(path, period_end) -> bool {
    mtime(path) >= period_end          // NOT path.exists()
}
```

…where `period_end` is the end of the period the file claims to cover: the day, the market's
close, the leg's resolution. Refetch whenever it returns false.

## The instance that has no code to fix

The fourth row above is the one worth the page. Three fetchers had a bug; the **archive
procedure** had the same bug with nothing to patch.

The daily duty was written as *"freeze the candle archive"*. A tarball was built by hand from
a `tar` line naming two directories, uploaded, and verified. The manifest existed. Verification
passed. And a third directory — the one holding the per-leg model output — had never been in
the `tar` line at all, so it was in no archive and, being gitignored as bulk data, in no repo
either.

Nothing was broken. The freeze's *existence* was taken as evidence of its *completeness*.

Two rules follow:

1. **A freeze is only as complete as the line that built it**, and that line is retyped by a
   human (or an agent) every single day. Put the file list in a script or a checklist, not in
   muscle memory. Enumerate what the archive must contain *before* asking whether it uploaded.
2. **"Verified" usually means the bytes arrived, not that the data is right.** Content-hash
   verification (`r2data verify` and its kin) HEADs the object and compares size and digest.
   It cannot see inside the tarball. An archive containing a truncated file verifies clean
   every day of its life.

## The audit question

Ask it of every cached artifact, and then — the step that is actually skipped — of every
archive and freeze procedure:

> Does anything here treat *presence* as *completeness*? For each file: what is the period it
> claims to cover, and had that period ended when it was written?

Fields and files that fail this test in our pipelines so far: any per-day candle or price
file written before that day closed; any `prices-history` or trade-tape pull taken while the
market is open; any snapshot of a market's own record made before it resolved; and any
archive whose contents are specified by a command rather than by a manifest of what should
be in it.

## See also

- [lifetime-volume-is-look-ahead](lifetime-volume-is-look-ahead.md) — the mirror failure: a
  field that reflects the *settled* state when you needed the state at your checkpoint
- [stale-feed-gate](stale-feed-gate.md) — the other way a frozen input produces a confident
  wrong answer, there because the market was shut rather than because the cache was wrong
- [first-print-vintages](first-print-vintages.md) — using a revised value where the market
  settled on the original
