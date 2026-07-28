# Filtering on lifetime volume is look-ahead

> **In plain English:** "only trade the liquid legs" sounds like prudence. But the volume
> number the API hands you is the volume the market ended up with, most of which was traded
> after the moment you are pretending to decide. Keeping the busy legs quietly keeps the legs
> that were *about to get busy* — and what made them busy is usually how they turned out.

Every other liquidity gate in this wiki is about a quote lying to us. This one is about
**our own filter cheating**, and it is the more dangerous kind, because it *improves* the
backtest and every instinct we have says a liquidity filter is conservative.

## Measured (mention markets, 2026-07-28 — `ideas/2026-07-28-mention-markets-discarded.md`)

The same trade, the same checkpoint (T−6h before the first leg of the event resolves), the
same executable price (`yes_bid − y`), event-clustered. The only thing that changes is which
volume number the filter reads:

| filter | legs | events | executable buy-NO edge | t |
|---|---:|---:|---:|---:|
| **lifetime `volume_fp` ≥ 20k** | 829 | 143 | **+21.15pp** | **+7.30** |
| **volume known at T−6h ≥ 20k** | 57 | 13 | +6.04pp | +0.59 |
| **volume known at T−6h ≥ 5k** | 388 | 60 | **−3.06pp** | −0.72 |
| volume known at T−6h ≥5k, spread ≤3c | 316 | 52 | −3.76pp | −0.83 |

A +21pp edge at t = +7.3 — the largest number produced in that run by a wide margin — is
**entirely manufactured by the filter**. Applied honestly it is −3.06pp.

The mechanism is one number: **the median mention leg had traded only 14.3% of its lifetime
volume by T−6h.** 40.6% of all volume in the family trades in the final hour before
resolution. So `volume_fp ≥ 20k` is very close to a statement about what happened *after* the
decision point, and in a family where legs trade hardest while they are resolving, that is a
statement about the outcome.

## Why it is easy to miss

- The field is called `volume`, not `final_volume`. Nothing in the name says "as of
  settlement". Kalshi's `volume_fp`, Gamma's `volumeNum` and `liquidityNum` are all
  terminal-state fields on a resolved market.
- It is applied for a *virtuous* reason. Every page in this wiki says to gate on liquidity
  ([tape-gate](tape-gate.md), [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md)), so the
  filter feels like the responsible thing rather than a free parameter.
- The bias is **not** a wash. It has a direction, set by what drives late volume in the
  family — resolution drama, a price running to 0 or 1, a leg becoming newsworthy. It will
  not cancel out with more data; more data makes it more significant.
- It survives the checks we normally run. The legs are genuinely liquid, genuinely
  two-sided, genuinely have tape. Every gate in `tape-gate.md` passes. The problem is not
  the legs, it is that we *chose* them with hindsight.

## The rule

> **Every filter in a backtest must be computable from data timestamped at or before the
> checkpoint.** For liquidity that means volume accumulated up to the checkpoint —
> reconstructed from the candlestick/price-history path, never read off the settled market
> record.

Operationally, for each candidate leg compute `vol_pre = Σ candle.volume for candle.ts ≤ checkpoint`
and gate on that. It is a two-line change and it is the difference between +21pp and −3pp.

## Generalisation — the audit question

The failure is not about volume. It is about **any** covariate read from a settled record.
Ask of every filter and every feature:

> *Was this number's value at my checkpoint the same as the value I am reading now?*

Fields that fail this test on a resolved market and have already appeared in our pipelines:
`volume_fp` / `volumeNum` (accumulates to settlement), `liquidityNum`, `open_interest`,
`n_markets` on an event whose legs were added late, and any "did this book ever move?"
statistic computed over the market's whole life — including the one
[phantom-midpoints](phantom-midpoints.md) recommends, which must be evaluated over the
pre-checkpoint window only.

The mirror-image trap is worth stating too: if you gate on pre-checkpoint volume and almost
nothing passes (57 legs across 13 events at the 20k threshold here), that is not a data
problem to be worked around by loosening back to lifetime volume. **It is the finding** — it
means the family does not have a tradeable book at the time you would need one.

## See also

- [tape-gate](tape-gate.md) — the liquidity gate this page tells you how to compute honestly
- [phantom-midpoints](phantom-midpoints.md) — the "did the book move?" split, which has the
  same window problem
- [checkpoint-artifact](checkpoint-artifact.md) — the other way a checkpoint fabricates edge
- [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) — where "both sides lose" is diagnosed
