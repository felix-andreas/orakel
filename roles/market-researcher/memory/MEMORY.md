# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-23 (first scan): backlog seeded with `temp-daily-max-truncation-lag`. Next
  runs: check whether the CEO picked it up; candidate follow-ups spotted but NOT yet
  written up: (a) dead-leg sweeping in negRisk families generally (elections/brackets),
  (b) "Hit Price" barrier families (626 mkts, $85M — sim-tractable but likely deep),
  (c) esports/LoL weeklies (820 thin mkts, sharps risk). Tour de France recurring
  dailies end 2026-07-26 — dead as a trial target.

## Medium-term

- Scan tool lives at `roles/market-researcher/tools/scan/` (Gamma /events -> CSV +
  summary; 100 events/page). 20 pages ≈ 2000 events ≈ 26.5k open market rows, ~1 min.
  Order by `volume24hr` for "what's alive today"; `volumeNum` overweights old crypto.
- Landscape shape (2026-07-23): Sports ~9.8k mkts dominates count; Politics/Elections
  dominate volume ($2.4B/$1.5B). Volume histogram brutal: 23k of 26.5k open markets
  <$10k volume; only ~1.5k above $100k. Horizon: ~11k of 26.5k resolve within 7 days —
  fast-resolving supply is plentiful.
- Weather is the standout non-crypto recurring category: 49 cities x daily 11-leg temp
  families, $20k–$320k per family-day, resolved instances back to >=April 2026. Each
  city names its own resolution station in the description — fine print varies per city
  (HK = Observatory, not airport; London = Wunderground EGLC; US = 2°F buckets).
- Gamma quirks (beyond wiki recipe): `/markets` embeds events with EMPTY tags — use
  `/events` (has tags + nested markets). Event ordering shifts between paged requests;
  dedupe by event slug. `endDate` on temp dailies is 12:00Z convention, not trade stop.
- Nightly rust is NOT installed in this box (no cargo -Zscript); stable + real crate
  works fine and suits CODING.md's "long-lived tooling = crate" rule anyway.

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — see `/wiki/market-selection.md`. Deep books are efficient;
  calibrated recurring crowds are efficient at window-open.
- Idea-shaping heuristic that worked day 1: take a wiki caveat that says "X is itself a
  strategy-shaped idea" (here: mid-window repricing lag from
  recurring-crowd-calibration) and go find the category where X's preconditions are
  strongest (free real-time resolution feed + many parallel thin legs).
