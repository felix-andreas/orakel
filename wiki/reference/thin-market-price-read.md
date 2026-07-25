# Reading the real price of a thin market

**Rule of thumb: spread > 10c or top-of-book < $100 ⇒ the midpoint is an artifact, not a
consensus.** The quantified version of this warning — including the case where a *dead*
book reports a ~0.50 midpoint and fabricates double-digit fake edge — is
[phantom-midpoints](phantom-midpoints.md). Read it before pooling midpoints into any study. Price discovery then lives in the *tape* and *related markets*, not the book.
(Origin: poly's Mt. Washington market — quoted "0.41" was the midpoint of a 0.25/0.57 book
with a $3 top bid while real money transacted at 0.51–0.54.)

## Steps

1. **Diagnose the book** (`clob.polymarket.com/book?token_id=...`; best bid/ask are the
   LAST array elements): spread, top-of-book dollars, depth within 10c, level count.
   Wide + shallow ⇒ downgrade the midpoint to "recorded by convention only".
2. **Read the tape** (`data-api.polymarket.com/trades?market=<condition_id>&limit=500`):
   taker VWAP (all-time, last-5d) and last trade — what money actually paid; **wallet
   concentration** (one wallet dominating taker volume = the "price" is one person's
   opinion); flow direction and drift (one-way flow with ~1c/day decay ⇒ prints lag fair
   value — correct for it). If the tape might be *fake*, run the
   [wash-trading tests](wash-trading.md) before trusting any VWAP.
3. **Sibling markets as a consistency oracle.** In "≥ X" threshold ladders, P(≥x) must
   decrease in x: a monotonicity violation (in mid, last, AND vwap) flags the mispriced
   leg and bounds the family's pricing error; fit a money-weighted isotonic regression
   over the family for one coherent curve. A lower rung jumping to ≥0.99 is an **event
   detector** — the partial event happened, and its jump date says when.
4. **Blend** estimators weighted by the real money behind each; state an interval no
   narrower than the family's proven pricing error.

## Conventions

- Still record the CLOB midpoint as `market_price` in the predictions CSV (standard,
  comparable) — but never treat "model far from midpoint" in a hollow book as tradeable
  edge: the executable side may sit on the *other* side of fair. Quote the tape for "what
  does the market think"; quote the touch for "what can I trade".
- **Thin ≠ stale.** Some thin books are live: many distinct wallets, low concentration,
  spread ≤ ~5c with real dollars at touch, prints clustered around information events and
  moving the right way. poly's WC goals market (~$3k book) repriced within *minutes*
  during live matches. A thin book passing these tests deserves midpoint-level respect.
