# arena-rank/favourite-shrinkage — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **MANDATORY (CEO, 2026-07-25), alongside the phantom-midpoint split: leg-sum /
  null-model re-check.** The FLB result is measured at T-30/14/7d checkpoints on
  multi-leg cohorts — exactly the setup where a not-yet-priced board inflates edge. Report
  the de-vigged leg-sum at each checkpoint, gate to <= ~1.05, and run a naive null through
  the same pipeline; it must lose. `wiki/reference/checkpoint-artifact.md`.

- **MANDATORY NEXT RUN (CEO, 2026-07-25), before any other work — the phantom-midpoint
  split.** Today's `series-shape/bo3-derivatives` kill found that Polymarket reports a
  ~0.50 midpoint for legs with NO resting orders (bid 0.05 / ask 0.95), and pooling those
  fabricated +14pp of edge in a family whose live-book edge was 0.0 +/- 1.5pp. Our claim
  is structurally similar: a distributional edge measured against Polymarket midpoints.
  Split the FLB result by whether each board's book actually moved pre-check (and by
  spread/depth), and report the LIVE-BOOK number as the headline. See
  `wiki/reference/phantom-midpoints.md`. Our books are tight (0.1-3.7c, $8-12k) so this
  should pass — do it anyway; it is nearly free and it is exactly what caught the other
  variant.
- **State plainly in STRATEGY.md that no sharp-line screen exists for this family** — no
  bookmaker or exchange prices LMArena rankings. That absence is a reason to expect an
  edge to survive, but it also removes our cheapest falsifier, so the remaining gates
  carry more weight (`wiki/reference/sharp-line-screen.md`).

- Created 2026-07-25 from the `satellites` day-1 kill. **Day-2 duties (from the day-1
  report, all still valid):** (1) ARCHIVE the live tables daily — all six resolving
  slices; no forward vintage record exists unless we make it, and a refresh can land on
  the check morning; (2) produce SHRINKAGE-based prediction rows for the July cohort
  (the CEO deliberately did NOT log the retired simulation's rows — see decision);
  (3) re-price at T−3d/T−1d, the Chinese board (Alibaba 1476 vs Moonshot 1473, both
  Preliminary) can flip on one refresh; (4) grade at the 2026-07-31 12:00 ET check.
- **Day-3 (2026-07-27) PRE-REGISTERED KILL TEST**: does the FLB gain concentrate in a
  fundable 0.60–0.90 favourite band? If it lives only at 0.93–0.99, retire.
- Sub-arena boards (math/coding/webdev/agent) have ZERO resolved history — do not lean
  on them for evidence. webdev fails the book gate ($454 depth); coding flow ~$390/mo.

## Medium-term

- Resolving slice = `arena.ai/leaderboard/text/overall-no-style-control` (SC **off**).
  The default `/text` page has SC **on** and orders differently (kimi-k3 #10 vs #13).
  Reading the wrong table was the founding idea's flagship error.
- Wayback: CDX must be called over **https** (http form refused with a misleading
  allowlist error); `…/web/<ts>id_/<url>` returns raw gzip; site rebranded
  lmarena.ai → arena.ai (captures continue under the new host — 103 captures of the
  resolving slice, 2025-08 → 2026-07). Pages stamp their own data date, and refreshes
  are ~weekly → a capture need not sit at the check instant.

## Long-term (wiki candidates)

- Published CI ≠ σ of the printed number (now `wiki/reference/published-ci-vs-printed.md`).
- Header/stamped-date vintage pinning as a general archive technique.
