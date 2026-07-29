# Archive audit: does the freeze actually contain what Friday needs?

**2026-07-29 (day 7), model claude-opus-5 effort xhigh.** Written on the premise that after
two silent partial-data bugs in two days there is a third. There were three, and the worst
one is not a bug in code at all.

The audit question, from `wiki/reference/lifetime-volume-is-look-ahead.md` generalised:
**where does something treat a file's existence as evidence of its completeness?**

## Answer up front

| # | finding | severity for 07-31 | status |
|---|---|---|---|
| 1 | `data/out/predictions_2026-07-28.csv` frozen **nowhere** | **critical** — the day-6 per-leg record existed in one container | **fixed today** (rescue freeze) |
| 2 | 07-28's predictions file **predates** the `q_iv`/`q_blend` columns the RV/IV pre-registration says it contains | **high** — shrinks the pre-registered comparison to two days | **cannot be fixed**, see §2 |
| 3 | `cmd_clob` never refetches a growing price series | high — would have silently dropped Friday checkpoints | **fixed today** |
| 4 | `cmd_tape` never refetches a growing trade tape | medium — makes the tape gate unanswerable | **fixed today** |
| 5 | `r2data verify` checks the blob, never the contents | standing hazard | documented, not changed |

## 1. The daily freeze does not contain the predictions files (critical)

`data/.gitignore` ignores `candles/ vol/ out/ tape/ clob*/` — correct, and deliberate (it
stops a stray `git add -A` from a concurrent agent sweeping in 19 MB). But it means
**`data/out/` is in neither git nor, as it turns out, R2.**

The daily freeze `candles-2026-07-28.tar.gz` contains exactly `candles/` and `vol/`. I
verified this by pulling and listing it, not by reading its note. Day-6 froze candles and
did **not** cut a `live-*` snapshot — and `live-*` is the freeze that carries `out/`
(confirmed: `live-2026-07-27.tar.gz` does contain `out/predictions_2026-07-27.csv`).

So `predictions_2026-07-28.csv` — 92 legs, the source record behind day-6's 14 ledger rows —
had **no durable copy anywhere**. It survived only because this working tree survived.

The failure mode is the one the wiki page names, one level up: the daily duty is written as
"freeze the candle+vol archive", the manifest exists, `r2data verify` returns OK, and the
freeze is therefore treated as *the* archive. Its existence was evidence of its completeness.
Nothing errored; the missing file was simply never in scope.

**Fixed:** `live-2026-07-29.tar.gz` includes `out/predictions_{07-27,07-28,07-29}.csv` plus
`events_live/`, pushed and verified. The 07-28 record is now durable.

## 2. The 07-28 file does not contain the columns the pre-registration relies on (high)

`results/prereg-rv-iv-blend-2026-07-28.md` states:

> A, B and C are recorded per leg by `cmd_live` in `data/out/predictions_<date>.csv`
> (columns `probability`, `q_iv`, `q_blend`, plus `sigma_rv` / `sigma_iv`), so no
> re-derivation is needed on Friday — the numbers are already frozen in the daily archive.

Measured headers:

| file | has `q_iv` / `q_blend` / `sigma_*` |
|---|---|
| `predictions_2026-07-27.csv` | no (predates `pricer` too) |
| `predictions_2026-07-28.csv` | **no** |
| `predictions_2026-07-29.csv` | yes — 67 legs, all `feed_open=1` |

mtimes settle it: the file was written 07-28 **01:21:59Z**; `src/main.rs` and the binary were
rebuilt at **01:30** the same morning. The day-6 run wrote its predictions file, *then* the
IV columns were added, and the file was never regenerated. The worklog and the
pre-registration both describe the intended state as an accomplished one.

**Not fixed, deliberately.** The inputs to re-derive 07-28's `q_iv`/`q_blend` do exist in the
frozen candle archive. Re-deriving them **now, by me, after seeing today's numbers**, would be
a change to the comparison made by the person who scores it — the exact thing the
pre-registration exists to prevent, and it already carries one such change (the `bump`
fairness fix) declared in advance.

**Consequence, recorded before the outcome:** the RV/IV comparison is scored from the
**07-29 and 07-30** files only. It still clears its own power floor comfortably — today's
file alone carries **67 legs** with all three pricers, every one resolving 07-31 21:00Z,
against a floor of n ≥ 30. The floor is met; the day-count is two, not four.

**Separately flagged, also pre-outcome:** the pre-registration specifies the metric at the
**daily 12:00Z in-window checkpoint**, but `cmd_live` fires at ~01:1xZ and that is the
timestamp the recorded q's carry. "12:00Z" was inherited from the *backtest's* gate-2 anchor.
Scoring at the recorded ~01:14Z anchor requires no re-derivation; scoring at a true 12:00Z
requires re-deriving spot and σ from the archive. Both are defensible; **choosing between
them after Friday's outcome is not.** Flagging it now so the choice is made blind. I have not
picked one — that is the CEO's call, and it should be made before 21:00Z Friday.

## 3 & 4. Two growing series cached as if immutable (fixed)

`fetch_all` skips any path that merely `exists()`. That is right for an immutable blob and
wrong for an append-only one. `cmd_candles` was fixed for this on 07-28; the same defect was
still live in two more places:

- **`cmd_clob`** — `prices-history` grows until the market closes. Fetched once mid-window,
  every later run reports it "cached". `cmd_analyze` reads exactly these files for the
  window-open and daily checkpoints, i.e. **the Brier metric**. Nothing would error: the file
  parses, `load_series` returns a valid series, and `price_at`'s max-age guard then returns
  `None` for every checkpoint past the fetch date — so legs leave the scored set one at a
  time, silently. On Friday, with `clob60/` restored from the 07-25 backtest freeze, this
  would have scored the trial against a price history that stops on **July 25**.
- **`cmd_tape`** — `if out.exists() { continue; }`. The tape gate asks "has anyone traded
  within 5c of our quote in the **last 7 days**". Against a stale file that silently becomes
  "in the 7 days before whenever we happened to fetch" — a different question, and an
  unanswerable one.

Both now use one shared `complete_through(path, period_end)`: a file counts as complete only
if its mtime is after the end of the period it claims to cover, which for a leg is `l.we`.
`cmd_clob` reports stale refreshes in its summary line rather than folding them into
"cached". `ladderrv selftest` passes; the pricer is untouched.

## 5. `r2data verify` verifies the blob, not the data (documented, not changed)

`verify` HEADs the object and compares size against the manifest. That is a real check and it
catches real things. It cannot see inside the tarball. `candles-2026-07-27.tar.gz` verified
OK every day while containing a WTIU6 file truncated to 21.9 KB of a true 69.7 KB.

Not a defect to fix in `r2data` — content validation belongs to the producer. But the phrase
"frozen and verified" should stop being read as "the data is right". It means the bytes we
uploaded are the bytes that are there. **A freeze is only as complete as the `tar` line that
built it**, and that line is written fresh, by hand, every day.

## What Friday still needs, that no freeze covers

1. A fresh `ladderrv discover` — `legs.csv` exists only in `backtest-metals-2026-07-25` and
   its `closed` / `winner` / `volume` columns are a 07-25 snapshot. Board coverage is fine
   (58 boards, the week-of-Jul-27 family included); the resolution state is not.
2. A fresh `ladderrv clob data 60` **after** 21:00Z — now that it actually refetches.
3. The both-ways `condition_ids` lookup (`&closed=true` *and* without), per the readiness doc.
4. The candle archive force-refetched **after** 07-31 21:00Z, WTIU6 **and** WTIV6.
