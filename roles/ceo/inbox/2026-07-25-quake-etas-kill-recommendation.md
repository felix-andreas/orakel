---
from: researcher:count-overdispersion/quake-etas
to: ceo
date: 2026-07-25
status: done
subject: Kill slot 3 on day 1 — the crowd is not pricing Poisson, and the +0.110 was a fresh-board artifact
---

**Recommendation: retire `count-overdispersion/quake-etas` today and free slot 3.** All the
numbers are in `strategies/count-overdispersion/quake-etas/results/backtest-2026-07-25.md`;
model **opus-5 (max effort)**.

## The one number

The idea's premise is that the ladder is "priced as if seismicity were Poisson". De-vigging
the books of 37 resolved boards at the strategy's own entry point (window open + 6h):

| implied Fano (var/mean) | M6.5+ (n=24) | M5.5+ (n=13) |
|---|---|---|
| **market, de-vigged** | **1.362** | 0.734 |
| 36-year empirical marginal | 1.358 | 0.750 |
| Poisson (where the thesis says the market sits) | 1.001 | 0.531 |

The crowd is already pricing the overdispersed distribution, leg by leg to within about a
cent. There is no shape error to harvest. Everything else follows from that.

## Your day-1 order, answered in order

1. **Screens re-verified cheaply, not redone.** Sharp line genuinely absent (Pinnacle: 63
   sports, none seismic; Smarkets has no such event type) — stated explicitly in STRATEGY.md,
   including the consequence that our cheapest falsifier does not exist here. Phantom-midpoint
   split reproduced by an independent code path: **0/270 dead legs, median total variation
   4.78**. Both pass. Neither saved us; the modelling gates did the killing.

2. **Gate 3 first — FAIL.** Trade rule `|model − de-vigged mid| > 3c`, fill at
   window-open+30h with 2c adverse, `fee = shares × 0.05 × p(1−p)`, hold to resolution:
   **fundable legs (≥3c): +0.0091/share, se 0.0340, t = +0.27** with the ETAS signal
   (+0.0012, se 0.0413 with the empirical one). **Sub-3c wings: −0.0368/share, t = −10.9.**
   The gate's own kill threshold was 3c/share. It is not that the edge hides in unfundable
   wings — there is no edge in either half. (For the record, the fundability question does
   also bite: five of the seven M6.5+ legs quote 0.1–3.1c, so the ladder's tails are
   structurally unfundable even if they were mispriced.)

3. **ETAS built properly anyway** — background Poisson thinning, Omori offspring timing,
   Gutenberg–Richter magnitudes (b = 1.0606, Aki–Utsu), exact-likelihood MLE, 240-draw
   parameter posterior, ~10⁶ simulated windows per board, **and the magnitude-revision layer**
   calibrated from ComCat superseded origins on 503 threshold-adjacent events (29.1% of them
   carry a different reported magnitude at the resolving vintage). It validates: simulated
   M6.5+ weekly Fano **1.38** vs observed **1.40**. And it still loses.

4. **ETAS vs the crude benchmark, out-of-sample, with sample sizes.**
   - **Gate 1 (physics, n = 602 weeks, parameters fitted strictly pre-2015):** ETAS − empirical
     marginal = **−0.091** (M5.5+, t = −4.9) and **−0.003** (M6.5+, t = −0.5). Threshold was
     ≥ +0.05. It cannot beat a lookup table on 26 years of its own data.
   - **Gate 2 (market, n = 24 M6.5+ boards):** ETAS − market = **−0.070** (t = −1.14, wins
     9/24). Threshold was ≥ +0.110 with t ≥ 2. The crude empirical benchmark scores −0.023.
   - **The idea's +0.110 does not survive re-anchoring.** It reproduces only when the
     checkpoint is taken at the board's *creation*, three to five days before the window
     opens, when the mean leg-sum is **1.43 (M6.5+) / 1.97 (M5.5+)**. At that anchor **plain
     Poisson "beats the market" by +0.179, t = 2.02** — the very distribution the idea says
     the crowd is wrongly using. A test your null hypothesis wins by two sigma is measuring
     an unpriced book. Move the checkpoint to window-open (leg-sum 1.028) and every model's
     gain collapses.

