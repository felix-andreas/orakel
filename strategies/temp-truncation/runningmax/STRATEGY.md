# temp-truncation/runningmax

> Thesis (from `ideas/2026-07-23-temp-daily-max-truncation-lag.md`): daily city
> temperature bucket families resolve on a monotone running max of a free public feed
> (station METAR / HKO). Legs below the intraday running max are mathematical zeros;
> post-peak upside legs die on diurnal physics. Forecast-anchored retail prices the
> wrong station (fine print varies per city) and spread-out MMs leave stale tail
> quotes across ~1,400 live legs. Fade structurally dead legs; buy the
> truncation-implied favorite when a thin family lags the latest print.

**STATUS 2026-07-23: KILL RECOMMENDED after day-1 backtest** — see
`results/backtest-2026-07-23.md`. Gates 2 and 3 of the idea's falsification sketch
triggered on 347 resolved families: dead legs collapse in 0–3 minutes (stable across
June and July), post-death premium is 0.1–0.8c median with dust top-of-book, and the
truncated model's paper edge vanishes under a 15-minute execution delay (+1.5c ± 2.7c,
sign-flipping across sample halves). Awaiting CEO decision; applications parked
(`active = false`).

## Method (day-1 state, as built)

Pipeline (`src/main.rs`, one crate, subcommands):

1. **discover** — Gamma `/events?slug=highest-temperature-in-<city>-on-<month>-<d>-<yyyy>`
   per city-day; legs (11 buckets) with condition/token ids, winner from collapsed
   `outcomePrices`.
2. **obs** — IEM ASOS (`asos.py`, `report_type=3,4`, UTC) per station; HKO CLMMAXT for
   Hong Kong finalized daily max. Station fine print per city is load-bearing:
   EGLC/LFPB/RKSI whole °C (display = round(tmpc)); KLAX/KLGA/KORD 2°F lattice via
   METAR T-group (display = round(tmpf)); HK resolves on **HKO** (VHHH only as
   intraday proxy) at 0.1°C with **floor** buckets (the range *contains* the value).
3. **prices** — CLOB `prices-history` per leg with explicit `startTs` (wiki gotcha),
   fid=10 for checkpoints, fid=1 for collapse timing.
4. **model** — per station, climatology of residuals r(h) = final daily max − running
   max at local hour h (leave-one-out), Gaussian kernel σ=0.45°C/0.8°F, truncated at
   the running max (sub-truncation mass to the running-max bucket), floored 0.002 and
   renormalized over the family lattice. For HK the residual is vs the VHHH proxy, so
   it folds in the HKO−VHHH station bias.
5. **analyze** — gate 1 window-open Herfindahl calibration; gate 2 dead-leg collapse +
   post-death premium + tape-verified sellable prints; gate 3 log-loss + trading sim
   vs de-vigged fid-10 mids at local 10/12/14/16h, with a delayed-execution (t+15min)
   robustness variant; gate 4 wash tests; gate 0 resolution reproduction (99.7%).
6. **live** — today's family books (best bid/ask = last array elements), live IEM
   runmax, model probabilities, prediction-row CSV.

## Applicability

A market fits when: it is one leg of a daily city max-temperature negRisk family whose
resolution source is a named station with a free real-time feed (Wunderground METAR
page / HKO), and the family has live books (top-of-book > $50 on traded legs).
Onboarding = an `applications/<market>.toml` naming the city, station id, feed URL,
bucket lattice (°C vs 2°F), and local peak-hours window.
**Empirical caveat from day 1:** the top-of-book condition fails in practice on
exactly the legs the strategy wants to trade (tails quote $1–26).

## How to run

```
cargo build --release   # in this folder
runningmax discover <data_dir> 2026-06-01 2026-07-20 london,paris,seoul,hong-kong,los-angeles,nyc,chicago
runningmax obs      <data_dir> 2026-05-01 2026-07-23
runningmax prices   <data_dir> 10            # all legs; add fam list for fid=1 subset
runningmax tape     <data_dir> london_2026-07-16 ...
runningmax analyze  <data_dir>               # gates -> stdout + <data_dir>/out/*.csv
runningmax live     <data_dir> 2026-07-23 london,nyc,los-angeles
```

Data freeze for the day-1 run: `data/backtest-2026-07-23.tar.gz.r2.json`.

## Evidence

- `results/backtest-2026-07-23.md` — day-1 backtest, 347 families, all four gates +
  resolution-reproduction check; kill recommendation with numbers.

## Changelog

- 2026-07-23 — variant created from the idea; slot 1 trial started.
- 2026-07-23 — day-1 backtest complete (347 families, 7 cities): gates 2 & 3 kill
  conditions met; KILL recommended to CEO; applications parked.
