# Clustering coarser is not automatically safer

> **In plain English:** when you realise your data points aren't independent, the instinct is
> to group them into bigger buckets and hope that's the conservative choice. It isn't. Group
> them the wrong way and the correlation *vanishes from the arithmetic* while staying in the
> world — and if you group them so coarsely that only a handful of buckets remain, the
> estimator quietly returns "no correlation" because it can no longer see any.

Standard practice for correlated observations: pick a cluster, estimate the intraclass
correlation `ρ`, inflate the variance by the design effect `1 + (k̄ − 1)·ρ`, and report the
effective sample size `n_eff = n / deff`. The unstated assumption is **monotonicity** — that a
coarser cluster always gives a smaller `n_eff`, so when in doubt you cluster coarser and call
it conservative.

**That assumption is false in both directions**, and both failures were measured on the same
dataset on 2026-07-31 (356 one-touch barrier legs, loss indicator, `barrier-touch/ladder-rv`).

| cluster level | clusters | k̄ | ρ | deff | n_eff |
|---|---:|---:|---:|---:|---:|
| family = board × **direction** | 84 | 4.24 | **0.326** | 2.05 | **173** |
| board (asset × window) — *coarser* | 46 | 7.74 | **0.073** | 1.49 | **238** |
| asset × direction — *coarser still* | 14 | 25.43 | 0.083 | 3.02 | **118** |
| asset — *coarsest* | 7 | 50.86 | **0.000** | 1.00 | **356** |

Reading down that table, `n_eff` goes 173 → 238 → 118 → 356. It is not monotone, and the
coarsest level returns the *original* sample size — the least conservative answer available.

## Failure 1: a coarser cluster can average away the correlation

Going from `board × direction` to `board` **more than quartered** `ρ` (0.326 → 0.073).

The reason is not statistical, it is structural. A board holds an up-ladder and a down-ladder.
Within a direction the legs are monotone functions of one number (the running extreme), so
they lose together — strong positive correlation. But up-legs and down-legs are driven by
*opposite* tails of the same path: the sessions that take out the down-ladder are usually the
ones that leave the up-ladder safe. Pooling them puts a negatively-related pair inside one
cluster, and the within-cluster variance the ICC divides by absorbs it.

**The correlation did not go away. It got averaged against its own opposite.** A design effect
of 1.49 on that clustering is an arithmetic artifact of a badly chosen bucket.

> **The rule:** cluster on the variable that generates the dependence, not on the container
> that happens to hold the rows. Here the generator is `(underlying, direction, window)` —
> "one draw of how far this thing travelled this way". The board is just a listing convention.

## Failure 2: too few clusters makes `ρ` unidentifiable, and it reports as zero

At the asset level there are **7 clusters**. The ANOVA estimator returned `ρ = 0.000`,
`deff = 1.00`, `n_eff = 356` — formally "these 356 legs are independent".

They are obviously not. What happened is that with `a = 7` the between-cluster mean square has
6 degrees of freedom, is estimated terribly, and comes in below the within-cluster mean square;
the estimator truncates the negative variance component at zero. **A ρ of zero from seven
clusters is unidentifiability, not independence** — and it is the most dangerous output in the
table, because it is the only one that says "promote".

This is the same shape as `wiki/reference/existence-is-not-completeness.md` at the level of a
statistic rather than a file: a fallible estimate defaulting to a valid-looking value that
carries no information. **Never quote an ICC without its cluster count**, and treat anything
under ~15–20 clusters as unmeasured rather than measured-at-zero.

## What to do instead

1. **Name the generating variable first**, from the mechanism, before touching the data. Then
   cluster on it. Do not search cluster definitions and report the one that clears.
2. **Report the whole ladder of levels, with cluster counts**, and take the answer from the
   level you pre-committed to. The others are diagnostics.
3. **Bracket, don't point-estimate.** Where the mechanism says two levels are both real —
   here, within-family and between-families-sharing-an-underlying — the honest statement is
   `n_eff ∈ [118, 173]`, not either endpoint.
4. **Drop levels whose cluster count is too small to identify ρ**, explicitly and by name, so
   nobody later mistakes the gap for an omission.
5. A coarser cluster is only conservative if it *contains* the finer one's dependence **and**
   has enough clusters left to estimate it. Check both before believing the number.

## See also

- `wiki/reference/nested-ladders-are-one-draw.md` — why a ladder is one bet paid `k` times.
- `wiki/reference/break-even-win-rate.md` — what `n_eff` is actually used for: the 95% lower
  bound that has to clear `q*`.
- `wiki/reference/existence-is-not-completeness.md` — the same failure shape for files and
  fields.
