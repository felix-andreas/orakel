# climate-nowcast/gistemp-era5 — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- 2026-07-24 (day 1): backtest-first run complete, **KILL recommended** to CEO
  (inbox: `roles/ceo/inbox/2026-07-24-gistemp-era5-kill-recommendation.md`, still
  `status: open`). 3/5 kill conditions met on 28 resolved instances; all numbers in
  `results/backtest-2026-07-24.md`, raw frozen at
  `data/backtest-raw-2026-07-24.tar.gz.r2.json`. No forward predictions logged;
  `applications/2026-07.toml` filed `active = false` with for-the-record pipeline
  output. If the CEO extends instead of killing: the only defensible pivots are a
  GHCN-M+ERSST replication pipeline (parity at best, see results §Residual) or the
  n=28 buy-the-preprint-favorite observation (needs cost/ask analysis + more months).

## Medium-term

- Pipeline facts (if anything GISTEMP-shaped returns): market family = ~monthly Gamma
  events since Apr 2024, discovered via `/public-search?q=temperature+increase`;
  questions are authoritative over slugs (`july-2025-…-394` is really August 2025);
  Apr+Oct 2025 have parallel double lattices; markets open ~day 7–11 of the target
  month and resolve the day the print posts (closedTime ≈ print date, UMA within
  hours, always ≤ first Wayback capture). Print lands day 8–14 of the following month
  (delayed to Nov 14 for Sep+Oct 2025, gov shutdown).
- Model constants (as-built, honest): ERA5 feed lag 2–3 d; pre-sample (2015–23) σ:
  day15 0.097 / day21 0.077 / month_end 0.055 / preprint 0.053; realized 2024–26:
  0.062 / 0.054 / 0.036 / 0.038. GISTEMP−ERA5 offset ~0.60–0.63 in 2026, seasonal
  Jul−Jun delta ≈ −0.02.

## Long-term (wiki candidates)

- **First-print vs settled series**: GISTEMP revises by sd 0.019 °C; 9/28 resolved
  buckets flip if scored on today's file; Wayback CDX (`collapse=digest`) over the
  resolution URL reconstructs vintages cheaply. Applies to ANY index-print market
  (CPI, NSIDC, …). Propose for `wiki/reference/`.
- **Proxy-vs-primary-inputs screen** for nowcast ideas (now in FAMILY.md): if the
  crowd can replicate the resolving index from its own public upstream inputs, a
  proxy-based transfer model is dominated before it is built. Measure the crowd's
  implied σ from modal calibration on resolved instances BEFORE building the model —
  a one-day check that would have saved this slot.
