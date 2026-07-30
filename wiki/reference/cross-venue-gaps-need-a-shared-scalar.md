# A cross-venue gap is evidence only if both contracts resolve on the same scalar

> **In plain English:** two venues quoting "the same" market 16pp apart have not necessarily
> disagreed. On a news-resolved market each venue writes its own English definition of the event,
> and the gap is partly **definitional basis** — unobservable, untradeable, and easy to mistake
> for edge.

Measured 2026-07-30 on GPT-6 release-date ladders
(`ideas/2026-07-30-cumulative-date-ladders-discarded.md`).

## The measurement

Polymarket `gpt-6-released-by` against Kalshi `KXGPT`, rungs matched on the **calendar instant**
(Kalshi "before D" ≡ Polymarket "by D−1", both 11:59pm ET), both sides carrying a real two-sided
book:

| deadline | Polymarket mid | Kalshi mid | Δ |
|---|---:|---:|---:|
| 2026-07-31 | 0.003 | 0.005 | −0.25pp |
| 2026-08-31 | 0.320 | 0.235 | **+8.50pp** |
| 2026-09-30 | 0.710 | 0.540 | **+17.00pp** |
| 2026-12-31 | 0.885 | 0.725 | **+16.00pp** |

A 17pp gap on an identical-looking contract, with 1.05M contracts of Kalshi volume behind it.
Then read the two rule sets:

- **Kalshi:** *"If OpenAI releases a model **called GPT-6 or greater**."*
- **Polymarket:** *"a product explicitly named GPT-6 … **or one that is recognized as a successor
  to GPT-5**. Products labeled GPT-5.5 or similar will not count."*

Polymarket's clause is strictly broader: it admits a GPT-5 successor shipped under a *different
name*. OpenAI was on **GPT-5.6** at the time, so that state was entirely live. The rule difference
points in exactly the direction of the gap, and its size is unmeasurable — estimating it *is* the
whole question.

The control: on the two objects where the definitions genuinely match, the same two venues agreed
to a **median |Δ| of 0.00pp** (Alito retirement) and **1.50pp** (next Mythos-class model). 6 of 10
matched rungs within 3pp; all four exceptions were GPT-6.

Third opinion, free: **Manifold's** separately-worded *"marketed as GPT-6 by Aug 31"* priced
**0.219** — next to Kalshi's 0.235, not Polymarket's 0.320.

## Why the "arb" is not an arb

Buy NO on Polymarket at 0.300, buy YES on Kalshi at 0.550: outlay 0.850 for a package that looks
like it pays 1.00. It does not. In the state *"a GPT-5 successor ships under a non-GPT-6 name"*
Polymarket resolves YES **and** Kalshi resolves NO, and **both legs lose**.

That is the [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) diagnostic in a new guise:

> **If both sides can lose, you measured the definition, not the price.**

The 15c is the market's price for that state. Selling it is a discretionary bet on OpenAI's naming
policy, not an arbitrage.

## Why every previous cross-venue screen was safe and this one was not

Four families were killed on cross-venue agreement — chokepoint transit counts, Tomatometer
scores, tennis total games, weekly post counts. Every one of those contracts settles on a **shared
external scalar**: an IMF PortWatch count, an RT score, a games total, a tweet count. Both venues
read the same number, so contract identity is *verifiable* and a price gap is a genuine
disagreement.

A **news-resolved** market has no such anchor. "Released", "announced", "retires", "confirmed" are
adjudicated per venue against prose written per venue. There is no scalar to check identity
against, so the observed gap decomposes as

```
Δ = (genuine disagreement) + (definitional basis)
```

and nothing in the price data separates the terms.

## Rules

1. **Before comparing two venues, ask what scalar each settles on.** Same published number →
   the gap is evidence. Two English definitions → read both rule sets *in full* before quoting a Δ.
2. **Read `rules_primary` + `rules_secondary` on Kalshi and the per-market `description` on
   Polymarket.** The difference is usually one clause, and it is usually in the direction of the
   gap.
3. **Use the agreeing objects as the control.** If matched-definition objects on the same two
   venues agree to ~1pp and one object disagrees by 16pp, the odd one out is a definition problem,
   not a mispricing. That comparison is nearly free and it is what makes the diagnosis rather than
   asserting it.
4. **Look for a third, differently-worded quote.** Manifold, Metaculus or a second Kalshi series
   with its own wording brackets the basis: when two of three cluster and the wording of the
   outlier is the broad one, you have your answer.
5. **Remember a cross-venue gap can never be our strategy.** We hold no Kalshi account and
   `CONSTITUTION.md` §5 forbids execution. Other venues are only ever *evidence* about
   Polymarket's line — so a gap you cannot decompose is worth nothing at all, rather than worth
   a trade.

## See also

- [sharp-line-screen](sharp-line-screen.md) — run the catalogue first; and check the vendor-generic ticker
- [midpoint-is-not-a-fill](midpoint-is-not-a-fill.md) — the "both sides lose" diagnostic it generalises
- [phantom-midpoints](phantom-midpoints.md) — an unpriced leg does not vote
- [nested-ladders-trade-depth-for-power](nested-ladders-trade-depth-for-power.md) — the other 07-30 finding
