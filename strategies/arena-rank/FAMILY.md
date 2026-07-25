# arena-rank

> **In plain English:** Bets on a public leaderboard that ranks AI models by how often people prefer their answers. One big market and several small ones are all settled by reading the same table at the same moment.

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

Cross-variant lessons (from `satellites` day 1, 2026-07-25 — full numbers in
`satellites/results/backtest-2026-07-25.md`):

- **The edge mechanism above is falsified.** The joint order-statistic simulation,
  anchor-calibrated leave-one-month-out on the deep #1 board, loses to the de-vigged
  satellite market at every checkpoint (log-loss 1.244 vs 0.504; better in 1/10
  cohort-months). The portfolio correction adds nothing the anchor wants. Any future
  variant in this family must start from that, not from the thesis paragraph above.
- **Do not use the published ±CI or Rank Spread as σ.** They describe uncertainty about
  *latent skill*; the boards resolve on the *printed* number, whose realised movement is
  ~2.5× tighter (top-25 models: sd 1.2 at 7d vs mean published 95% CI ±5.9; median Rank
  Spread width 15 ranks vs median realised |Δrank| of 1). Using them fades favourites and
  loses.
- **Read the resolving URL out of the rules text.** `arena.ai/leaderboard/text` (default)
  is style control **ON**; boards saying "style control off" resolve on
  `text/overall-no-style-control`, a different ordering. The family has been slugged four
  ways and rebranded `lmarena.ai` → `arena.ai` mid-life.
- **Vintages are obtainable** for the whole family life via Wayback across *both* hosts,
  pinned by the data date each page stamps on itself — 47/47 exact resolution
  reproductions. Archive the live tables daily anyway; nothing else will exist forward.
- **What is alive here is not modelling, it is shrinkage**: the satellite crowds are
  underconfident in their own favourite by 6–9c, which is the favourite-longshot bias
  larger than the 1–3c the wiki expects, on books quoting 0.1–3.7c.
