# Break-even win rate — the promotion gate that survives small samples

> **In plain English:** a bet that has won every single time can still be a bad bet. If you
> pay 97 cents to win 3, you need to be right 97 times in 100 just to break even — and
> sixteen wins in a row is nowhere near enough evidence that you are.

Report three numbers for every favourite-side trade, and refuse it when the third is below
the first:

| symbol | meaning |
|---|---|
| `q*` | **break-even win rate** = cost + fee. Buy at 0.97 with a 0.3c fee → `q*` = 0.973 |
| `q` | observed win rate in the sample |
| `q⁻` | **95% lower bound** on `q` (Wilson or Clopper–Pearson — a proportion near 1 has a wildly asymmetric interval, so never use the normal approximation) |

**Trade only when `q⁻ > q*`.** Not `q > q*` — the point estimate of a 16/16 sample is 1.000
and it means almost nothing.

## Why this beats a t-statistic (measured, 2026-07-26, arena-rank/favourite-shrinkage)

| band | n | `q*` | `q` | `q⁻` | t | verdict |
|---|---:|---:|---:|---:|---:|---|
| **0.60–0.90** | 74 | 0.829 | 0.919 | **0.846** | +5.94 | **clears** |
| 0.90–0.93 | 20 | 0.945 | 1.000 (20/20) | 0.861 | +31.2 | fails |
| 0.93–1.00 | 18 | 0.972 | 1.000 (16/16) | 0.819 | +10.3 | fails |

The 0.93–1.00 band **never lost**, has a t-statistic of +10.3, and is uninvestable: it needs
97.2% and 16 instances cannot bound a 3% tail. **2.83 losses per 100 trades take it to
zero.** A t-statistic against "no edge" answers the wrong question — the null that matters
is not zero, it is `q*`, and `q*` climbs toward 1 exactly as the sample's ability to
distinguish it collapses.

This is the same failure the cents-per-trade metric makes. Cents ranked those bands
12.5 / 5.2 / 2.8 — a 4:1 spread that makes the top band look merely worse. Return on locked
capital ranked them 15.2% / 5.5% / 2.9%, and the break-even bound ranks them
**tradeable / not / not**. Only the last one is a decision.

## Rules

1. **Every promotion case carries the `q*` / `q` / `q⁻` table**, per price band, not pooled.
   Pooling hides that the edge and the fundability live in different bands.
2. **Use an exact interval.** Near `q = 1` the normal approximation gives a lower bound
   above 1 or a nonsense width. Wilson is the cheap correct choice.
3. **Express the margin as losses-to-ruin**, because it is the number people feel:
   "2.83 losses per 100 trades take this to zero" ends an argument that "+4.9pp edge,
   t = 10.3" does not.
4. **Deep books do not rescue a thin margin.** The same run measured $7.2k–$25.9k of book
   depth on boards whose realised taker flow on our side was $58–$668 over seven days —
   see [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md).

## See also

- [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) — the price you were scored against
  may have had no counterparty at all
- [favorite-longshot-bias](favorite-longshot-bias.md) — where the favourite-side edge comes
  from in the first place
- `execution/DESIGN.md` §3 — return on locked capital, the metric that ranks bands correctly
  but still does not decide them
