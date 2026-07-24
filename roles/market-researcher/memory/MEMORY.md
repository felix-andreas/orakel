# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-24: filed `ideas/2026-07-24-gistemp-monthly-nowcast.md` per Felix's
  market-specific-data directive (inbox msg now status: done). ERA5-daily nowcast of
  the GISTEMP cell that resolves Polymarket's monthly "Temperature Increase (ºC)"
  family + ranking siblings. Next run: check CEO pickup; if trialed, first derisks =
  (1) CLOB prices-history on resolved instances (does the crowd already track ERA5?),
  (2) GISTEMP first-print vintages via web archive (revision risk to backtest truth).
  Watch: August 2026 instance should list early Aug (July listed Jun 8); the $108k
  "July 1st hottest?" sibling (bid 0.94 vs model 0.68–0.82) resolves ~Aug 8–12 — a
  cheap live scoring event for the model even without a slot.
- 2026-07-23 recap: run 1 filed temp-truncation (trialed AND killed same day — speed
  race; lessons → `wiki/reference/delayed-execution-test.md`). Run 2 filed
  hit-price-ladder-rv → now trialing as `barrier-touch/ladder-rv` (slot 1, sell-only
  method, review due 2026-08-02). Backlog candidate esports/LoL weeklies still
  unfiled, sharps risk unassessed.

## Medium-term

- Scan tool `roles/market-researcher/tools/scan/` (Gamma /events → CSV + summary).
  20 pages ≈ 26.6k open market rows, ~1 min. Order `volume24hr` for "alive today".
  Series discovery: Gamma `/public-search?q=<text>&limit_per_type=20` finds all
  instances of a recurring family incl. resolved (added to wiki recipe 2026-07-24).
- Landscape shape (stable 07-23→07-24): Sports ~10.3k mkts dominates count;
  Politics/Elections dominate volume ($2.4B/$1.5B). ~87% of open markets <$10k
  volume; ~11k of 26.6k resolve within 7 days — fast-resolving supply plentiful.
- **Climate/GISTEMP family** (2026-07-24): monthly "«Month» Temperature Increase (ºC)"
  resolves on GISTEMP LOTI `GLB.Ts+dSST.txt` row/col cell (0.01 °C, 1951-80 base);
  ≥20 resolved instances back to May 2024, $0.4–4.9M lifetime volume each (current
  July only $40k live-to-date); trades PAST month-end until the print (~day 8–12 of
  next month; Jul-2025 closedTime Aug 8). ERA5 Climate Pulse daily CSV
  (`sites.ecmwf.int/data/climatepulse/...era5_daily_series_2t_global.csv`, 2–3 day
  lag) nowcasts it; GISTEMP−ERA5 offset is non-stationary (2026 ~0.06 hot vs 2015-25
  July mean) + month-seasonal. My day-21 hindcast: σ 0.056 °C, bias +0.025, n=30.
  Both sources verified read-only from this box; snapshots frozen 2026-07-24 to R2
  (`data/sources/`).
- Other primary-source markets seen under the climate tag (future market-specific
  ideas): NSIDC arctic sea-ice min ($62k, Oct 1), VEI-6 volcano ($121k), Cat-4 US
  hurricane landfall ($334k, NHC), natural-disaster umbrella ($227k). Also EIA/AAA
  gas-price, CDC counts, Netflix Top-10 weeklies (all seen in scans, unprobed).
- Weather city-dailies: 49 cities × 11-leg daily temp families, $20k–320k/family-day;
  per-city resolution-station fine print. Category is bot-patrolled intraday (kill
  evidence) — only pre-day/forecast-based angles remain plausible there.
- "Hit Price" one-touch family: 64 events / ~750 mkts / ~$88M; 3 tiers; resolution =
  Pyth/Binance 1-min candles; KEY TRAP: mid-window strike additions carry private
  window starts (`startDate`), board titles lie. Depth lives in 3–50c zone of
  monthlies. Now trialing as barrier-touch/ladder-rv — don't re-propose.
- Gamma quirks: use `/events` not `/markets` for tags; dedupe by event slug across
  pages; `endDate` conventions vary by family (temp dailies 12:00Z; monthly climate
  keeps trading past endDate until print). Nightly rust NOT installed — stable crate
  workflow.

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — see `/wiki/market-selection.md`. Deep books are efficient;
  calibrated recurring crowds are efficient at window-open.
- Idea-shaping heuristics that produced filed ideas: (1) take a wiki caveat that says
  "X is itself a strategy-shaped idea" and find the category where X's preconditions
  are strongest; (2) post-kill: sort mispricings by what reveals them — public print →
  bot food; model-run → agent-harvestable (speed screen upfront, in the idea file);
  (3) directive-driven (2026-07-24): start from the *resolution source*, not the
  market — find primary feeds that determine the outcome days early (GISTEMP/ERA5
  pattern: a second feed nowcasts the resolving feed), then check who's on the wrong
  side of the mapping between them. Mode-overconfidence in bucket families is the
  recurring wrong side: crowd gets the modal bucket right and the variance wrong.
