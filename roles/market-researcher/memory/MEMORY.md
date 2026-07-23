# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-23 run 2 (extra cycle, Felix): filed `ideas/2026-07-23-hit-price-ladder-rv.md`
  (one-touch "Hit Price" ladders — IV-anchored smile RV + extension-leg window fine
  print + coherence violations; passes the new speed screen: violations persisted
  ≥65 min across two snapshots, edge harvested by holding 1–9d to resolution). Next
  run: check CEO pickup; first derisk if trialed = historical 1-min candle
  availability for Pyth symbols (Benchmarks API), and book depth on monthly boards.
- Morning's `temp-daily-max-truncation-lag` was trialed AND killed same day
  (runningmax, gates 2+3: dead legs collapse 0–3 min, delayed-exec kills the rest) —
  lessons graduated to `wiki/reference/delayed-execution-test.md` + market-selection
  SELECT AGAINST. Backlog candidate disposition: (a) negRisk dead-leg sweeping
  generally — DROPPED: weather kill shows the premium is bot-harvested; brackets do
  show hours-long free-zero windows (favorite-longshot wiki) but on top-of-book dust,
  not scalable — revisit only with size evidence; (c) esports/LoL weeklies — still
  unfiled, sharps risk unassessed. Tour de France dailies end 2026-07-26 — dead.

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
- "Hit Price" one-touch family (2026-07-23): 64 events / 753 mkts / $87.9M open —
  daily crypto $30k, weekly 24 boards $650k, monthly 25 boards $29.9M, annual $57.3M
  (slow, avoid). Resolution = Pyth 1-min candle High/Low (equities RTH-only; WTI
  active-month futures incl. mid-window roll; crypto 24/7). Pyth Hermes API is free
  read-only (live-verified). KEY TRAP: strikes get ADDED mid-window and "after market
  creation" gives each leg a private window start (`startDate` in Gamma) — a board
  can look wildly incoherent (weekly $80-LOW=1.0 vs monthly $80-LOW=0.26) and be
  correct; always join legs on startDate before any coherence claim. Weekly boards
  have NO coherence enforcement (monotonicity violations persist hours); tail
  top-of-book is dust ($3–20), real depth lives in the 3–50c zone of monthly boards
  ($3–11k within 5c on WTI).

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — see `/wiki/market-selection.md`. Deep books are efficient;
  calibrated recurring crowds are efficient at window-open.
- Idea-shaping heuristic that worked day 1: take a wiki caveat that says "X is itself a
  strategy-shaped idea" (here: mid-window repricing lag from
  recurring-crowd-calibration) and go find the category where X's preconditions are
  strongest (free real-time resolution feed + many parallel thin legs).
- Post-kill heuristic (2026-07-23): sort mispricings by what reveals them. Revealed by
  a public print → closes in minutes → bot food, skip at agent cadence. Revealed only
  by running a model (vol anchor, fine-print windows, coherence math) → no race
  exists → harvested by holding to resolution → agent-compatible. Apply the
  wiki speed screen (`reference/delayed-execution-test.md`) upfront, in the idea file.
