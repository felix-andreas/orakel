# arena-rank

Price Polymarket's monthly LMArena-ranking boards as **one latent object**: seven-plus
boards (#1/#2/#3 overall, Math/Coding/WebDev sub-arenas, style-control, "best Chinese
company") all resolve off a single reading of the arena Rank column at one instant. The
deep #1 board ($7.6M) is an efficient, free anchor on the same latent ranking that the
satellite boards ($30k–$300k, 10–250× thinner, 3–37× wider spreads) price in isolation.

Edge mechanism: simulate the ranking from the leaderboard's own published uncertainty
(Bradley-Terry scores ± CI, vote counts, preliminary flags, published rank spreads),
calibrate the simulation to reproduce the deep board's implied distribution, then price
the satellites by arithmetic. Two things satellite crowds miss: the boards are order
statistics over *company portfolios* (a company holding ranks 1-4 is a max over
correlated scores), and the Rank integer is an estimate the publisher itself stamps
±5 to ±23 places.

Born from `ideas/2026-07-25-arena-rank-satellites.md`. Selected because the
sharpest-incumbent screen comes back empty: no desk prices arena Elo, no derivative
exists, so our comparative advantage (building calibrated pipelines fast) is not
dominated by a better-equipped incumbent — the failure mode that killed
`climate-nowcast/gistemp-era5`.

Variants:

- [`satellites/`](satellites/) — anchor-calibrated order-statistic simulation across the
  monthly board cohort (trial, slot 2, started 2026-07-25).

Cross-variant lessons: (none yet)
