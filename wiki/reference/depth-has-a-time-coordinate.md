# Depth has a time coordinate — walk the book at your price band, your size, AND your hour

> **In plain English:** a market can be genuinely busy and still be untradeable, because all
> of its volume trades *after* the outcome is already known. 85.5% of one family's entire
> tape printed after the resolution instant, at a median price of **0.994**. Before the
> deadline — the only time a forecast is worth anything — the median leg had **$76**.

Measured 2026-08-02 on White House "full lid by 6:30 PM" per-day binaries
(`ideas/2026-08-02-full-lid-timing-discarded.md`), 132 settled legs, full Data-API tape.

## The measurement

Each leg resolves on whether a full lid is called by **6:30 PM ET** on a named date. That
instant is the whole contract: before it the outcome is uncertain, after it the outcome is
public and the market simply waits for UMA (median `closedTime` − deadline = **6.7h**).

Splitting every taker fill on that instant:

| | taker notional |
|---|---:|
| before the 6:30 PM deadline | **$156,580** |
| after the 6:30 PM deadline | **$922,456** |
| **post-resolution share** | **85.5%** |

Median post-deadline trade price: **0.994**.

And the pre-deadline half is not merely smaller, it is concentrated in the last hours, when
the answer is nearly obvious:

| checkpoint | median notional/leg | mean | p90 | legs with **zero** tape |
|---|---:|---:|---:|---:|
| T−48h | $11 | $262 | $689 | **59 / 132** |
| T−24h | **$76** | $405 | $1,145 | **38 / 132** |
| T−12h | $157 | $572 | $1,393 | 17 / 132 |
| pre-deadline (all) | $507 | $1,186 | $2,546 | 7 / 132 |

Total ask-side notional available at T−24h across the **entire six-month record**: **$25,606**.
Capturing 100% of it would have earned $3,928.

## Why this is a distinct failure mode

[depth-lives-where-the-edge-is-not](depth-lives-where-the-edge-is-not.md) established the
anti-correlation on the **price** axis: depth concentrates at the mode, mispricing lives in
the wings. That page then bounded itself — the wall "needs a mode", so it does not bite a
modeless ladder or a standalone binary.

This family has **no price mode**: six independent single-day binaries, each its own market.
By the price-axis rule it should have cleared. It did not, because the same anti-correlation
runs along **time**:

> **Depth concentrates where uncertainty is lowest.** On a contract that settles at a fixed
> clock time, that is *after* the clock strikes — and forecastable uncertainty lives entirely
> before it. The property that makes a moment forecastable is the property that makes it
> unquoted.

That is the identical structure, rotated. Which means the generalisation is:

> Walk the book **at the price band your rule buys in, at the size it buys, and at the hour
> it fires.** All three, or the walk answers a question you did not ask.

## The diagnostic, which is one query

You do not need a model to run this. Pull the tape (`data-api.polymarket.com/trades`), take
the contract's own resolution instant from the rules text — not `endDate`, which is often the
board's close — and compute the share of notional printed after it. Anything over ~50% means
the headline volume is settlement carry, and your reachable size is the *pre*-deadline
number, which may be two orders of magnitude smaller.

Sanity check on the interpretation: if the post-deadline prints cluster at 0.99+/0.01−, they
are people buying a decided outcome, not a disagreeing crowd.

## And the settlement carry is not the consolation prize

The obvious response is "then trade the 85.5%". Priced properly it fails, for the reason
object 16 failed:

- Fill-level it looks superb: 2,835 post-deadline favourite-side fills, $1,268,162 notional,
  **+0.133% net over a median 6.75h lock = +172% annualised** against ~4% risk-free.
- **Clustered to legs — the honest n — it dies:** the favourite won **120 of 121 legs =
  0.9917** at a mean entry of **0.9883**, so break-even *is* 0.9883 and the Wilson 95% lower
  bound is **0.9547**. **Fails by −3.37pp.** One losing leg in 121 is the entire margin.

Fills are not draws. See
[rare-event-edges-need-rare-event-samples](rare-event-edges-need-rare-event-samples.md) and
[clustering-coarser-is-not-safer](clustering-coarser-is-not-safer.md).

## Rules

1. **Take the resolution instant from the rules text**, then split the tape on it. `endDate`
   and `closedTime` are both wrong for this: one is the board's close, the other is when UMA
   finished.
2. **Report reachable size at your checkpoint, never lifetime.** "Median $76 at T−24h" and
   "$1.79M lifetime volume" describe the same family.
3. **A high post-resolution share is also a warning about every volume-ranked scan you run** —
   these boards sort as liquid, and they are not.
4. If the family survives, state capacity in dollars per leg per checkpoint, not as a spread.

## See also

- [depth-lives-where-the-edge-is-not](depth-lives-where-the-edge-is-not.md) — the same anti-correlation on the price axis
- [lifetime-volume-is-look-ahead](lifetime-volume-is-look-ahead.md) — the same field lying about the past
- [tape-gate](tape-gate.md) — listed depth that never trades at all
- [phantom-midpoints](phantom-midpoints.md) — and, per 08-02, the phantom can live in the **metadata**, not just the book
- [rare-event-edges-need-rare-event-samples](rare-event-edges-need-rare-event-samples.md) — why the settlement carry is not the escape
