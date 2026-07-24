# climate-nowcast/gistemp-era5

> Thesis (from `ideas/2026-07-24-gistemp-monthly-nowcast.md` — read it fully):
> Polymarket's monthly "Temperature Increase (ºC)" families resolve on one cell of
> NASA GISTEMP v4 LOTI. ERA5 daily global 2m (ECMWF Climate Pulse, 2–3 day lag)
> nowcasts that cell to ±0.05–0.06 °C weeks before the print via a seasonal,
> non-stationary transfer function the crowd skips. Fade modal-bucket overconfidence;
> buy the under-priced adjacent buckets. Edge horizon 8–19 days; the resolving print
> IS the resolution — nothing to race.

## Method

DAY-1 STATE — to be established backtest-first per the idea's 5 kill conditions
(market-already-sharp log-loss test on resolved instances, modal-calibration test,
t+24h delayed PnL, GISTEMP first-print vintage check, capacity floor). ≥20 resolved
monthly instances back to May 2024 exist. Keep this section as-built.

## Applicability

A market fits when: it resolves on a specific climate-index print (GISTEMP monthly
LOTI cell for this variant) and our nowcast σ for that print is meaningfully smaller
than the crowd's implied σ. Onboarding = `applications/<month>.toml` (event slug,
bucket lattice, resolution source cell, print ETA). The annual "hottest year rank"
market is a free secondary application of the same pipeline.

## How to run

(to be written with the first working scripts in `src/`)

## Evidence

- (backtests land here as `results/…`)

## Changelog

- 2026-07-24 — variant created from the idea; slot 2 trial started (first two-slot day).
