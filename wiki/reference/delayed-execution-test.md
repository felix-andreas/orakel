# The delayed-execution robustness test

Any backtest of an *intraday repricing* idea that computes its edge against checkpoint
mids is suspect until it survives execution delay: re-run the trading sim with fills at
**t+15 minutes** while the model's information is frozen at t. If the edge collapses,
you were measuring the speed race, not skill.

Origin (temp-truncation/runningmax kill, 2026-07-23, 347 resolved city-day families):
instant-execution sim showed +4.9c/trade at the 16h checkpoint; with 15-minute delayed
execution it fell to +1.5c ± 2.7c s.e. and sign-flipped between June and July halves.
The 14h checkpoint flipped from +0.3c to −2.5c. Full numbers:
`strategies/temp-truncation/runningmax/results/backtest-2026-07-23.md`.

Rules of thumb:

- Delay ≥ the cadence you can actually achieve (agent runs: 15 min is generous; we're
  really daily).
- Freeze model inputs at t — a delayed fill with updated inputs is a different (faster)
  strategy than the one you can run.
- Report the delayed number with a standard error and a sample split; a sign-flip
  across halves means the residual "edge" is noise.
- Companion selection screen: **"is the edge inside the first 3 minutes?"** — measure
  how fast the mispricing you target actually closes (here: dead legs collapsed to
  ≤1.5c in 0–3 minutes, p95 = 3 min). If the answer is minutes, only speed
  infrastructure can harvest it; skip at agent cadence.
