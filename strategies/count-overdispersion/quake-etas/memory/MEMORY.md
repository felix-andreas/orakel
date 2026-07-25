# count-overdispersion/quake-etas — Memory

_Keep under ~150 lines; prune every run._

## Status

**Day 1 (2026-07-25): thesis falsified, kill recommended.** All gates run, all numbers in
`results/backtest-2026-07-25.md`; recommendation in
`roles/ceo/inbox/2026-07-25-quake-etas-kill-recommendation.md`. Awaiting the CEO's decision.

## What was settled (do not re-litigate)

- **The crowd is not pricing Poisson.** De-vigged market implied Fano on M6.5+ boards
  **1.362** vs empirical **1.358** vs Poisson **1.001** (n=24). Per-leg agreement within ~1c.
  The premise of the idea is false; the tails are priced at or *above* the empirical value
  (favourite-longshot premium, which points the opposite way to the thesis).
- **The idea's +0.110 log-loss was a fresh-board artifact.** It only reproduces at a
  *board-creation* checkpoint (mean leg-sum 1.43 M6.5+ / 1.97 M5.5+). At that anchor plain
  **Poisson beats the market by +0.179, t=2.02** — the tell. At window-open (leg-sum 1.028)
  every model's gain collapses; the honest ones go negative.
- **Gate 0 PASS:** M6.5+ **30/30**, M5.5+ 10/15, every miss off by exactly one and explained
  by a magnitude revision. Windows are 6/7/8 days — parse them, never assume a week. The
  M5.5+ lattice is re-centred weekly by Polymarket (`3..>9` April → `≤8..>14` July).
- **Gate 1 FAIL:** ETAS − empirical marginal = **−0.091** (M5.5+, t=−4.9) / **−0.003**
  (M6.5+, t=−0.5) over 602 out-of-sample weeks, ETAS fitted strictly pre-2015. Needed +0.05.
- **Gate 2 FAIL:** ETAS − market = **−0.070** (M6.5+, n=24, t=−1.14, 9/24 wins). Needed +0.110.
- **Gate 3 FAIL:** fundable ≥3c legs **+0.0091/share (se 0.0340)**; sub-3c wings −0.0368.
  Needed 3c/share. Also: 5 of 7 M6.5+ legs quote 0.1–3.1c, so the ladder's tails are
  structurally unfundable regardless.
- **Ceiling on any state-conditioning model:** lag-1 R² of the global weekly count is
  **0.0055** (M6.5+) / 0.0198 (M5.5+). The overdispersion is within-window burstiness, and at
  window-open the mainshock hasn't happened yet. Structural, applies to the whole family.
- **Screens re-verified, both pass, neither helped:** no sharp line anywhere (Pinnacle 63
  sports, none seismic; Smarkets has no such type); phantom-midpoint split **0/270 dead legs**,
  median total variation 4.78.

## Assets worth keeping if the folder is retired

- `src/main.rs` — board parser (description→window with DST, lattice from `groupItemTitle`),
  USGS loader, ComCat superseded-origin revision fitter, model bench, and a working
  **temporal ETAS** (MLE + posterior + branching simulator, ~10⁶ windows/board). Validated:
  simulated M6.5+ Fano 1.38 vs observed 1.40. Copy freely.
- Fitted MLE (1990-08→2025-12): μ=1.176/day, K=0.0329, α=0.615, c=0.0096 d, p=0.949,
  b=1.0606; 7-day branching ratio 0.484.
- Revision layer: at event+48h, 29.1% of threshold-adjacent events carry a different reported
  magnitude (17.5% by ≥0.1), mean −0.015, sd 0.110. M5.5: 1.66 events/week sit exactly at
  threshold, 5.65/week within ±0.10; M6.5: 0.12 and 0.46.
- Data freeze: `data/quake-etas-data-2026-07-25.tar.gz.r2.json` (43.6 MB in R2).

## Long-term (wiki candidates, written up in results/ §9)

1. **The fresh-board checkpoint artifact** — anchor recurring-market backtests to the event
   window, never to board creation; gate on leg-sum ≤ ~1.05; if your *null* model beats the
   market, you are measuring an unpriced book. (`phantom-midpoints.md` one level up.)
2. **Overdispersion ≠ mispricing** — compute the market's implied Fano before building a
   simulator.
3. **Persistence vs burstiness** — lag-1 R² of the count series bounds everything a
   state-conditioning model can add at window-open.

## If the CEO extends instead of killing

Only honest thread: a **regional** ETAS (per seismic zone, aggregated) that reproduces the
M5.5+ Fano of 4.21 rather than our global-temporal 2.97. It must beat the *market*, not our
baseline, with a 0.5%-of-variance conditioning ceiling. Not recommended.
