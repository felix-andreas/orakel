# A midpoint is not a fill

> **In plain English:** the price you see quoted on a thin market is the average of
> what buyers offer and what sellers ask. On a long-shot bet those two numbers are
> miles apart, so the quoted price is one nobody has ever agreed to. Beating it is a
> real forecasting result and is worth exactly zero cash.

A midpoint is `(bid + ask) / 2`. On a liquid leg the bid and the ask are a cent apart
and the midpoint is very close to a price you can trade at. On a wing leg quoted
**0.001 / 0.08** the midpoint reads **4c** — and 4c is a number no counterparty ever
offered. Every "we beat the market by X" computed against that number is measuring
forecasting skill against a phantom.

This is the sibling of [phantom-midpoints](phantom-midpoints.md). That page is about
*dead* books, where the quote is fabricated by an empty order book. This page is the
harder case: the book is **alive**, the quote is real, and it is still not a price you
can get.

## Measured on our own ledger (2026-07-25)

Our first scored batch was 21 predictions, all `barrier-touch/ladder-rv`, and all 21
beat the market on paired Brier. `tools/fillcheck` replayed Polymarket's public trade
feed for every one of them and asked: after we spoke, at what price did somebody
demonstrably trade the side we wanted to take?

| | rows |
|---|---|
| beat the market on paired Brier | **21 / 21** |
| a counterparty reachable at-or-above the price we were scored against, ever | **2 / 21** |
| …within the first hour, when we would actually have traded | **1 / 21** |

Summed across the batch, the midpoints we scored against total **1.335**; the best
prices anyone was actually observed at total **1.037** — **78%**, and that is the
generous reading, because it lets each row take its best price over the market's whole
remaining life rather than at the moment we spoke.

The single row with real liquidity was `will-wti-dip-to-90-in-july-2026`: $34k of
volume, a bid above our price inside the first hour. It is also the row where we had
almost no edge — we said 0.8263, the market said 0.82. **The one trade that existed is
the one that wasn't worth doing.** It contributed 11% of the batch's total improvement;
the other 89% sat in wings nobody would take the other side of.

The wings are stark on their own terms: `will-spy-reach-760` was scored at a midpoint of
2.55c against a best-observed bid of 0.12c. `will-nvda-dip-to-188` never traded on the
bid at any price at all.

## The one-line diagnostic: if both sides lose, you measured the spread

**Measured 2026-07-28 (mention markets), and it is the cheapest form of this check we have
found.** Re-price your claimed edge twice — once as "buy YES at the ask", once as "buy NO at
the bid" — and look at the signs:

| priced at | buy NO (`bid − y`) | buy YES (`y − ask`) |
|---|---:|---:|
| last traded price | *+5.13pp, t = +5.27* | — |
| **executable** | **−2.51pp, t = −2.48** | **−7.92pp, t = −7.97** |

A real mispricing is one-sided by construction: if YES is too dear, selling it must pay. When
**both** directions lose simultaneously, no mispricing exists and the number you were
admiring is the distance from your quote to the executable price. Here that identity closed
to the cent, on 3,996 legs:

```
mean spread                8.70c
mean (last trade − mid)   +1.83c
mean (last trade − bid)   +6.18c
last-trade edge (p − y)   +4.56pp
executable edge (bid − y) −1.63pp
                           ------
difference                 6.18pp   ==  mean(last trade − bid)
```

Two things this adds to the pages above:

- **A last-traded price is not a fill either.** This page and
  [phantom-midpoints](phantom-midpoints.md) both warn about *quotes*. A trade print looks
  immune — someone really did transact there — but on a one-sided book the prints cluster at
  the ask, and scoring against them flatters the sell side by the full half-spread plus the
  order-flow imbalance (+1.83c of it here).
- **Run it before the modelling, not after.** It costs one extra column. It killed a family
  that had already passed the catalogue scan, the specialist search, the tape gate and the
  feed-stability gate.

## Rules

0. **Report the edge on both sides at executable prices.** If both are negative, stop —
   there is nothing there, whatever the t-statistic on the quoted price says.
1. **Never report a Brier improvement without its fillable count.** `scoring/` now
   prints `n_fillable / n_known_fill` on every aggregate row and refuses to stay quiet
   about it. A headline improvement with 2/21 fillable is a calibration result, and must
   be called one.
2. **Calibration and tradeability are separate claims and neither implies the other.**
   Being right about a probability is the research product; being able to transact near
   that probability is the business. Report both, never let one stand in for the other.
3. **Wing legs are where the two diverge most.** The cheaper the leg, the wider the
   relative spread, and the more the midpoint flatters us. A 4c midpoint on a
   0.1c / 8c book overstates the sellable price by 40×.
4. **Score against the executable price, not the quote, when deciding whether to
   promote a variant.** `exec_edge` in `scores_detail.csv` is cents per share at the
   best price actually observed — that is the number a trial review lives or dies on.
5. **Fix it at the source: record the book.** Prediction rows are getting `bid`, `ask`
   and `depth_usd` columns (`predictions/README.md`). A trade-feed replay is an
   after-the-fact reconstruction; the book at prediction time is the ground truth, and
   the hourly snapshot worker already captures it for anything on the watchlist. **This
   is why the watchlist must be mirrored before a predicting run, not after.**

## The honest limit of this method

`fillcheck` sees trades, not orders. A resting bid that nobody ever hit leaves no trace
in the trade feed, so a row it reports as unreachable might have had a quiet bid sitting
there the whole time. **The count is a lower bound on fillability, not an exact
measure** — "at least 2 of 21 were reachable", never "exactly 2". It errs in the
conservative direction, which is the right direction for a claim about our own edge, but
it is not a substitute for recording the book.

## See also

- [phantom-midpoints](phantom-midpoints.md) — the dead-book version of this failure
- [thin-market-price-read](thin-market-price-read.md) — the original, softer warning
- [delayed-execution-test](delayed-execution-test.md) — the time-axis version of the
  same question: not "would anyone trade", but "would they still be there when we woke up"
- `execution/DESIGN.md` — the cost model that turns a reachable price into a return
