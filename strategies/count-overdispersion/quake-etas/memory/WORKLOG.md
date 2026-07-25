# count-overdispersion/quake-etas — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-25 — day 1 (trial start). Model: **opus-5 (max effort)**

Ran the CEO's day-1 order in the order given. Outcome: **thesis falsified, kill
recommended** (`roles/ceo/inbox/2026-07-25-quake-etas-kill-recommendation.md`).

**Data pulled and frozen** (`data/quake-etas-data-2026-07-25.tar.gz.r2.json`, 43.6 MB,
uploaded to R2 before this commit):
- USGS FDSN M4.5+, 1990-01-01 → 2026-07-25, chunked by year — **228,900 events** (62,162 at
  M≥5.0).
- All **48** Polymarket earthquake-count ladder boards (both weekly series, discovered via
  `series_id` 11837 / 10844 plus paged `public-search`), **348 legs**, CLOB `prices-history`
  at fidelity 10 for every leg (0 empty), live books for the 3 open boards.
- ComCat version history for **503** threshold-adjacent events. Note: the plain
  `fdsnws/event/1/query?eventid=…&format=geojson` returns only current origins — you need
  **`&includesuperseded=true`** or you get a biased n≈45 sample with the wrong sign.

**Step 1 — screens re-verified cheaply.** Sharp line absent (Pinnacle guest API: 63 sports,
none seismic; Smarkets v3 has no seismicity event type) — stated in STRATEGY.md together
with its consequence: our cheapest falsifier does not exist here, so the modelling gates
carry the load. Phantom-midpoint split reproduced independently: **0/270 dead legs**, 0
near-flat, median total variation 4.78. Both pass; neither killed the idea.

**Step 2 — GATE 3 FIRST. FAIL, and not as anticipated.** Before building ETAS I de-vigged
every resolved board at window-open and compared the *market's own implied distribution* to
the empirical marginal and to Poisson. **Market implied Fano 1.362 (M6.5+) vs empirical
1.358 vs Poisson 1.001.** The crowd is already pricing the overdispersed distribution. Net
of the 0.05 taker fee, a 2c adverse delayed fill and the 3c floor: **fundable ≥3c legs
+0.0091/share (se 0.0340, t=+0.27)**; sub-3c wings **−0.0368/share (t=−10.9)**. Threshold
was 3c/share. Separately, the fundability question does bite — five of seven M6.5+ legs
quote 0.1–3.1c.

**Step 2b — traced the idea's headline.** The reported +0.110 (se 0.046, 17/22) reproduces
only when the checkpoint is anchored to the board's **creation**, 3–5 days before the window
opens, where the mean leg-sum is 1.43 (M6.5+) / 1.97 (M5.5+). At that anchor **plain Poisson
beats the market by +0.179, t = 2.02** — a test on which the null hypothesis wins by two
sigma is measuring an unpriced book, not a crowd error. Re-anchored to window-open (leg-sum
1.028) every model's gain collapses.

**Step 3 — built ETAS properly anyway.** Temporal ETAS, M₀ = 5.0: exact log-likelihood with
compensator (thread-parallel, 200-day kernel truncation), Nelder–Mead MLE, b-value by
Aki–Utsu, 240-draw parameter posterior from the curvature at the MLE, background thinning +
Omori-weighted offspring of pre-window history + recursive G-R/Omori branching, **~10⁶
simulated windows per board**, and the **non-optional magnitude-revision layer** calibrated
from ComCat superseded origins (at event+48h, 29.1% of threshold-adjacent events carry a
different reported magnitude, 17.5% by ≥0.1, mean −0.015, sd 0.110). Fit: μ = 1.176/day,
K = 0.0329, α = 0.615, c = 0.0096 d, p = 0.949, b = 1.0606; 7-day branching ratio 0.484.
Validation: simulated M6.5+ weekly **Fano 1.38 vs observed 1.40** (mean 0.958 vs 0.889). It
under-clusters at M5.5+ (2.97 vs 4.21) with a +17% rate bias — the known cost of fitting a
*temporal* ETAS to a globally aggregated catalogue.

**Step 4 — ETAS vs the crude benchmark, out-of-sample, both reported.**
- Gate 1 (physics, **n = 602 weeks**, ETAS fitted strictly pre-2015): ETAS − empirical
  marginal = **−0.091** (M5.5+, t = −4.9) / **−0.003** (M6.5+, t = −0.5). Needed ≥ +0.05.
- Gate 2 (market, **n = 24 M6.5+ boards**): ETAS − market = **−0.070** (t = −1.14, wins
  9/24); the crude empirical benchmark scores −0.023. Needed ≥ +0.110 with t ≥ 2.
- Gate 0 (with the revision layer): **M6.5+ 30/30**, M5.5+ 10/15, every miss off by exactly
  one and each explained by a threshold-crossing revision. Passes its kill criterion.
- Why, physically: lag-1 R² of the global weekly count is **0.0055** (M6.5+) / 0.0198
  (M5.5+). The overdispersion is within-window burstiness; at window-open Wednesday's
  mainshock has not happened yet, for us or for the crowd. **This bounds the whole family at
  window-open, not just this variant.**

**Step 5/6 — deliverables.** `results/backtest-2026-07-25.md`; STRATEGY.md rewritten with
the method as-built and the falsification; two `applications/*.toml`, both `active = false`
with the full gate record. **No prediction rows** — the model is measured worse than the
market at the checkpoint it would trade, and the next window-open vehicle
(`july-27-august-2`) currently quotes 3–23c spreads with leg-sum 1.237 and a monotonicity
violation (`3` at 0.145 above `2` at 0.101), i.e. a placeholder book. Emitting rows against
those midpoints would inject a fake market baseline into scoring.

**Engineering notes.** Rust throughout (`src/main.rs`, subcommands); data collection by
`curl` per the wiki recipe. Two traps worth remembering: (a) the ETAS likelihood is
O(N × events-in-kernel) — parallelise it or a fit takes an hour on one core; (b) the MLE is
cached in `$DATA/etas_mle*.json`, delete to refit.

**Wiki candidates** (written up in `results/` §9, not graduated without the CEO's word):
the fresh-board checkpoint artifact; overdispersion ≠ mispricing (compute the market's
implied Fano first); persistence-vs-burstiness as the conditioning ceiling.
