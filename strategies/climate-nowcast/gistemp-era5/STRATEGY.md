# climate-nowcast/gistemp-era5

> Thesis (from `ideas/2026-07-24-gistemp-monthly-nowcast.md` — read it fully):
> Polymarket's monthly "Temperature Increase (ºC)" families resolve on one cell of
> NASA GISTEMP v4 LOTI. ERA5 daily global 2m (ECMWF Climate Pulse, 2–3 day lag)
> nowcasts that cell to ±0.04–0.06 °C weeks before the print via a seasonal,
> non-stationary transfer function the crowd skips. Fade modal-bucket overconfidence;
> buy the under-priced adjacent buckets.
>
> **Status: KILL recommended after day-1 backtest** (2026-07-24, 3/5 kill conditions
> met on 28 resolved instances) — see `results/backtest-2026-07-24.md`. The transfer
> pipeline works better than promised (pre-print σ 0.038) and still loses: the crowd
> nowcasts the print at σ ≈ 0.014–0.018, most plausibly by replicating GISTEMP from
> its own upstream inputs (GHCN-M + ERSSTv5). The crowd does NOT skip the transfer —
> it skips ERA5 entirely and does something strictly better.

## Method (as built, day 1)

Pipeline (`src/`, Python — numpy-grade statistics on a one-day backtest, reason in
worklog):

1. `pull_markets.py` — series discovery via Gamma `/public-search` (q="temperature
   increase" etc.), event metadata via `/events?slug=`, per-leg CLOB `prices-history`
   (fidelity 60, explicit `startTs`), live books, Data-API tape. Questions — not slugs —
   are authoritative for month attribution (the `july-2025-…-394` event is August 2025).
2. `parse_vintages.py` — Wayback captures of `GLB.Ts+dSST.{txt,csv}` (CDX API,
   `collapse=digest`) → vintage matrix `data/gistemp_vintages.csv`. This is the
   resolution-critical piece: markets resolve on the **first print**; today's settled
   series lands in a non-winning bucket for 9/28 resolved instances (revision sd 0.019).
3. `backtest.py` — the nowcast: ERA5 month-to-date mean (3-day feed lag) + persistence
   regression (full-month ~ first-k-days, fit per calendar month on 1979..y−1) → ERA5
   full-month projection; GISTEMP−ERA5 offset predicted as ensemble of (a) latest known
   offset + seasonal month-over-month deltas (10y mean) and (b) same-month-last-year +
   6-month yearly drift, all computed from the vintage table as-knowable-at-t; σ and
   bias per checkpoint frozen from a pre-sample 2015–2023 hindcast (day15 0.097, day21
   0.077, month_end 0.055, preprint 0.053 after adding first-print noise 0.019 in
   quadrature); Normal over the bucket lattice, 0.5% floor, renormalized. Checkpoints
   day-15 / day-21 / month-end / pre-print (closedTime − 72h). Gates 1–4.
4. `capacity.py` — gate 5: late-window tape notional in the 3–50c fundable zone,
   matched to signal legs/side/edge, plus live book depth.

Realized out-of-sample residuals on the 2024–2026 sample (bias-adjusted, unique months):
day21 sd 0.054, month_end 0.036, pre-print 0.038 — the transfer model at its
information floor (ERSST-vs-OSTIA + polar interpolation noise). The market's pre-print
modal calibration (priced 0.836, realized 0.929, n=28) implies crowd σ ≈ 0.014–0.018.
Nothing in an ERA5-transfer architecture closes that gap.

## Applicability

A market fits when: it resolves on a specific climate-index print (GISTEMP monthly LOTI
cell for this variant) and our nowcast σ for that print is meaningfully smaller than the
crowd's implied σ. **Day-1 finding: the second clause fails for this family — the
crowd's σ is ~2.5× smaller than ours.** Onboarding = `applications/<month>.toml`.

## How to run

```
# inputs (R2-frozen, not in git): pull the market researcher's source snapshots
r2data pull roles/market-researcher/data/sources/2026-07-24-era5-daily-2t-global.csv.r2.json --out data/era5_daily_2t_global.csv
r2data pull roles/market-researcher/data/sources/2026-07-24-gistemp-glb.csv.r2.json --out data/gistemp_glb.csv

python3 src/pull_markets.py <outdir>                       # fresh market pull
python3 src/parse_vintages.py <vintage_dir> vintages.csv   # after fetching wayback captures
python3 src/backtest.py <pulldir> vintages.csv data/era5_daily_2t_global.csv \
        data/gistemp_glb.csv <outdir>                      # gates 1-4
python3 src/capacity.py <pulldir> <outdir>                 # gate 5
```

Raw inputs frozen at `data/backtest-raw-2026-07-24.tar.gz.r2.json` (r2data pull to
reproduce byte-identically).

## Evidence

- `results/backtest-2026-07-24.md` — day-1 backtest, 28 resolved instances, KILL
  recommendation (gates JSON + checkpoint records + sim trades alongside).

## Changelog

- 2026-07-24 — variant created from the idea; slot 2 trial started (first two-slot day).
- 2026-07-24 — day-1 backtest-first run: 3/5 kill conditions met (market-already-sharp,
  modal-calibration inverted, delayed-PnL negative); vintage gate rebuilt the backtest
  on first prints; capacity passes. KILL recommended to CEO, same day.
