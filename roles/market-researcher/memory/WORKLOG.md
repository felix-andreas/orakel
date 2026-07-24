# Market Researcher Worklog

One dated entry per run. Name the exact model id that did the work.

---

## 2026-07-24 — Felix directive: market-specific data sources; gistemp idea filed

Model: fable (high). Governing input: inbox
`2026-07-23-felix-market-specific-strategies.md` (now status: done).

- Fresh scan (20 pages, 26,614 open market rows, frozen:
  `data/scans/2026-07-24-events-vol24.csv.r2.json`). Grepped for primary-source
  topics: climate/GISTEMP, hurricanes/NHC, CDC counts, CPI/BLS, EIA gas, Netflix/
  Spotify/Billboard weeklies, earthquakes/USGS, box office. Strongest data-advantage
  candidate by far: the GISTEMP monthly bucket family (poly's literal heritage).
- Verified both primary sources read-only from this box and froze snapshots to R2
  (`data/sources/2026-07-24-{gistemp-glb,era5-daily-2t-global}.csv.r2.json`): GISTEMP
  LOTI monthly CSV (Jun 2026 = 1.18) and ECMWF Climate Pulse ERA5 daily global 2t
  (through Jul 21, updated Jul 23).
- Built the actual nowcast live: ERA5 July MTD(21d) +0.632, persistence-projected
  full month 0.629±0.032; June-anchored + year-effect transfer → GISTEMP nowcast
  center ~1.246 σ 0.056 (30-month day-21 hindcast: MAE 0.052, bias +0.025, only 6/30
  within ±0.025). Market prices modal 1.20–1.24 bucket 72/83 vs model 0.31–0.37;
  adjacent buckets 4.9c/20c asks vs model 0.15–0.24/0.25–0.32; $108k ranking sibling
  bids 0.94 on "1st hottest" vs model 0.68–0.82. Confirmed the family trades past
  month-end until the print (Jul-2025 closedTime Aug 8) → 8–19 day model-revealed
  edge horizon, no print to race; speed screen passes by construction.
- Filed `ideas/2026-07-24-gistemp-monthly-nowcast.md` (backlog): 5 kill conditions
  incl. market-already-sharp test, modal-calibration test, t+24h delayed-exec sim,
  GISTEMP first-print vintage check, capacity floor. Replied to Felix (status: done).
- Wiki maintenance: added Gamma `/public-search` series-discovery recipe to
  `wiki/recipes/polymarket-api.md`. Memory pruned (run-2 detail compressed; climate
  family facts + "start from the resolution source" heuristic added).

---

## 2026-07-23 (run 2) — extra cycle after same-day kill; idea 2 filed

Model: fable (high). Felix requested a second cycle after runningmax was killed day 1.

- Onboarded on the kill: `wiki/reference/delayed-execution-test.md` (new),
  market-selection's new SELECT AGAINST (speed-race mispricings), runningmax memory +
  `results/backtest-2026-07-23.md`. Takeaway operationalized: speed screen now goes
  *in the idea file, upfront*.
- Re-mined the frozen 08:11Z scan (pulled from R2, sha-verified) for the three unfiled
  candidates. Dropped (a) generic negRisk dead-leg sweeping (bot-harvested premium in
  weather; brackets show hours-long windows but dust-sized books). Left (c) esports
  unfiled. Pursued (b): "Hit Price" one-touch ladders — found the family is 3-tier
  (daily/weekly/monthly) across ~25 assets incl. equities+commodities, with weekly
  boards squarely thin-to-mid ($5–80k), not just the deep BTC annuals memory recalled.
- Fresh probes (Gamma + CLOB books + Pyth Hermes, 09:15–09:20Z): live monotonicity
  violations on SPY/NVDA weekly LOW ladders persisting ≥65 min unchanged (vs 0–3 min
  weather collapse); WTI July board implies 53% ATM → 88% wing touch-vol smile;
  discovered the extension-leg trap — strikes added mid-window carry private
  `startDate` windows ("after market creation"), e.g. monthly WTI $80-LOW created
  Jul 20 16:30Z = re-touch claim at 0.25/0.26 while the weekly $80-LOW sits at 1.0.
  Tail top-of-book is dust ($3–20); depth is real in the 3–50c zone of monthlies.
- Idea filed: `ideas/2026-07-23-hit-price-ladder-rv.md` (status: backlog) — IV-anchored
  one-touch relative value + fine-print windows; speed screen passed upfront (edge is
  model-revealed, harvested over 1–9d holds; post-touch convergence explicitly
  excluded); kill gates include measured violation-lifetime screen and t+24h
  delayed-execution sim. Resolved supply verified: WTI May $40.2M/30 legs, BTC June
  $25.2M, SPY+NVDA week-of-Jul-13 boards closed.

---

## 2026-07-23 — first scan, backlog seeded

Model: fable (high). First run since founding; backlog was empty.

- Built `tools/scan/` (stable-rust crate, not cargo -Zscript — nightly absent from this
  box): pages Gamma `/events`, flattens to CSV, prints horizon/volume/tag summary.
- Scanned top 2000 open events by 24h volume -> 26,539 open market rows. Freeze:
  `roles/market-researcher/data/scans/2026-07-23-events-vol24.csv.r2.json` (11MB -> R2).
- Landscape: Sports 9.8k mkts / Politics $2.4B vol; 87% of open markets <$10k volume;
  ~11k markets resolve <=7d. Recurring series = crypto (deep, avoid) + 49-city daily
  temperature bucket families (~$3.2M open, resolved instances back to April).
- Probed temp families live: Seoul post-peak fully converged (0.9995); Hong Kong 16:40
  local still bid 0.016 on a near-dead 34°C leg vs 5h of 33°C METAR prints; London
  pre-peak with genuine spread (26°C at 0.50). Resolution stations differ per city (HKO
  vs Wunderground airport pages) — structural fine print confirmed in descriptions.
- Idea filed: `ideas/2026-07-23-temp-daily-max-truncation-lag.md` (status: backlog) —
  intraday truncation repricing in daily temp families; falsification = window-open
  Herfindahl test + dead-leg collapse-time backtest + truncated-model log-loss vs
  de-vigged mids, on months of resolved city-days.
