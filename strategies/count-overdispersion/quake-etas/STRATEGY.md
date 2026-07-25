# count-overdispersion/quake-etas

> **Status after day 1 (2026-07-25): the thesis is falsified. Kill recommended — see
> `results/backtest-2026-07-25.md` and `roles/ceo/inbox/2026-07-25-quake-etas-kill-recommendation.md`.**
>
> Original thesis (from `ideas/2026-07-25-quake-ladder-overdispersion-3.md`): weekly USGS
> seismicity-count ladders are priced against an implicitly **Poisson** distribution, while
> the true process is self-exciting (M5.5+ weekly Fano 4.2–5.1), so both tail buckets are
> ~1.6× too cheap and the middle up to 1.6× too rich — a pure **shape** claim traded once at
> window-open.
>
> **What the data says.** The market's own de-vigged distribution has an implied Fano of
> **1.362** on M6.5+ boards against an empirical **1.358** and a Poisson **1.001**. The crowd
> is already pricing the overdispersed distribution, to within about a cent on every leg. The
> premise of the idea is false, and everything downstream of it fails: a full ETAS simulation
> is **0.070 log-loss worse** than the market at window-open (needed +0.110), and **loses to
> a plain lookup table** on 602 out-of-sample weeks of the catalogue itself.

## Sharp-line screen — explicitly absent, and that mattered

No bookmaker or exchange prices global weekly earthquake counts: Pinnacle's public guest API
lists 63 sports, none seismic; Smarkets v3 has no such event type. Re-verified 2026-07-25.
Per `wiki/reference/sharp-line-screen.md` the absence of a professional counterparty is one
of the few good reasons to expect an edge to survive — **but it also removes our cheapest
falsifier, so gates 0–4 carry the entire load.** That is exactly how this trial went: the
idea passed the screen that killed three candidates this week and then died to the modelling
gates. The lesson is not "the screen was wrong" — it is that *no counterparty* buys you no
information about whether the crowd is wrong, only about whether someone else is right.

## Method — as built

Rust crate, `src/main.rs`, subcommands `boards | gate0 | revision | revfit | models |
ceiling | gate3 | etas {validate,physics,physics2015,score}`.

1. **Board model.** Every board's window is parsed from its own description (ET → UTC with
   2026 DST) — windows are 6, 7 or 8 days depending on the board, and the M5.5+ bucket
   lattice is re-centred by Polymarket week to week (`3..>9` in April, `≤8..>14` in July).
   Legs are parsed from `groupItemTitle` into `[lo, hi]` count intervals.
2. **Catalogue.** USGS FDSN, M4.5+, 1990–2026, chunked by year: 228,900 events (62,162 at
   M≥5.0). Counting is restricted to `type == earthquake`.
3. **Magnitude-revision layer.** ComCat `origin` products fetched with
   `includesuperseded=true`; the magnitude reported at time *T* is the highest
   `preferredWeight` origin with `updateTime ≤ T`. Calibrated on 503 threshold-adjacent
   events: at the resolving vintage (event + 48h) the reported magnitude differs from
   today's final one in **29.1%** of cases (17.5% by ≥0.1), mean −0.015, sd 0.110.
4. **ETAS.** Temporal conditional intensity
   `λ(t) = μ + Σ_{tᵢ<t} K·10^{α(Mᵢ−M₀)}·(t−tᵢ+c)^{−p}`, M₀ = 5.0. Exact log-likelihood with
   compensator (parallelised, 180–200 day kernel truncation), Nelder–Mead MLE, b-value by
   Aki–Utsu with 0.1 binning. Parameter posterior = 240 draws from the curvature at the MLE.
   Simulation: background Poisson thinning → offspring of pre-window history sampled by
   Omori-integral weights → recursive Gutenberg–Richter/Omori branching inside the window →
   magnitudes perturbed by the revision layer, binned to 0.1, thresholded. **~10⁶ simulated
   windows per board.**
   Fit: μ = 1.176/day, K = 0.0329, α = 0.615, c = 0.0096 d, p = 0.949, b = 1.0606;
   branching ratio 0.484 over a 7-day window (stable).
5. **Scoring.** De-vigged CLOB midpoints at the strategy's own entry point (window
   open + 6h), paired log-loss per board, plus a full cost stack (delayed fill at +30h, 2c
   adverse, `fee = shares × 0.05 × p(1−p)`).

**Validation the engine passes:** simulated weekly M6.5+ counts have mean 0.958 / var 1.321 /
**Fano 1.38** against observed 0.889 / 1.244 / **1.40**. It under-clusters at M5.5+
(Fano 2.97 vs 4.21) and carries a +17% rate bias there — a temporal ETAS fitted to a
*globally aggregated* catalogue is a superposition of many regional processes and flattens
the Omori tail. That is the one genuine modelling gap left (see `results/` §8).

## Applicability

A market fits when: it is a bucket ladder over a count of clustered events; the resolving
catalogue is public and reproducible; the book is live at the entry point; and — the
condition this variant proves is *not* automatic — **the crowd's implied distribution is
actually Poisson-shaped**. Check that before anything else: de-vig the ladder and compute
its implied Fano factor. It costs an afternoon and no model.

## How to run

```bash
cargo build --release
D=/path/to/data-dir                      # outside git; frozen in data/*.r2.json
./target/release/quakeetas boards   $D   # parse all boards, windows, lattices
./target/release/quakeetas gate0    $D   # reproduce every resolution from USGS
./target/release/quakeetas revfit   $D   # calibrate the magnitude-revision layer
./target/release/quakeetas ceiling  $D   # lag-1 R^2 of weekly counts = conditioning ceiling
./target/release/quakeetas models   $D 6 open     # every count model vs the market
./target/release/quakeetas models   $D 6 create   # the same, anchored to board creation
./target/release/quakeetas gate3    $D   # fee / book / fundability
./target/release/quakeetas etas     $D validate|physics|physics2015|score
```
Data collection is `curl` (see `results/backtest-2026-07-25.md` §0 and the r2 manifest
note); the MLE is cached in `$D/etas_mle*.json` — delete to refit.

## Evidence

- `results/backtest-2026-07-25.md` — gates 0–5, all numbers, out-of-sample.
- Gate 0 **PASS**: M6.5+ 30/30, M5.5+ 10/15 with every miss explained by a magnitude revision.
- Gate 1 **FAIL**: ETAS − empirical marginal = **−0.091** (M5.5+, t=−4.9) / **−0.003**
  (M6.5+, t=−0.5) over 602 out-of-sample weeks. Threshold was ≥ +0.05.
- Gate 2 **FAIL**: ETAS − market = **−0.070** (M6.5+, n=24, t=−1.14, wins 9/24). Threshold
  was ≥ +0.110 with t ≥ 2.
- Gate 3 **FAIL**: net **+0.0091/share (se 0.0340)** in fundable ≥3c legs with the ETAS
  signal, **−0.0368/share** in the sub-3c wings. Threshold was ≥ 3c/share.
- Gate 4 **explains the idea's headline**: the reported +0.110 reproduces only at a
  board-*creation* checkpoint where the legs sum to 1.43–1.97; at that anchor plain **Poisson
  beats the market by +0.179 (t = 2.02)**, which is the artifact's signature.
- Physics ceiling: lag-1 R² of the global weekly count is **0.0055** (M6.5+) / 0.0198
  (M5.5+) — that is all a state-conditioning simulator has to work with.

## Changelog

- 2026-07-25 — created from the idea; slot 3 trial started.
- 2026-07-25 — day 1 (opus-5, max): full gate run. Thesis falsified; kill recommended.
