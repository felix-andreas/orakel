# Market Researcher Worklog

One dated entry per run. Name the exact model id that did the work.

---

## 2026-07-25 (run 2) — extra cycle after the arena kill; esports series-shape idea filed

Model: **opus-5 (xhigh)**. Felix requested a second cycle on the day
`arena-rank/satellites` was falsified and rebadged as `arena-rank/favourite-shrinkage`.

- Onboarded on the four screens that now gate idea quality, incl. the new
  `wiki/reference/published-ci-vs-printed.md`, and on my own post-mortem
  (`strategies/arena-rank/satellites/results/backtest-2026-07-25.md`). Two corrections
  internalised: verify the *exact resolving object* and state how; classify the claim
  **level vs shape** before filing. Both are now standing memory entries.
- Fresh scan: 2,000 open events by `volume24hr`. Filtered to multi-leg, fast-resolving,
  non-weather/non-crypto-ladder/non-arena families. Rejected on the spot: company
  market-cap ranking boards (glanceable resolution variable). Parked with reasons:
  non-US central-bank decision boards (rates-desk incumbent), US primary-election winner
  boards (good, but election-calendar cadence). Pursued: **esports BO3 bundles**.
- Built the resolved universe from scratch: **16,959 resolved esports events**
  (Dec-2025 → Jul-2026) via date-windowed offset paging — plain offset paging hard-caps
  at 2,000 and `/events/keyset` ignores `cursor` (the param is `after_cursor`). Yielded
  **6,710 BO3 triples** (`moneyline` + `map_handicap` + `totals`) with known outcomes;
  the three legs are arithmetically consistent in **6,705/6,710** (99.93%).
- Fetched per-leg CLOB history (6,375/6,375 tokens non-empty) and the taker tape for 300
  events. Headline measurements at T−1h: moneyline favourite 0.727→**0.788**; handicap
  0.531→**0.683**; Over-2.5 overpriced in **8/8 cohort-months** (−9.0pp, se 1.75,
  t=−5.16) including the months where the moneyline bias was ≈0. Convex transfer: a
  +5.6pp moneyline error becomes a **+13.8pp** handicap error in the 0.80–0.90 band.
- Ran the kill-screens *before* filing, not after: speed screen (handicap mid moves
  0.519→0.537 across the whole 24h pre-match window vs a 0.683 realised rate — no print
  to race); delayed execution (T−6h signal, T−15m fill, +2c adverse = **+14.7c**, se 2.0);
  midpoint-vs-tape (taker BUY VWAP only +0.68c above mid, so the mid is executable);
  selection-artifact guard (reproduces on a **disjoint** low-derivative-volume sample:
  ML +7.0pp, Over −10.0pp); incumbent test by tournament tier (bias is *larger* on
  tier-1 — fan money, not information).
- Quantified the cost stack properly: Polymarket's **taker fee = shares × rate × p(1−p)**,
  sports rate 0.05 ⇒ ~1.2c/share in the fundable band, makers free. Graduated to
  `wiki/recipes/polymarket-api.md` along with the pagination facts and `sportsMarketType`
  typing — both are cross-strategy, not idea-specific.
- Filed `ideas/2026-07-25-esports-series-shape-2.md` (backlog): explicit **shape** claim,
  all four screens answered upfront with evidence, three live example markets with
  today's book numbers, six gates with numeric kill thresholds, and a pre-registered live
  stop. Honest open item recorded as gate 5: I could **not** read an external bookmaker
  line from this box (hltv.org 403 through the proxy, the-odds-api needs a paid key), and
  that is the cheapest way to kill the whole thesis.
- Standing caution written into the idea: an edge this large on 1c-spread books should
  not exist, so gate 0 is an artifact hunt, not a confirmation.

---

## 2026-07-25 — Netflix killed in research; arena-rank-satellites filed

Model: **opus-5 (max)** — claude-opus-5. First run under the 2026-07-24 routing change
(Opus 5 everywhere, max effort for this role; fable retired).

