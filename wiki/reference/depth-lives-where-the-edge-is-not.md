# Depth lives where the edge is not — gate the leg, never the board

> **In plain English:** a busy market is not a busy price. On a bucket ladder the money
> trades in the middle and the mispricing sits in the wings, and those are different legs
> with different books. A board that turns over $1.5M a week can offer **$7** at the price
> you actually want to buy.

Measured 2026-07-29 on post-count ladders (`ideas/2026-07-29-post-count-ladders-discarded.md`).

## The measurement

The live Polymarket Elon weekly post-count board carried **$1,562,707** of volume. Its
interior legs quoted **1c wide on $74–79k** each. The legs where the backtest put the edge —
the wings — looked like this at the same instant:

| leg | lifetime volume | live book |
|---|---:|---|
| `500+` | $96,266 | **no quote, either side** |
| `380-399` | $81,193 | **no quote, either side** |
| `440-459` | $71,815 | **no quote, either side** |
| `400-419` | $65,109 | **no quote, either side** |

On the companion Trump board: `160-179` quoted **0.050 / 0.110** — a 6c spread on a 5c leg —
and `180-199` quoted **0.200 / 0.350**, an **11c spread**.

Walking the book on legs that *did* have one, in the 2–10c band where the edge lived:

| order | VWAP vs a 0.0265 mid | VWAP vs a 0.0685 mid |
|---|---:|---:|
| $100 | +1.72c | +0.23c |
| $500 | +6.54c | +3.49c |
| $2,000 | **+14.36c** | **+10.13c** |

Median notional resting at the best ask across all such legs: **$7**. Against a measured
+11.14pp edge, the break-even bound `q⁻ = 0.0709` cleared `q*` by +0.07pp at $100, and failed
by **−4.95pp at $500** and **−13.04pp at $2,000**.

## Why this is a distinct failure mode

We already had three ways a quote lies — [phantom midpoints](phantom-midpoints.md) (a dead
book quotes a fabricated mid), the [tape gate](tape-gate.md) (a tight spread with listed
depth and no trades ever), and [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) (the mid
and the print are both unreachable). This is a fourth, and it is the one that survives all
three checks:

**The board is genuinely liquid. The tape is real. The mid is honest. And the specific leg
your edge lives on has nothing behind it.**

On any bucket ladder the crowd's mass — and therefore the flow, the market makers and the
depth — concentrates around the mode. A mispricing, when one exists, is almost by
construction *away* from the mode: that is where attention is thin, which is why it is
mispriced in the first place. **The property that makes a leg mispriced is the same property
that makes it unfillable.** So the two sets are not merely different, they are
anti-correlated, and a board-level liquidity gate is structurally incapable of detecting it.

## Rules

1. **Never gate on a board-level statistic.** Volume, `liquidityNum`, taker counts and spread
   medians computed over a whole event answer a question about legs you will not trade.
2. **Walk the book at the price band your rule actually buys in**, for the size you actually
   intend, and report the VWAP — not the best ask, and never the mid. Top-of-book size is
   itself misleading: $7 at the ask and a 5,000-share second level are the same "best ask".
3. **Report break-even at the walked VWAP** (`break-even-win-rate.md`), and state the order
   size it assumes. An edge that clears at $100 and fails at $500 is a capacity number, not
   a strategy — say so in those words.
4. **Do this before the modelling, not after.** It is nearly free: one `/book` call per leg
   on today's live boards tells you whether the band your idea targets can be traded at all,
   and it is valid evidence even though it is measured today rather than historically.
5. **A wing leg with large lifetime volume and no current book is the norm, not an anomaly** —
   see [lifetime-volume-is-look-ahead](lifetime-volume-is-look-ahead.md). Those two pages are
   the same lesson at two different times: that field is wrong about the past *and* about now.

## The corollary that is worth more than the rule

If depth is systematically absent exactly where mispricing is systematically present, then
**taker-side edge in ladder wings is close to structurally unreachable for us**, across
families, not just this one. That points the same way as the mention-market kill of 07-28:
the surviving construction is the **maker** side, where a 6–11c wing spread is income rather
than cost. `CONSTITUTION.md` §5 forbids resting orders, so this is a note for whoever revisits
that constraint — it is now the second family in two days whose only live thread is maker-side.

## The boundary: this needs a mode, and a standalone binary does not have one

Amended 2026-08-01 (`ideas/2026-08-01-deep-tail-carry-discarded.md`), after the second family
to clear this wall and the first to clear it with room to spare.

The anti-correlation above is a property of **multi-leg boards**. It needs somewhere else for
the depth to go: a mode that attracts the flow while the mispricing sits in a wing. Two
measurements now bound it:

- **Object 14 (cumulative "by &lt;date&gt;" ladders):** a ladder with *no mode* — the unquoted
  legs were the already-decided rungs, not the edge rungs. It cleared, but modestly:
  $264 at the bid, $2,000 walked for 0.4–2.0c. See
  [nested-ladders-trade-depth-for-power](nested-ladders-trade-depth-for-power.md).
- **Object 16 (standalone ≤5c binaries):** no board at all. The tail leg **is** the market, so
  there is no mode to compete with it, and depth and edge land on the same token:

  | leg | $ resting at the NO ask | VWAP $100 | VWAP $10,000 |
  |---|---:|---:|---:|
  | Hantavirus pandemic in 2026? | **$19,352,132** | 0.9650 | 0.9650 |
  | Will China invade Taiwan by end of 2026? | $16,123,889 | 0.9610 | 0.9610 |
  | Xi Jinping out before 2027? | $15,321,526 | 0.9550 | 0.9554 |
  | Will Jesus Christ return before 2027? | $5,445,228 | 0.9810 | 0.9810 |

  **Zero slippage to $10,000** against object 13's median **$7** at the ask. Median spread
  0.60c — 20.0% of the YES leg but **0.62% of the NO leg**, which is the asymmetry that makes
  the fade look clean.

So the rule to carry forward is not "books are thin", it is:

> **Ask where the mode is before you walk the book. If the leg you want to trade is the whole
> market, the depth wall does not apply — and you must then find your kill somewhere else.**

Object 16 did: it cleared this wall by the largest margin the firm has recorded and died on
power and on the incumbent anyway. Passing the depth gate is not evidence of edge; it only
means the cheapest execution kill is unavailable, exactly as an empty Kalshi slot means the
cheapest incumbent kill is unavailable ([sharp-line-screen](sharp-line-screen.md)).

## See also

- [rare-event-edges-need-rare-event-samples](rare-event-edges-need-rare-event-samples.md) — the wall object 16 died on instead
- [tape-gate](tape-gate.md) — listed depth that never trades
- [phantom-midpoints](phantom-midpoints.md) — an unpriced leg does not vote
- [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) — price both sides before believing an edge
- [lifetime-volume-is-look-ahead](lifetime-volume-is-look-ahead.md) — the same field, lying about the past
- [break-even-win-rate](break-even-win-rate.md) — q*, q, q⁻, and losses-to-ruin
