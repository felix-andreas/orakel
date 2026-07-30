# A ladder is one draw with a staircase payoff, not N bets

> **In plain English:** if you sell "will oil dip to 80", "dip to 75", "dip to 70" and "dip to
> 65" on the same contract in the same month, you have not made four bets. You have made one
> bet on how far oil falls, paid four times. When it falls far enough to hurt, it hurts on all
> four at once — and the ones that pay you least are the ones you lose first.

This is the correction to a specific and very natural mistake: counting **legs** as
observations. It bites twice — once in the *evidence* (your sample is smaller than n) and once
in the *risk* (your book is more concentrated than its leg count).

## The structure

A one-touch ladder on one underlying over one window is a set of legs that are **monotone
functions of a single random variable** — the running extreme of that underlying over that
window. If the running minimum reaches 80, every down-leg with a barrier above 80 has already
resolved YES. There is nothing probabilistic left to diversify:

> **k nested legs on one (asset, direction, window) = one draw with a staircase payoff.**

Up-legs and down-legs on the same board are two draws only in the loosest sense — they are
functions of the same path, negatively related. So the honest independent unit is closer to
**(asset, window)** than to (asset, direction, window), and much further from **leg**.

This is *not* the same as the leg-sum objection in
[checkpoint-artifact](checkpoint-artifact.md). That page says a nested ladder makes
`Σ leg ≈ 1` a gate that cannot fail. This page says the same nesting makes your *sample size*
and your *tail* a fiction. Different consequence, same structural fact.

## Measured (`barrier-touch/ladder-rv`, 2026-07-30, 356 sell-signal legs)

| | |
|---|---:|
| legs | 356 |
| distinct boards (asset × window) | 46 |
| distinct monotone families (board × direction) | 84 |
| mean legs per family | 4.24 (max 12) |
| **intraclass correlation of the loss indicator within a family** | **ρ = 0.325** |
| design effect `1 + (k̄ − 1)·ρ` | **2.05** |
| **effective n** | **173** |

And what that does to the promotion decision, via
[break-even-win-rate](break-even-win-rate.md) on a sell-only (i.e. favourite-buying) book:

| | n | `q*` | `q` | `q⁻` | verdict |
|---|---:|---:|---:|---:|---|
| nominal | 356 | 0.822 | 0.868 | 0.829 | **clears** by +0.73pp |
| **effective** | **173** | 0.822 | 0.868 | **0.808** | **fails** by −1.32pp |

**The same evidence clears its break-even bound at the leg count and fails it at the draw
count.** Nothing about the edge changed; only the honesty of the sample size did.

## The risk half: the premium and the loss sit on different rungs

The evidence problem is the boring half. The dangerous half is that a ladder's premium is
**not** distributed along it. Measured on the same variant's live book — a WTI down-ladder of
21 outstanding legs that collected 1.15 of premium:

| running minimum reaches | legs lost | loss | net vs premium |
|---:|---:|---:|---:|
| 80 | 1 | 0.66 | +0.49 |
| **77.80 — what actually happened (−14.0%)** | **1** | **0.66** | **+0.49** |
| 75 | 8 | 6.96 | **−5.81** |
| 70 | 11 | 9.90 | −8.75 |

**90% of the premium sat on the two rungs nearest spot**, and those are the first two a
continuing move removes. The far wings contributed **4.7% of the premium and 5.92 of loss
exposure**. The marginal cost of the next 5% move was **548% of the family's entire premium**.

That is a **cliff, not a tail**: a −14% move left the book profitable and a −18% move took it
to −5.8× its premium. The distinction matters because a tail can be sized against and a cliff
mostly cannot — the loss is not a smooth function of the shock.

Selling far wings for a cent or two is therefore not "cheap diversification". Those legs are
nearly free to sell **and nearly free to lose**, carried off by the same move that takes the
rungs you actually get paid for. See [favorite-longshot-bias](favorite-longshot-bias.md) for
why they look mispriced in the first place, and why that is still true and still not enough.

## Rules

1. **Report the number of independent draws beside every n.** For ladder families that is
   distinct (asset, window), not legs, and not leg-days.
2. **Recompute every bound at the effective n.** `n_eff = n / (1 + (k̄ − 1)·ρ)`, with ρ the
   intraclass correlation of the *outcome* within a family. Quote ρ; it is the whole argument.
3. **Between-family correlation makes it worse, never better.** Families on the same
   underlying in different windows, and all families under one macro factor, are not
   independent either. **An effective-n computed within families is an upper bound on your
   real sample**, so a failure there is a floor on the problem.
4. **Size by family, not by leg.** A per-leg position limit lets one contract's running
   extreme carry a dozen positions.
5. **Report the staircase, not the expectation.** For each family: loss as a function of how
   far the underlying travels, and where the premium sits along it. A mean and a p99 both
   average over the one variable that drives everything.
6. **A repeated daily prediction on the same leg is not a new draw either.** That is the same
   error in time rather than in strike — see [checkpoint-artifact](checkpoint-artifact.md).

## Where else this applies

Any family whose members are thresholds on one quantity: price ladders ("reach X by date"),
temperature and climate thresholds, cumulative-count and over/under ladders, tournament
"reach round ≥ k" markets, and rating/rank cutoffs. If the legs can be sorted so that one
outcome implies the next, they are one draw.

## Converged on independently, the same day

A market researcher reached the same structural fact on 2026-07-30 from the opposite
direction — evaluating *cumulative by-date* ladders for **selection** — and wrote
[nested-ladders-trade-depth-for-power](nested-ladders-trade-depth-for-power.md). Two agents,
two unrelated market families (barrier ladders on one price; date ladders on one event), same
conclusion: **count events, not legs.**

The two pages are different consequences and both are worth keeping:

- **that page** is a *selection* result — nesting buys real book depth (a cumulative ladder has
  no mode, so no single leg hoards the flow) and pays for it in power. It tells you which
  ladder shape to onboard.
- **this page** is a *sizing and evidence* result — nesting halves your effective n and
  concentrates your loss on the rungs holding your premium. It tells you how big to go on a
  ladder you already own.

That the same fact fell out of both a depth question and a risk question is the reason to
trust it. If the two pages ever drift, this one owns ρ and effective n; that one owns the
depth/power tradeoff.

## See also

- [nested-ladders-trade-depth-for-power](nested-ladders-trade-depth-for-power.md) — the
  selection half of this fact, derived independently
- [break-even-win-rate](break-even-win-rate.md) — the bound that this halves the power of
- [checkpoint-artifact](checkpoint-artifact.md) — nesting makes `Σ leg ≈ 1` unfailable, and
  repeated checkpoints are the time-domain version of this page
- [favorite-longshot-bias](favorite-longshot-bias.md) — why the wings look overpriced
- [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) — the other correction that moves `q*`
  the wrong way
