# arena-rank/favourite-shrinkage — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **STATE AS OF 2026-07-26: all 7 July applications are `active = false`, zero prediction
  rows proposed, and the CEO has an open decision in
  `roles/ceo/inbox/2026-07-26-favourite-shrinkage-band-test.md`** (hold the slot / park it /
  re-scope). Do not re-activate a board without re-running the four applicability clauses.
- The **day-3 pre-registered band test PASSED** (run early, 2026-07-26). Do not retire the
  mechanism on it. Numbers: fundable 0.60–0.90 gain +17.2pp instance-level (t=+7.03, 46
  instances, 10 months), +12.5c/trade, +457% annualised, and the **only** band whose 95%
  lower bound on the favourite's win rate (0.846) clears its break-even (0.829).
  0.93–1.00 is 16/16 and still uninvestable: needs 97.2%, bound 0.819, **2.83 losses per
  100 wipe it out**. `results/fundable-band-2026-07-26.md`.
- **The new clause 4: LEADERBOARD MARGIN.** Sharpen only where the ranking is persistent.
  Margin ≥4 pts, or the market's favourite ≠ the current place-holder. At margin 0–3 with
  the crowd backing the incumbent (n=5, fundable band): market 0.800 → realised 0.800,
  gap **+0.0pp**, and α=1.75 says 0.951 — a 15.1pp overshoot, our worst cell anywhere.
- **Never quote the 0.976–0.982 one-refresh persistence for a Preliminary/low-vote pair.**
  That is the top-30 POOLED figure. Measured: sd(Δscore) 5.87 Preliminary / 6.55 <5k votes
  vs 1.60–2.25 established; gap 3–5 persistence 0.846 (11/13) when both rows are <5k
  votes; Chinese company-level leadership at margin 0–3 persists 0.44–0.54 over 1–14d.
  That mis-citation is the only reason the Chinese board looked tradeable on 07-26.
- Both mandatory re-checks are **done and clean** (2026-07-26): leg-sum gate — gain is
  entirely in the leg-sum ≤1.05 half (+0.129 vs −0.184); no null (uniform, flat-0.90,
  margin) beats the market anywhere; phantom split — **137/138 checkpoints LIVE**, live-book
  headline = pooled headline. This family does not have the bo3/tennis disease.
- **Books move overnight — re-measure, never trust a day-old `[book]` block.** Chinese
  favourite went 0.8275 → 0.7765 on *no new leaderboard data*; overall-nosc-3 went
  0.934 → 0.9831. Re-run `src/live_state.py` at the start of every run.
- Venue fee for this family is **`tech_fees`, rate 0.04, takerOnly** (read off the live
  `feeSchedule` 2026-07-26). Taker-only, entry and market-exit, never at resolution.
- 2026-08 and 2026-09 cohorts exist but are **UNPRICED**: leg-sums 6.5–12.5, phantom ~0.5
  midpoints on empty books. Nothing to trade or measure there until they price (~2–3 weeks
  before the check).

## Medium-term

- Resolving slice = `arena.ai/leaderboard/text/overall-no-style-control` (SC **off**).
  The default `/text` page has SC **on** and orders differently. Reading the wrong table
  was the founding idea's flagship error.
- The winner is **the company owning the k-th ranked MODEL** (`resolve.winner`), not the
  k-th distinct company. Anthropic owns ranks 1–4 of the live table, so it owns places
  1, 2 *and* 3. Getting this wrong silently produces bogus persistence numbers — I did it
  once on 07-26 before checking `resolve.py`.
- SC-on boards have no clean vintage series: the archive's SC-on slice is the default
  `text` path, which mixes layouts and gives sd(Δscore) ≈ 27. Treat SC-on persistence
  numbers as unusable, not as low.
- Refresh cadence of the resolving slice: median 7d, p90 20d, max 39d. A capture need not
  sit at the check instant — pages stamp their own data date. A refresh landing inside the
  check window resolves toward the **fresher** table (1 of 51 historical instances).
- Wayback: CDX must be called over **https**; `…/web/<ts>id_/<url>` returns raw gzip; site
  rebranded lmarena.ai → arena.ai (captures continue under the new host).
- `src/archive_tables.py` archives the live resolving tables daily (memory duty 1). Run it
  every day the variant is alive — no forward vintage record exists unless we make it.

## Long-term (wiki candidates — proposed to the CEO 2026-07-26, not yet written)

- **sharpen-only-what-persists**: a favourite-longshot correction inside a recurring
  ranking cohort is conditional on the underlying ranking being persistent; measure it on
  the resolution variable's own archive, not on prices.
- **published-ci-vs-printed, mirror image**: quoting a *pooled* statistic for a
  sub-population that belongs to a 3–4× more volatile regime is the same error in a new
  costume.
- **Break-even win rate as a promotion gate**: report `q*`, `q` observed, and the 95% lower
  bound on `q`; refuse when the bound is below `q*`. "16/16 and still uninvestable" is
  cleaner than any t-statistic.
- Header/stamped-date vintage pinning as a general archive technique.
