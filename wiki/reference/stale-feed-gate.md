# When the feed is shut, your number is the stale one

> **In plain English:** if the thing a bet resolves on stops updating over a weekend, your
> model keeps producing the same answer while the market keeps thinking. When you then
> disagree with the market, that is not a view — it is you quoting Friday at people who have
> spent the weekend reading the news.

This is the **fourth** distinct way a quoted price misleads, and the only one where the
market's quote is honest and **ours** is the broken number:

| failure | whose number is wrong |
|---|---|
| [phantom-midpoints](phantom-midpoints.md) | the venue's — an empty book reports ~0.51 |
| [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) | the venue's — the mid is not a price you get |
| [tape-gate](tape-gate.md) | the venue's — a tight spread with no tape |
| **this page** | **ours** — priced off an input that could not move |

## Measured (barrier-touch/ladder-rv, 2026-07-27)

`will-wti-dip-to-85-in-july-2026` resolved **YES**. We had predicted no-touch four mornings
running while the market went 0.525 → 0.410 → 0.415 → **0.715**.

The 07-25 and 07-26 runs read the **same spot** (Friday 20:59Z close, 90.46), the **same σ**,
and the **same five remaining sessions**. WTI and the metals trade 22:00Z→21:00Z Mon–Fri, so
the resolving feed was shut from Friday 20:59Z until Sunday 22:00Z — **28.8 hours stale** at
the 07-26 run. The book repriced **0.475 → 0.715 during exactly that closure**, and had been
sitting at 0.71 for seven hours by the time we quoted 0.365 against it. Our model's only
movement across the two runs was **−2.8 points**, caused by a 14-day realized-vol lookback
sliding across two closed days — a bookkeeping artifact, not a view. Then the contract opened
**−7.79%** and printed through the barrier in the first minute.

Three checks that make this a finding rather than a story:

- **No feed we hold saw it.** The spot, the futures and the metals feeds printed **zero**
  times during the closure. There was no input we ignored and nothing to buy.
- **No volatility model reaches 0.715.** As-run 0.3928; implied-vol instead of realized
  0.5156; realized plus a measured weekend-jump term 0.4445; both together 0.5432. Solving
  the market's quote for spot gives **87.3–88.0** against our 90.46 — the market was pricing
  **a lower level**, not a wider distribution. No σ recovers a level from a Friday close.
- **It is systemic, not one row.** Two of the trial's four prediction batches were emitted
  during a closure: Saturday (51 rows, 4.5h stale) and Sunday (13 rows, 28.8h). **64 of 95
  outstanding rows were priced off a shut feed.**

## The gate

> Do not treat a disagreement with the market as edge when the resolving feed has been shut
> for the whole period over which the market moved. If the feed's last print is older than
> the current session break **and** the mid has moved more than ~5c since that print,
> **suppress the row.**

And the operational half, which is the part people skip: **do not schedule a predicting run
into a market's closure.** A daily cadence that ignores session calendars will emit its worst
rows on exactly the days it has least information.

## Rules

1. **Every prediction carries the age of the input it was priced from.** `feed_age_h`,
   `feed_open`, and the jump size in session-σ units. A row without them cannot be audited
   later, and this failure is only visible in hindsight.
2. **A moving market against a frozen input is information you do not have** — not an edge.
   The direction of the surprise is unknowable in advance, so this is not a rule you can
   trade around, only one you can refuse.
3. **Check the session calendar of the RESOLUTION source, not of the venue.** Polymarket
   never closes. The thing it resolves on does.
4. **Weekend and overnight gaps are a whole session's variance.** Measured WTI weekend gap
   sd **3.78%**, comparable to a full session; equity overnight gaps likewise. A model that
   diffuses continuously across a closure understates the tail on both sides.
5. **Suspect a lookback that "moves" during a closure.** A rolling realized-vol window sliding
   across shut days changes your answer while carrying no new information — motion that looks
   like a view and is arithmetic.

## See also

- [delayed-execution-test](delayed-execution-test.md) — the other time-axis failure: an edge
  that exists only in the first three minutes
- [first-print-vintages](first-print-vintages.md) — using a revised value where the market
  settles on the original
- [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) — the venue-side siblings of this page
