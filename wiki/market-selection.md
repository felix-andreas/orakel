# Market selection — where is edge findable?

Strategy ideas and applications should target markets where our comparative advantage —
building calibrated models fast, reading fine print carefully, and re-forecasting daily —
can beat the marginal trader. Distilled from poly's scored runs (2026-07).

## Select FOR

1. **Simulation-tractable generating processes** — counting processes (tweet counts),
   brackets/schedules (tournaments), physical time series (weather), mechanical indices
   (CPI rounding). If the resolution variable can be Monte-Carlo'd from public data, we
   can be calibrated where the crowd vibes.
2. **Thin-to-mid liquidity** (~$10k–$1M real volume). Deep books are efficient: poly's
   $40M BTC market converged to our number on its own — confirmation, not edge.
3. **Structural fine print** — rounding buckets, tie rules, revision policies, sibling
   families with coherence constraints. Careful reading beats casual traders. (poly's
   France market resolved on a tie-break every casual trader missed: model the tie-break,
   don't reduce "most goals" to "clear the leader".)
4. **Fast resolution** (days–weeks). Scoring is what compounds; a Dec-31 market teaches
   nothing until winter. For *trials* this is near-mandatory.
5. **Genuine uncertainty** (roughly 15–85% prices) — a 1% market is usually right for
   boring reasons.
6. **Stale or salience-anchored crowds** — sim-tractability alone isn't enough; mature
   recurring series with weekly-scored crowds are efficient at window-open (see
   [recurring-crowd-calibration](reference/recurring-crowd-calibration.md)). Look for
   prices that *haven't repriced* on fresh data, or that ran on narrative (poly's
   hottest-year market climbed 3 straight days on El-Niño headlines relevant to the
   *following* year — the fade was the biggest edge on the book).

## Select AGAINST

- **Speed-race mispricings** — if the inefficiency closes within minutes of a public
  print, bots own it and agent cadence structurally cannot compete. Measure before
  trialing: how fast does the target mispricing actually close on resolved instances?
  (temp-truncation kill, 2026-07-23: dead legs collapsed in 0–3 min; see
  [delayed-execution-test](reference/delayed-execution-test.md).)
- Deep, professionally-arbitraged books (major crypto levels, headline politics).
- Pure insider processes (M&A rumor resolution) — the decisive information is private;
  only worth it with a structural angle (e.g. wash-trade detection changing the read).
- Markets whose resolution source we can't access read-only.
- Calibrated recurring crowds at window-open — run the calibration test before spending
  a slot.

## The test

> Can a well-calibrated model, buildable from public data, know something this market's
> marginal trader doesn't — and will we find out (via resolution) fast enough to learn?
