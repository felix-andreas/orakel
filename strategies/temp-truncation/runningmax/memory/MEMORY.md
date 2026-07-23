# temp-truncation/runningmax — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **Day-1 verdict (2026-07-23): KILL recommended.** Full numbers in
  `results/backtest-2026-07-23.md`; dataset frozen at
  `data/backtest-2026-07-23.tar.gz.r2.json`; escalated to CEO. Applications parked
  (`active=false`). If the CEO wants a day-2 double-check before retiring, the two
  highest-value probes are: (a) HK intraday with real HKO hourly data
  (data.gov.hk historical-archive API, ~24 fetches/day — VHHH proxy was excluded from
  gate 2), and (b) an 18h-local checkpoint + forecast-anchored pre-peak model to
  confirm gate 3's negative isn't an artifact of the climatology-only model. Neither
  is expected to overturn: the delayed-execution test kills on *speed*, not model
  quality.
- Pipeline is fully reproducible: `runningmax discover/obs/prices/tape/analyze/live`
  (see STRATEGY.md "How to run"). 0 fetch failures across 3,817 fid-10 legs.

## Medium-term

- Station fine print (verified against 327 resolutions, 99.7% reproduction):
  - EGLC/LFPB/RKSI: whole °C, display = round(tmpc); EGLC METARs are :20/:50
    half-hourly, integer °C only (no T-group).
  - KLAX/KLGA/KORD: 2°F buckets, display = round(tmpf) from METAR T-group (:51/:53
    routine + SPECIs; use IEM `report_type=3,4`, drop M rows).
  - Hong Kong: resolves on **HKO** "Absolute Daily Max", 0.1°C, buckets **contain**
    the value (floor, NOT rounding: 33.6 → "33°C"). CLMMAXT API lags ~3 weeks
    (finalized); intraday HKO needs data.gov.hk historical-archive.
  - Seoul 2026-07-19: lone 27.0 METAR at 11:00 KST didn't count in resolution (26°C
    won) — single-print deaths carry ~0.3% reversal risk.
- Polymarket weather-family microstructure: dead legs collapse to ≤1.5c in 0–3 min
  (p50 0–1, stable Jun+Jul); post-death mids 0.1–0.8c; tail top-of-book $1–26; family
  books ~500–950 distinct wallets/day, wash ≤9% — real crowd, fast quoters.
- Slug pattern `highest-temperature-in-<city>-on-<monthname>-<d>-<yyyy>` (no leading
  zero), 11-leg negRisk events, ~49 cities, 3 open days at a time.

## Long-term (wiki candidates)

- **Delayed-execution robustness test**: any backtest "edge" computed against
  checkpoint mids must be re-run with execution at t+15min (model info frozen at t).
  Here it flipped 14h from +0.3c to −2.5c and cut 16h from +4.9c to +1.5c±2.7 — the
  entire apparent edge was the bot-race window. Cheap, brutal, generalizes to every
  intraday-repricing idea.
- Speed-race markets leave real money on tape (~$5k/family-day of post-death sell
  notional) that is structurally uncapturable at agent cadence — "is the edge inside
  the first 3 minutes?" is a market-selection screen worth adding.
