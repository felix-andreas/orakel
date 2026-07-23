---
date: 2026-07-23
slug: temp-daily-max-truncation-lag
status: backlog
example_markets: ["highest-temperature-in-london-on-july-23-2026", "highest-temperature-in-hong-kong-on-july-23-2026", "highest-temperature-in-los-angeles-on-july-23-2026"]
---

## Thesis

Polymarket runs daily "Highest temperature in <city>" bucket families for **49 cities**
(11 legs each, negRisk, ~$20k–$320k volume per family-day, ~$3.2M open across the 3
listed days in today's scan). The resolution variable is a **monotone running statistic
of a free public feed**: one named station's observed daily max (Wunderground's
airport-METAR history page for most cities; the Hong Kong Observatory's "Absolute Daily
Max" for HK), in whole °C (2°F buckets for US cities). Every intraday observation is a
hard floor — a leg below the running max is a mathematical zero — and after the local
afternoon peak the upside tail dies on diurnal physics, not news.

Two groups are on the wrong side intraday:

1. **Forecast-anchored retail** prices the consumer-app headline high for "the city",
   not the resolution station. The station is load-bearing fine print: HK resolves on
   HKO (not the VHHH airport METAR that weather apps surface); US legs sit on a °F
   lattice reachable only via the METAR T-group's 0.1°C precision, while non-US legs
   round to whole °C. Bucket-boundary errors from station bias + rounding are
   systematic, not noise.
2. **Spread-out market makers**: ~1,400 live legs across 49 cities × 3 open days, with
   METAR prints landing every 30 minutes somewhere. Nobody re-quotes every tail leg
   within minutes of every print; stale quotes linger on legs the running max has
   already killed.

Live evidence from today's scan: Seoul at 17:20 local (post-peak) had fully converged
(30°C+ leg 0.9995 — end-state efficient). But Hong Kong at 16:40 local, cooling well
underway, still had **34°C bid at 0.016** and 35°C asked at 0.005 — cents of near-dead
premium sitting on the bid side. The strategy: monitor station feeds in real time,
sell/fade legs that the running max or the post-peak profile has structurally killed,
and buy the truncation-implied favorite when a thin family lags the latest print. Same
mechanism, ~49 shots per day, scored within 24h.

## Example market(s)

Scanned 2026-07-23 ~08:30 UTC (books via Gamma, negRisk families, all resolve today):

- **highest-temperature-in-london-on-july-23-2026** — $141k family volume, resolution:
  Wunderground EGLC (London City Airport) daily max, whole °C. At 09:20 local
  (pre-peak): 26°C 0.49/0.51 (mid 0.50), 25°C 0.21/0.22, 27°C 0.17/0.20, 24°C
  0.072/0.080, 23°C 0.013/0.014. Live-leg spreads 0.8–3c.
- **highest-temperature-in-hong-kong-on-july-23-2026** — $217k family volume,
  resolution: HK Observatory "Absolute Daily Max" (NOT the airport). At 16:40 local
  (post-peak): 33°C 0.980/0.984, **34°C 0.016/0.020**, 35°C 0.001/0.005; VHHH METAR had
  printed 33°C for the prior 5 hours.
- **highest-temperature-in-los-angeles-on-july-23-2026** — $51k family volume, 2°F
  buckets: 80–81°F 0.53/0.56, 78–79°F 0.20/0.21, 82–83°F 0.14/0.16, 84–85°F
  0.030/0.042.

Universe today: 49 cities, 1,464 open legs, top families Seoul $317k / HK $270k /
London $180k / LA $174k over 3 open days. Scan freeze:
`roles/market-researcher/data/scans/2026-07-23-events-vol24.csv.r2.json`.

## Falsification sketch

Resolved instances exist at daily cadence back months (spot-checked: London June 10 and
Seoul April 15 both closed) — thousands of city-day families with per-leg CLOB price
history. Pull leg histories (`prices-history` with explicit `startTs`, fine fidelity)
plus hourly station obs (IEM ASOS archive for METAR stations; HKO archive for HK), then:

1. **Window-open calibration** (wiki recipe): Herfindahl vs mean winner open price over
   ≥100 city-days. Expected result: calibrated → no window-open edge; that alone does
   NOT kill the idea, it scopes it to intraday.
2. **Dead-leg lag**: at each obs timestamp compute the running max; legs strictly below
   it are dead. Measure median time-to-≤1c and the bid-side premium actually printed on
   dead legs afterward. **Kill if** dead legs collapse within ~10 min median AND
   post-death sellable premium averages <2c, or top-of-book on those legs is <$50
   (midpoint artifact per `wiki/reference/thin-market-price-read.md`) — that means the
   weather bots already own this.
3. **Truncated-model vs market**: station-specific diurnal climatology + same-day
   forecast anchor, truncated at the running max; log-loss vs de-vigged family mids at
   local 10:00/12:00/14:00/16:00. **Kill if** the model's edge is less than half the
   touch spread on the legs it would trade.
4. **Integrity checks**: wash-test the tape on 2–3 families before trusting printed
   volume; quantify resolution-revision risk ("revisions considered until next day's
   first datapoint") — kill if revision reversals exceed the premium collected.

Fast scoring: every instance resolves same-day; a 10-day trial sees ~490 family
resolutions.
