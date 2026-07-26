# Threshold ladders on a rounded number: get the rounding exactly right

> **In plain English:** lots of markets ask "will this percentage be at least 80?" where the
> percentage is itself a rounded figure. If you round the way your programming language
> rounds by default, you will be wrong on exactly the cases that decide the bet — the ones
> landing precisely on the boundary.

A ladder over a **rounded** quantity is a ladder over a lattice, and the strike is a lattice
boundary. Whether 79.5 becomes 79 or 80 is not a detail; it *is* the payout of the `80+` leg.

## Measured (2026-07-26, Rotten Tomatoes score paths, 2,608 Wayback captures)

`score == round(100 × liked / (liked + not_liked))` holds on **2,128 / 2,128 testable rows —
100.000%, zero counterexamples**. The subscore obeys the identical rule (1,628/1,628). And
`n == liked + not_liked` on every row where all three appear, so the denominator is exactly
the review count with no hidden abstention bucket.

**The mode is half-up, not banker's.** Twelve rows land on an exact `.5` tie and the venue
rounds **up** in all twelve:

| film | liked | not liked | n | exact | published |
|---|---:|---:|---:|---:|---:|
| 28 Years Later: The Bone Temple | 259 | 21 | 280 | 92.50 | **93** |
| Eternals | 105 | 95 | 200 | 52.50 | **53** |
| The Batman | 173 | 27 | 200 | 86.50 | **87** |
| Star Wars … Grogu | 140 | 84 | 224 | 62.50 | **63** |

Accuracy of each approach over the same 2,128 rows:

| method | correct |
|---|---:|
| `Decimal(...).quantize(1, ROUND_HALF_UP)` | **2,128 / 2,128** |
| Python's built-in `round()` (banker's) | 2,116 / 2,128 |
| truncation | 1,094 / 2,128 |

Python's `round()` is wrong on **exactly the twelve ties** — i.e. it fails only where the
answer matters most, and passes 99.4% of a backtest while doing it. That is the worst
possible error profile: invisible in aggregate, decisive at the strike.

## Rules

1. **Verify the rounding rule against the venue's own published number** before modelling
   anything, on every row you can. It is a few lines and it is falsifiable.
2. **Use an explicit half-up rounder.** Never a language default — Python, and IEEE-754
   generally, round half to even.
3. **Check for a hidden denominator.** Confirm the published `n` equals the components you
   are summing; a "no opinion" bucket silently shifts every strike.
4. **Strikes are lattice boundaries, not judgements.** A `56/57/58/59/60` ladder is not five
   opinions about a film, it is five adjacent integers. Model the lattice, then the process.
5. **Expect near-monotone counters, not monotone ones.** In this data 11 of 2,492 consecutive
   observations showed the denominator *falling* by 1–4 (a review pulled). A hard monotonicity
   assertion will occasionally reject real data.

## Where this applies beyond one family

Any board resolving on a rounded published statistic: review scores, approval ratings,
poll averages, vote shares, market share, "percentage of X that are Y". The rounding rule is
part of the resolution source and belongs in the same audit as the source itself — see
[first-print-vintages](first-print-vintages.md) for the sibling problem of *which* published
value settles the bet.

## See also

- [venue-resolution-epsilon](venue-resolution-epsilon.md) — the venue can resolve on a
  near-miss; never sell inside the epsilon
- [first-print-vintages](first-print-vintages.md) — which vintage of a revised number settles
