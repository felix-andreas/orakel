# temp-truncation/runningmax

> Thesis (from `ideas/2026-07-23-temp-daily-max-truncation-lag.md`): daily city
> temperature bucket families resolve on a monotone running max of a free public feed
> (station METAR / HKO). Legs below the intraday running max are mathematical zeros;
> post-peak upside legs die on diurnal physics. Forecast-anchored retail prices the
> wrong station (fine print varies per city) and spread-out MMs leave stale tail
> quotes across ~1,400 live legs. Fade structurally dead legs; buy the
> truncation-implied favorite when a thin family lags the latest print.

## Method

DAY-1 STATE — to be established by the trial researcher, backtest-first per the idea's
falsification sketch (window-open Herfindahl, dead-leg lag stats, truncated-model
log-loss vs de-vigged mids, wash/integrity checks). This section must always reflect
the current method.

## Applicability

A market fits when: it is one leg of a daily city max-temperature negRisk family whose
resolution source is a named station with a free real-time feed (Wunderground METAR
page / HKO), and the family has live books (top-of-book > $50 on traded legs).
Onboarding = an `applications/<market>.toml` naming the city, station id, feed URL,
bucket lattice (°C vs 2°F), and local peak-hours window.

## How to run

(to be written with the first working scripts in `src/`)

## Evidence

- (backtests land here as `results/…`)

## Changelog

- 2026-07-23 — variant created from the idea; slot 1 trial started.
