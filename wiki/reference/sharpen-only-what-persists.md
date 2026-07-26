# Sharpen only what persists

> **In plain English:** on markets that ask "who is top of the leaderboard at the end of the
> month?", it is often right to back the current leader harder than the crowd does. But that
> only works when the leaderboard actually stays put. When the top two are close together
> and both are freshly measured, the crowd's caution is not timidity — it is correct, and
> pushing past it is how you lose.

A favourite-longshot correction inside a recurring ranking cohort is **conditional on the
underlying ranking being persistent**. The correction is not a property of the market; it is
a property of the process the market resolves on. So measure persistence on the
**resolution variable's own archive**, never on prices.

## The proxy that works: margin between incumbent and best challenger

Measured 2026-07-26 on the LMArena vintage archive (`arena-rank/favourite-shrinkage`):

| cell | n | months | favourite won | market said | market gap | our α=1.75 rule said | our gap |
|---|---:|---:|---:|---:|---:|---:|---:|
| margin ≥8, market fav == leader | 56 | 9 | 0.946 | 0.815 | +13.2pp | 0.954 | −0.8pp |
| margin 4–7, market fav == leader | 27 | 10 | 0.889 | 0.777 | +11.1pp | 0.918 | −2.9pp |
| margin 0–3, market fav ≠ leader | 12 | 5 | 0.917 | 0.709 | **+20.7pp** | 0.836 | +8.1pp |
| **margin 0–3, fav == leader, priced 0.60–0.90** | **5** | **4** | **0.800** | **0.800** | **+0.0pp** | 0.951 | **−15.1pp** |

The last row is the trap. **At a 0–3 margin with the crowd backing the incumbent, the crowd
is already right** — its gap is zero and our sharpening rule overshoots by 15.1pp, the
largest model error anywhere in the sample. The edge is real in the other three cells.

Note the shape of this: the margin screen **does not invert** the favourite-longshot result,
it *localises* it. The market beat a margin-based null model in every band (log-loss
0.25–0.46 vs 0.52–1.26), so the finding is "the crowd is right here specifically", not "the
crowd is right in general".

## What drives persistence: measurement noise, not the gap alone

Per-refresh score movement on the same archive:

| row type | sd(Δ score) |
|---|---:|
| Preliminary | **5.87** |
| under 5k votes | **6.55** |
| established | 2.25 |
| ≥5k votes | 1.60 |

A 3-point margin between two rows that each move ±6 per refresh is not a lead — it is a coin
flip that has not been flipped yet. And **the entity the board resolves on may churn far
faster than the row does**: company-level "best Chinese model" leadership persisted only
0.44–0.54 at a 0–3 margin, going Alibaba → Z.ai → Baidu → Alibaba → Baidu → Bytedance →
Alibaba → Baidu → Z.ai → Alibaba across the archive, while the overall board barely moved.
Persistence is a property of the *resolution question*, not of the leaderboard in general.

## Rules

1. **Measure persistence on the resolution variable's archive**, at the granularity the
   market resolves on (company, not model row, if the board asks about companies).
2. **Condition on measurement quality**, not just on the gap. Preliminary status and low vote
   counts are what make a lead fragile.
3. **Never carry a pooled statistic across a sub-population boundary** — see below; this is
   the specific error that made a dead board look tradeable.
4. **Where the crowd already prices the persistence, there is no trade.** A gap of +0.0pp is
   a finished market, not a cheap one.

## The pooled-statistic trap (a sibling of published-ci-vs-printed)

The application that justified the losing trade cited "one-refresh persistence 0.976–0.982".
That figure was real — it was the **top-30 pooled** number from the predecessor's backtest,
computed mostly on established rows with 50k+ votes. The pair it was applied to were both
Preliminary and both under 5k votes, a sub-population where the same statistic is
**0.846 (11/13)**.

This is the mirror image of the error in [published-ci-vs-printed](published-ci-vs-printed.md).
That page warns against using a published confidence interval as your σ, because the CI
describes a *latent* quantity while the market resolves on the *printed* one. Here the
mistake runs the other way: a statistic computed on one population was quoted for a
sub-population that behaves nothing like it. **A statistic is only a description of the
population it was computed on.** Carrying it across a boundary is the same failure in a new
costume, and it is the single thing that made this board look tradeable.

## See also

- [favorite-longshot-bias](favorite-longshot-bias.md) — the effect this page conditions
- [published-ci-vs-printed](published-ci-vs-printed.md) — the sibling error
- [break-even-win-rate](break-even-win-rate.md) — why the high-price end of the same cohort
  was uninvestable even where the effect was real
- [recurring-crowd-calibration](recurring-crowd-calibration.md) — the cheap first test of
  whether a recurring crowd is already calibrated