- Onboarded on both kills + the three screens. Fresh scan (20 pages, 26,668 open market
  rows). Backlog was empty, slot 2 free.
- **Candidate 1, Netflix weekly Top-10 — measured hard, killed before filing.** Structure
  looked close to ideal: 8 boards/week, 243 resolved instances, precise fine print
  (global boards are **"English only"**; US boards resolve on country rank, global on
  views), and — verified — *every* official Netflix publication lands before the market
  opens, with nothing further until the resolving print, so Mon–Tue trades a frozen,
  unpublished outcome. Netflix hands out complete free ground truth
  (`all-weeks-{global,countries}.tsv`, 264 weeks × 94 countries). Measured on resolved
  instances: crowd modal leg underpriced at all 7 checkpoints (Wed 0.651→won 0.750;
  Mon-frozen 0.920→won 0.971, n=102), field legs in the 3–50c zone priced 0.08–0.12 vs
  0.02–0.07 realized, taker flow ~$160k/wk arriving 71–88% in the frozen window,
  "Other" wins 4.5%. Two independent kill findings: (a) the executable side is missing —
  bids are $0–96 top-of-book while tail *asks* are deep, so the sell-the-field direction
  cannot be filled; (b) decisive — a decay model fitted on Netflix's own 264 weeks scores
  **23% (shows) / 42% (films) argmax vs the market's 77% / 83%** at Thursday. The crowd
  has an observation channel we don't (in-app daily Top 10; FlixPatrol 403 from this box).
  Same shape as the gistemp kill, caught for one day of research instead of a slot.
- **Candidate 2, filed: `ideas/2026-07-25-arena-rank-satellites.md`.** Monthly
  arena.ai/LMArena family — 7+ boards resolving off ONE Text-Arena Rank column read at
  ONE instant, liquidity spanning 250× ($30,299 WebDev → $7,587,062 #1-overall), spreads
  spanning 37×. Thesis inverts the deep-book rule: the efficient $7.6M board is a free
  sharp anchor on the same latent ranking that seven thin satellites price separately.
  Two mechanical mispricings: company boards are order statistics over *portfolios*
  (Anthropic holds ranks 1,2,3,4,6 today), and the Rank column is an estimate whose
  publisher stamps every row with ±CI, vote count, Preliminary flag and an explicit Rank
  Spread (rank 1 → "1–5", rank 10 → "4–27"). Screen 2 passes structurally: the upstream
  is a private vote stream, so *nobody* can hold a better feed — the published table is
  the primary for every participant, and a cache-busted fetch confirmed we hold the same
  Jul-21 vintage. Live incoherence evidence: WebDev-Aug legs sum 1.222 vs Math-Aug 0.934;
  Moonshot priced 0.001 / 0.017 / 0.182 / 0.662 across four boards on one ranking; the
  $655k Chinese board has Alibaba 0.786 vs Moonshot 0.182 while the table has Moonshot
  11 points ahead. 6 kill gates incl. gate-0 resolution reproduction, refresh-lifetime
  speed gate, paired log-loss, a portfolio-effect gate, t+24h delayed exec, capacity.
  Whole July cohort checks 2026-07-31 → a trial scores in six days, then monthly.
- Screen 3 has real teeth here and is written into the file: the leaderboard is
  *recomputed retroactively* at each refresh, so today's table resolved nothing; Wayback
  has 500 captures 2025-05-28→2026-01-28 and **zero after**, so Feb–Jul 2026 instances
  are unscoreable and we must archive the live table ourselves from day 1.
- Wiki maintenance: new SELECT AGAINST bullet in `wiki/market-selection.md` —
  **"glanceable within-window state"**, the sibling of proxy-vs-primary, earned by the
  Netflix numbers, with the corollary on where to look instead (state hidden from the
  amateur too). Memory pruned and restructured; API gotchas recorded (prices-history
  `interval=max` 30-day cap, placeholder legs pinned at 0.500, Wayback CDX needs https).

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
