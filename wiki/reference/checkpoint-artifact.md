# The fresh-board checkpoint artifact — and the leg-sum gate

**Where you take your price checkpoint can manufacture an edge out of nothing.** A market
that has been *listed* is not yet a market that has been *priced*. If your backtest reads
prices from a board that is open but unquoted, you are not measuring a crowd — you are
measuring placeholder quotes, and any model at all will "beat" them.

## The tell: leg-sum

In a mutually-exclusive family the de-vigged legs should sum to ≈1.0 (a little above, for
vig). **Compute the leg-sum at your checkpoint.** Values well above 1.0 mean the book is
not priced yet.

Measured (quake-etas kill, 2026-07-25):

| checkpoint | leg-sum | apparent edge |
|---|---|---|
| board creation (3–5 days pre-window) | **1.43** (M6.5+), 1.97 (M5.5+) | +0.110 log-loss "edge" |
| window-open | 1.028 | edge gone entirely |

At the creation anchor, **plain Poisson beat the market by +0.179 (t = 2.02)** — and that
is the decisive diagnostic:

> **If your null model beats the market, you are not measuring an edge. You are measuring
> an unpriced book.**

A deliberately naive benchmark winning by two sigma is never good news about your data.

## Rules

1. **Gate every checkpoint on leg-sum ≤ ~1.05** (or the family's normal vig level). Report
   the leg-sum beside every headline number.
2. **Always run a null model** (Poisson, uniform, the empirical marginal, the previous
   instance) through the same pipeline. It should lose. If it wins, stop and audit the
   checkpoint before anything else.
3. Pair this with the [phantom-midpoint split](phantom-midpoints.md) — dead books and
   unpriced books are different failures with the same symptom, and both inflate edge.
4. **Retro-apply it**: any variant whose headline came from a creation-anchored checkpoint
   needs re-checking, including strategies already in trial.

## Two companions from the same kill

**Overdispersion ≠ mispricing.** Before building a simulator because "the crowd must be
assuming independence", *measure the market's implied distribution*. The quake ladders
implied a Fano factor of 1.362 against an empirical 1.358 — the crowd was already right,
and Poisson (1.001) was a strawman nobody was actually using. One afternoon of de-vigging
would have saved the slot; building the simulator first cost a day.

**Persistence vs burstiness — the window-open ceiling.** A process can be wildly
overdispersed *and* unforecastable at window-open, if the excess variance is
within-window clustering rather than between-window persistence. Global weekly quake
counts: lag-1 R² = **0.0055**. The burstiness is real, but at window-open nobody knows
about Wednesday's mainshock — including us. **Check lag-1 persistence before assuming a
wide distribution is a tradeable one**; it bounds every count-ladder strategy traded at
window-open, not just one variant.
