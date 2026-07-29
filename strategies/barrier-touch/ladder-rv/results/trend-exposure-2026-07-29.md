# Is the trial's loss a calibration problem or a directional exposure?

**2026-07-29 (day 7), model claude-opus-5 effort xhigh.** Written in answer to the CEO's
question after `will-wti-dip-to-80-in-july-2026` resolved YES and the headline moved
**−0.0172 over 25 rows → −0.0466 over 31** (per market −0.0051 → −0.0133).

**Answer: neither, as posed. On the variant's own resolved sample the model does not lose
in a trend toward the barrier — it wins there, most of all on WTI down legs. What the July
episode is, is a fat one-sided tail draw, and the tail is real, one-directional, and lands
exactly on the legs we sell.** The 08-02 question is therefore not "is it calibrated" and
not "is it exposed" but **"is this tail acceptable at the size we would trade"**.

## 1. The trial rows, reconstructed from the frozen archive

32 resolved rows over 21 markets (the CEO's scoring run reports 31/20 — see §5). Outcome
taken from the CEO's scoring run; the price path taken from the frozen 1-min archive.
`toward` = the underlying's move from the row's own timestamp to resolution, signed toward
that leg's barrier, as a fraction of spot. `closed` = `toward` / (initial gap to barrier).

| bucket | rows | pooled model−market Brier |
|---|---:|---:|
| barrier reached (`closed ≥ 1`, all WTI ↓) | 12 | **+0.1391** |
| barrier not reached (`closed < 1`) | 20 | **−0.0011** |

The concentration is total. Every row that lost is a WTI ↓ leg the underlying walked into;
the twenty rows that did not touch contribute **−0.0011 pooled, with no single row outside
±0.004**. The 31-row headline is not an average over 31 trials — it is four nested WTI down
barriers on one underlying over one week, plus twenty rows of noise that rounds to zero.

Within the losers, the ordering is monotone in how early the row was written: dip-to-80 on
07-26 (q 0.074 vs mid 0.405) is +0.504; the same market on 07-27 after the move (q 0.491 vs
mid 0.490) is −0.001. We converged; we converged late.

**That much is real but not yet an answer.** Any barrier model loses on the paths that touch
and wins on the paths that do not. The question is whether the *market* beat us on those
paths systematically, or whether we drew a bad week.

## 2. The properly-powered test

`gate2_checkpoints.csv` from `backtest-metals-2026-07-25.tar.gz` (restored from R2):
**5,927 daily in-window checkpoints over 633 resolved legs, 7 assets, May–Jul 2026** — many
episodes rather than one. `cmd_analyze` prices these with plain `touch_prob`, **the same
pricer that produced every losing trial row**, so this is apples-to-apples.

Split by outcome — the exposure hypothesis, stated as "we lose when they touch":

| bucket | n | model−market Brier | t |
|---|---:|---:|---:|
| touched (y=1) | 625 | **−0.01152** | −1.99 |
| not touched (y=0) | 5,302 | −0.00046 | −0.70 |

**The model beats the market on the touched legs.** Its entire measured edge comes from
them. The untouched legs are a wash.

Split by realised trend toward the barrier (an **ex-post diagnostic, not a filter** — see §4):

| trend toward barrier | n | model−market Brier | t |
|---|---:|---:|---:|
| away (< −2%) | 2,012 | +0.00402 | +3.34 |
| flat (±2%) | 2,360 | −0.00437 | −3.08 |
| toward 2–5% | 666 | −0.01400 | −4.80 |
| toward 5–10% | 548 | +0.00469 | +1.79 |
| toward >10% | 341 | −0.00187 | −0.46 |

And on the exact construction the CEO names:

| WTI ↓ legs | n | model−market Brier | t |
|---|---:|---:|---:|
| all | 571 | −0.00573 | −2.39 |
| trend toward barrier ≥ 5% | 363 | **−0.01259** | **−4.66** |
| trend toward barrier < 5% | 208 | +0.00622 | +1.38 |

Pearson correlation of per-checkpoint error with trend: **r = −0.031** (n = 5,927) — nil,
and what little there is points the *helpful* way.

**So the hypothesis "a sell-touch variant is structurally punished in a trending market" is
refuted on this data.** WTI down legs with the underlying trending ≥5% into the barrier are
the single best bucket the variant has.

## 3. What is real: a one-sided tail

Per-leg mean model−market Brier over the 633 resolved legs:

| stat | value |
|---|---:|
| mean | −0.00268 |
| median | −0.00020 |
| sd | 0.06160 |
| p1 / p5 / p25 | −0.2239 / −0.0995 / −0.0124 |
| p75 / p95 / p99 | +0.0115 / +0.0817 / **+0.1727** |

- dip-to-80 at **+0.169** sits at ≈ p98.7 — **8 of 633 legs** (1.3%) are worse.
- dip-to-85 at **+0.113** sits at ≈ p97 — **18 of 633** (2.8%) are worse.

Drawing one such leg in a trial holding 20 markets is ordinary. Drawing two is ordinary
*given that they are the same event*: both are WTI down barriers on one contract over one
selloff, nested, so they are ≈1 observation and not 2.

But the tail has a direction, and this is the part that should survive to the review:

> **The eight worst legs in the entire 633-leg backtest are all `dip-to` legs.** Every one.
> Across silver, NVDA, SPY and gold — so it is not a WTI fact, it is a *down-barrier* fact.

The distribution is left-skewed in the good direction (p1 −0.224) with a long right tail
(p99 +0.173), and the right tail is where the sell-side sits. Mean edge, fat one-sided tail.

## 4. What this does not license

`trend` is computed **from the leg's future**. It explains errors after the fact; it is not
available at the checkpoint and must never become a filter. Gating on it would be precisely
the error in `wiki/reference/lifetime-volume-is-look-ahead.md` — a covariate read off the
settled record, improving the backtest by selecting on the outcome. It is written here as a
diagnostic and is labelled as one everywhere it appears.

Second caveat, stated plainly: the 633-leg sample is the sample this variant's method was
chosen on (sell-only, RV-primary, the gates). "Model beats market on touched legs" is
therefore **partly in-sample**. The out-of-sample answer lands 07-31.

## 5. Loose end

I reconstruct **32** resolved rows over 21 markets; the CEO's scoring run reports **31** over
20. The difference is one row and I have not identified which — my reconstruction takes
outcomes from the CEO's run and derives only the price path, so it does not resolve the
discrepancy. Worth one minute on Friday before the numbers are quoted; not worth changing
any conclusion here, since every split above is robust to one row.

## 6. What I recommend the 08-02 review actually asks

1. Not "is the model calibrated" — on 5,927 checkpoints it is, and it beats the market where
   the CEO expected it to lose.
2. Not "is it directionally exposed" — the mean effect runs the other way.
3. **"What is the loss at the 99th percentile leg, at the size we would trade, and how
   correlated are the legs we would hold?"** The July episode put four nested barriers on one
   contract in the book at once. That is a *correlation and sizing* question, and it is what
   `break-even-win-rate`'s q\*/q/q⁻ table plus return-on-locked-capital exist to answer.
   `execution/DESIGN.md` §3 already says cents/trade is the wrong unit; this is the concrete
   reason.