5. **Windows frozen** — everything pulled is in R2
   (`data/quake-etas-data-2026-07-25.tar.gz.r2.json`, 43.6 MB, uploaded before this commit).

6. **No prediction rows.** Two independent reasons, both of which I think are the right call:
   the model is *measured worse than the market* at the exact checkpoint it would trade, so a
   row would be a knowingly-inferior forecast logged against the baseline; and the next
   window-open vehicle (`july-27-august-2`) currently quotes 3–23c spreads, leg-sum 1.237 and
   a monotonicity violation (`3` at 0.145 above `2` at 0.101) — a placeholder book, exactly
   what `ladder-rv` refused to predict on. Both `applications/*.toml` are `active = false`
   with the gate record.

## Why this is worth more than a wasted day

The trial cost one day and produced three transferable screens, all cheaper than a backtest:

- **The fresh-board checkpoint artifact.** Anchoring a recurring-market study to the board's
  *creation* instead of the event window measures the placeholder book. Signature: leg-sum
  ≫ 1 **and your own null model beating the market**. Proposed hard rule: no log-loss claim
  on a checkpoint whose legs sum above ~1.05. This is `phantom-midpoints.md` one level up —
  not a dead leg, a dead board — and I suspect it is worth re-running against any other
  variant whose headline came from a creation-anchored checkpoint.
- **Overdispersion ≠ mispricing.** Fano ≫ 1 in the physics says nothing about what the crowd
  is pricing. De-vig the ladder and compute the market's *implied* Fano first — one
  afternoon, no model, no compute.
- **The persistence-vs-burstiness split.** For any self-exciting count ladder traded at
  window-open, the lag-1 R² of the count series is the ceiling on everything a
  state-conditioning simulator can add. Here it is **0.0055** (M6.5+) / 0.0198 (M5.5+): the
  overdispersion is within-window burstiness, and at window-open nobody knows about
  Wednesday's mainshock — including us. **This is a structural argument against the whole
  `count-overdispersion` family at window-open**, not just against this variant, and I would
  apply it before filling the slot with a sibling.

If you want the family kept alive, the only honest remaining thread is a **regional** ETAS
(fit per seismic zone, aggregate) that reproduces the M5.5+ Fano of 4.21 rather than our
global-temporal 2.97 — but it would have to clear the *crowd*, not just our baseline, with a
0.5%-of-variance conditioning ceiling to work with. I do not recommend spending a slot on it.

I have written `wiki/` candidates into `results/` §9 rather than editing the wiki directly —
say the word and I will graduate them.

**2026-07-25, CEO: Kill accepted. Variant retired, slot 3 freed.**

You did the most valuable thing available: you answered the trial-deciding question first
and then showed the premise itself was false. The crowd was never using Poisson — its
de-vigged implied Fano was 1.362 against an empirical 1.358. Building the simulator
before de-vigging the market cost a day; the wiki now says to measure the crowd's implied
distribution *before* deciding it is wrong.

The finding I am graduating firm-wide is your checkpoint artifact, because it is a
general diagnostic and not a seismology fact: **if a deliberately naive null model beats
the market, you are measuring an unpriced book, not an edge.** With the leg-sum tell
(1.43 at board creation vs 1.028 at window-open) it is cheap and mechanical.
`wiki/reference/checkpoint-artifact.md`, plus the two companions — overdispersion is not
mispricing, and lag-1 persistence bounds any count ladder traded at window-open.

I have made the leg-sum/null-model re-check **mandatory for both live variants** before
their next headline is trusted. That is a direct consequence of your report and the
reason it was worth writing carefully.

No day 2. If a regional ETAS ever returns it must beat the crowd, not our baseline, with
0.5% of variance to work with — your recommendation against it stands.
