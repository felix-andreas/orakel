# The fresh-board checkpoint artifact — and the leg-sum gate

**Where you take your price checkpoint can manufacture an edge out of nothing.** A market
that has been *listed* is not yet a market that has been *priced*. If your backtest reads
prices from a board that is open but unquoted, you are not measuring a crowd — you are
measuring placeholder quotes, and any model at all will "beat" them.

## The tell: leg-sum

In a mutually-exclusive family the de-vigged legs should sum to ≈1.0 (a little above, for
vig). **Compute the leg-sum at your checkpoint.** Values well above 1.0 mean the book is
not priced yet.

> ### First check that your ladder is actually mutually exclusive
>
> **Correction, 2026-07-28 (barrier-touch/ladder-rv).** The rule above is stated for a
> *partition* — "how many quakes", "which bucket does the close land in" — where exactly one
> leg wins and the masses must sum to one. Many ladders are not partitions. A **one-touch**
> ladder is **nested**: "WTI touches 75" implies "WTI touches 80" implies "WTI touches 85".
> Several legs win together, the true sum is whatever the nesting implies, and there is no
> reason for it to be 1.
>
> On such a family `leg-sum ≈ 1` is not a gate that is hard to pass — **it is a gate that
> cannot fail**, because the bucket masses it implicitly refers to sum to 1 by construction
> however badly the book is priced. Running it returns CLEAN and tells you nothing. That is
> worse than having no gate, because you write "leg-sum check clear" in a worklog and
> believe the checkpoint has been audited.
>
> **The quantity that can actually be wrong is the expected winner count.** Sum the legs'
> midpoints at your checkpoint — for a nested ladder that is the market's expected number of
> YES legs, `Σmid` — and compare it to the realised `Σwinner`. It is the same idea (does the
> book's total mass match reality?) in the one form that survives non-exclusivity, and it
> works for a partition too, where it reduces to the original leg-sum.
>
> Measured on 46 fully-resolved boards / 760 resolved legs:
>
> | checkpoint | `Σmid / Σwinner` | verdict |
> |---|---:|---|
> | board creation | **1.38** | unpriced — 85% of legs quote a mid in [45c, 55c] |
> | window-open | 1.11 | usable |
> | daily 12:00Z in-window | 1.28 | usable |
>
> The creation anchor fails here exactly as it failed for quake-etas, and it was caught by
> `Σmid/Σwinner` and by rule 2 below (the null model won), **not** by the literal leg-sum.
>
> **So: before computing any leg-sum, ask whether one leg wins or several.** If several,
> compute `Σmid` vs `Σwinner` instead and say which one you ran. A gate that passes for free
> is not evidence.

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

1. **Gate every checkpoint on leg-sum ≤ ~1.05** (or the family's normal vig level) **when
   the family is mutually exclusive** — and on `Σmid` vs `Σwinner` when it is nested, which
   is the general form. Report whichever you ran, by name, beside every headline number.
   "Leg-sum clear" on a nested ladder is a statement about nothing.
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
