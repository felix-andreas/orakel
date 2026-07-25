# A published CI is about the latent quantity, not the printed number

When a source publishes an estimate **with error bars** — Elo/Bradley-Terry scores ± CI,
poll margins of error, index confidence intervals, "rank spread" columns — those bars
describe uncertainty about the **latent quantity being estimated**. But a market that
resolves on that source resolves on the **printed number**, and the printed number is
usually far more persistent than the latent uncertainty implies.

Use the published CI as your σ and you will systematically over-disperse your
distribution, fade the favourite, and lose to a crowd that simply reads the integer.

Measured (arena-rank day-1 kill, 2026-07-25, LMArena ranking boards):

- Mean published 95% CI on top-25 model scores: **±5.9** points.
- Realised sd of the change in the **printed** score over 7 days: **1.23** points.
- Median published "rank spread" width: **15 ranks**. Median realised |Δrank|: **1**;
  only **1.3%** of models finished outside their published spread.

Consequence: a properly-built simulation calibrated on the published bars lost to the
market in 9 of 10 cohort-months (log-loss 1.244 vs 0.504), while *sharpening* the
crowd's own distribution gained +0.111 log-loss out-of-sample.

## Rules

1. **Estimate σ from the realised dynamics of the printed series**, never from the
   publisher's CI — measure how much the printed number actually moves over your horizon.
2. Decompose the residual: if the remaining uncertainty is dominated by **discrete
   arrival events** (new model releases, new entrants, revisions) rather than estimation
   noise, that is private/unpredictable information — see the "select against pure
   news processes" rule in [market-selection](../market-selection.md).
3. If the modelable part is near-deterministic, the crowd almost certainly has it. The
   remaining edge, if any, is in the **shape of the crowd's distribution** (favourite-
   longshot, over/under-dispersion), not in re-estimating the level.

Generalises to: CPI/index confidence intervals, poll margins of error, Elo systems,
any leaderboard that publishes uncertainty about a quantity it also prints.
