# A tight spread is not liquidity — gate on the tape

> **In plain English:** a market can show a buyer and a seller a penny apart and still be a
> market where nobody has ever traded. The quote is two people posting prices at each other.
> If you want to know whether you can get out, look at what has actually changed hands, not
> at what is being asked.

This is the **third** distinct way a quoted price lies to us, and the least obvious:

| failure | what you see | what is true |
|---|---|---|
| [phantom-midpoints](phantom-midpoints.md) | a midpoint near 0.50 | the book is empty; 0.05/0.95 reported as ~0.51 |
| [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) | a plausible midpoint on a live book | the mid sits between a derisory bid and a greedy ask; you trade at the bid |
| **this page** | **a 1c spread with listed depth** | **nobody has ever traded there** |

The first two are about the *price*. This one is about the *venue*: the quote is real, the
spread is genuinely tight, the depth is genuinely posted — and the market has no tape.

## Measured (2026-07-26, barrier-touch/ladder-rv)

NVDA week-of-Jul-27 quoted **six legs 1–5c wide with $470–780 of listed liquidity**, and
**five of them had zero trades, ever**. Every screen we had at the time passes that board:
the spread gate passes, the depth gate passes, the phantom-midpoint gate passes because the
book is genuinely two-sided. It is exactly the profile that looks best on a dashboard.

The same run measured what the tape says about reachability, by board family, replaying
forward from each prediction's own timestamp across all 70 markets we had predicted on:

| family | reachable fraction of the scored midpoint |
|---|---:|
| BTC | 100% |
| WTI | 99% |
| silver | 89% |
| gold | 82% |
| **SPY / NVDA weeklies** | **38%** |

**This materially revised our own headline.** The 2-of-21 reachability result in
[midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) was not a property of the strategy — it
was equity weeklies and sub-3c wings. The commodity monthlies, which are the same variant
doing the same thing, are 82–100% reachable. Splitting by board family before concluding
anything about a *variant* is now mandatory.

## The gate

Adopt both halves; the first without the second is what failed:

1. **Relative spread**, not absolute: `spread ≤ min(5c, ½ × mid)`. A 5c spread is nothing at
   a 60c mid and is the whole edge at a 6c mid.
2. **Tape gate**: at least one *taker* trade **on the side we would take**, within 5c of our
   price, in the last 7 days. Fold No-side trades into Yes-equivalent units the way
   `tools/fillcheck/src/main.rs` does, or you will count the wrong side.

## Warnings

- **Listed depth and realised flow differ by orders of magnitude.** Measured the same day on
  a different family: book depth of $7.2k–$25.9k against seven-day taker flow of $58–$668 on
  the side we would have taken.
- **Best edge and worst book correlate.** Gold had the best Brier edge of any commodity board
  and the thinnest tape — **0 of 11 markets ever showed a bid at our midpoint**. If a screen
  ranks legs by disagreement with the market, it is partly ranking them by illiquidity.
- **A leg with no tape is not necessarily unfillable** — a resting bid nobody hit leaves no
  trace. Treat the gate as evidence of fillability, never as proof of unfillability, and
  keep it conservative in the direction that costs us trades rather than money.

## See also

- [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) — the mid is not the price you get
- [phantom-midpoints](phantom-midpoints.md) — the dead-book case
- [break-even-win-rate](break-even-win-rate.md) — once you know the price you can get, this
  decides whether it is worth taking
