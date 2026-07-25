# Wiki

Durable, **cross-strategy** knowledge only. Run-specific notes stay in memories; market-
specific notes stay in variant folders. The market researcher maintains this wiki —
promoting long-term memory bullets and retirement post-mortems that generalize, merging
overlaps, pruning stale pages. One concept per file.

## Selection

- [market-selection.md](market-selection.md) — where edge is findable (and where it isn't)

## Reference (base rates, biases, methods)

- [reference/sharp-line-screen.md](reference/sharp-line-screen.md) — if a bookmaker prices it, check their line FIRST; cheapest kill we have
- [reference/checkpoint-artifact.md](reference/checkpoint-artifact.md) — an unpriced board manufactures edge; gate on leg-sum, and if your null model wins, audit the checkpoint
- [reference/phantom-midpoints.md](reference/phantom-midpoints.md) — a dead book quotes 0.05/0.95 and the API calls it 0.51; pooling those fabricated +14pp of fake edge
- [reference/midpoint-is-not-a-fill.md](reference/midpoint-is-not-a-fill.md) — our own first batch beat the market 21/21 and was reachable 2/21; always report the fillable count next to the Brier

- [reference/favorite-longshot-bias.md](reference/favorite-longshot-bias.md) — tails rich, favorites shaved inside bucket families
- [reference/recurring-crowd-calibration.md](reference/recurring-crowd-calibration.md) — cheap test: is this recurring market's crowd already calibrated?
- [reference/delayed-execution-test.md](reference/delayed-execution-test.md) — re-run intraday backtests with t+15min fills; "is the edge inside the first 3 minutes?"
- [reference/published-ci-vs-printed.md](reference/published-ci-vs-printed.md) — a source's error bars describe the latent quantity; the market resolves on the printed number
- [reference/venue-resolution-epsilon.md](reference/venue-resolution-epsilon.md) — the venue can resolve Yes on a feed near-miss; never sell inside the epsilon
- [reference/first-print-vintages.md](reference/first-print-vintages.md) — index markets resolve on the FIRST print; reconstruct vintages or corrupt the backtest
- [reference/thin-market-price-read.md](reference/thin-market-price-read.md) — when the midpoint is an artifact; tape, wallets, sibling curves
- [reference/wash-trading.md](reference/wash-trading.md) — detecting fake volume; what to trust instead of the mid

## Recipes (copy-paste code patterns)

- [recipes/polymarket-api.md](recipes/polymarket-api.md) — Gamma / CLOB / Data API endpoints, deep-history pagination, **taker-fee formula by category**, gotchas, Rust snippets

_Seeded 2026-07-22 from the poly experiment's scored findings; everything else starts
clean._
