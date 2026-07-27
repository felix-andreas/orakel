# Wiki

Durable, **cross-strategy** knowledge only. Run-specific notes stay in memories; market-
specific notes stay in variant folders. The market researcher maintains this wiki —
promoting long-term memory bullets and retirement post-mortems that generalize, merging
overlaps, pruning stale pages. One concept per file.

## Selection

- [market-selection.md](market-selection.md) — where edge is findable (and where it isn't)

## Reference (base rates, biases, methods)

- [reference/sharp-line-screen.md](reference/sharp-line-screen.md) — if a bookmaker prices it, check their line FIRST; cheapest kill we have. **Kalshi's 12,186-series catalogue with declared settlement URLs is ONE unauthenticated call — run it before anything else.** But it has a blind spot: an empty catalogue slot says no *venue* prices the object, never that it is unforecast — box office died to a free Substack, and the market's implied sigma named the analyst before we found him
- [reference/implied-sigma-names-the-incumbent.md](reference/implied-sigma-names-the-incumbent.md) — run BEFORE the modelling: if the market's implied σ is tighter than your in-sample model, someone published the answer. Box office 0.120 vs 0.171 found a free Substack no venue scan could reach — the third family lost this way
- [reference/checkpoint-artifact.md](reference/checkpoint-artifact.md) — an unpriced board manufactures edge; gate on leg-sum, and if your null model wins, audit the checkpoint
- [reference/phantom-midpoints.md](reference/phantom-midpoints.md) — a dead book quotes 0.05/0.95 and the API calls it 0.51; pooling those fabricated +14pp of fake edge
- [reference/midpoint-is-not-a-fill.md](reference/midpoint-is-not-a-fill.md) — our own first batch beat the market 21/21 and was reachable 2/21; always report the fillable count next to the Brier
- [reference/tape-gate.md](reference/tape-gate.md) — a 1c spread with listed depth and ZERO trades ever; the third way a quote lies, and the one that revised our own 2-of-21 headline
- [reference/stale-feed-gate.md](reference/stale-feed-gate.md) — the FOURTH way a quote misleads, and the only one where OUR number is the broken one: 64 of 95 rows priced off a feed shut for the weekend, while the book repriced 0.475 → 0.715 during the closure
- [reference/break-even-win-rate.md](reference/break-even-win-rate.md) — a band that went 16/16 with t=+10.3 and is still uninvestable; report q*, q and the 95% lower bound, refuse when the bound is under break-even
- [reference/sharpen-only-what-persists.md](reference/sharpen-only-what-persists.md) — a favourite-longshot correction is conditional on the ranking persisting; measure persistence on the resolution variable's own archive, and never quote a pooled statistic across a sub-population

- [reference/favorite-longshot-bias.md](reference/favorite-longshot-bias.md) — tails rich, favorites shaved inside bucket families
- [reference/recurring-crowd-calibration.md](reference/recurring-crowd-calibration.md) — cheap test: is this recurring market's crowd already calibrated?
- [reference/delayed-execution-test.md](reference/delayed-execution-test.md) — re-run intraday backtests with t+15min fills; "is the edge inside the first 3 minutes?"
- [reference/published-ci-vs-printed.md](reference/published-ci-vs-printed.md) — a source's error bars describe the latent quantity; the market resolves on the printed number
- [reference/venue-resolution-epsilon.md](reference/venue-resolution-epsilon.md) — the venue can resolve Yes on a feed near-miss; never sell inside the epsilon
- [reference/first-print-vintages.md](reference/first-print-vintages.md) — index markets resolve on the FIRST print; reconstruct vintages or corrupt the backtest. **Rebuild ≥3 settled instances from the live feed before modelling: one "public machine-readable" feed restated by +247% and got 37% of boards wrong**
- [reference/rounded-threshold-ladders.md](reference/rounded-threshold-ladders.md) — the strike IS a rounding boundary; verified half-up 2,128/2,128, and Python's round() is wrong on exactly the 12 ties that decide the bet
- [reference/thin-market-price-read.md](reference/thin-market-price-read.md) — when the midpoint is an artifact; tape, wallets, sibling curves
- [reference/wash-trading.md](reference/wash-trading.md) — detecting fake volume; what to trust instead of the mid

## Recipes (copy-paste code patterns)

- [recipes/polymarket-api.md](recipes/polymarket-api.md) — Gamma / CLOB / Data API endpoints, deep-history pagination, **taker-fee formula by category**, gotchas, Rust snippets

_Seeded 2026-07-22 from the poly experiment's scored findings; everything else starts
clean._
