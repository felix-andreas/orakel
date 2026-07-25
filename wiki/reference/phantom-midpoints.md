# Phantom midpoints — the most expensive artifact we have found

**A leg with no resting orders quotes bid 0.05 / ask 0.95, and Polymarket's
`outcomePrices` and `prices-history` report that as a price of ~0.51.** It is not a
price. Nobody would trade there. Pool enough of those into a study and you will
manufacture an edge out of nothing.

Measured (series-shape/bo3-derivatives kill, 2026-07-25, n=9,592 esports series legs):

| book state | apparent "edge" |
|---|---|
| dead (never moves pre-match) | **−7.11pp** |
| near-flat | −3.19pp |
| **live (price moves pre-match)** | **+0.08pp (se 0.58)** |

The pooled headline was ~+14pp on the sweep leg. The true edge on tradeable books was
**0.0 ± 1.5pp**. 23% of those handicap legs never moved before the match; 85% were under
$5k volume. The claimed bias even *inverted with liquidity* — +6.5pp under $5k versus
−4.0pp above $50k — which is the signature of the artifact, not of a real crowd error.

## Rules

1. **Never trust a midpoint without its spread.** A wide spread is not a noisy price; it
   is the absence of one. Gate at spread ≤ 5c *and* real depth before a quote enters any
   study or any prediction row.
2. **Decompose every claimed edge by pre-match price *movement*, not just by volume.**
   Volume filters help but are not sufficient — split by "did this book ever move?" and
   report the live-book number as the headline. If the edge lives in the dead half, there
   is no edge.
3. **This corrupts scoring, not just backtests.** Our `market_price` column is a CLOB
   midpoint; a "paired improvement vs the market" computed against a phantom midpoint is
   meaningless in both directions. Prediction rows should carry the book state that
   produced their market price.
4. A market whose *whole family* is quoted 0.020/0.980 is not a cheap opportunity — it is
   an unlisted market (ladder-rv correctly refused to predict on such boards, 2026-07-25).

**Reproduced on a second sport, same day** (tennis match-totals, n=1,683 resolved legs,
market-researcher cycle 3): headline **−7.61pp**; DEAD legs (never moved pre-match)
**−27.46pp**; near-flat −16.66pp; LIVE (total variation ≥2c) **−5.00pp**. By leg volume the
"edge" **inverted**: $0 volume −17.26pp, $1–100 −7.99pp, $100–1k −4.36pp, **>$1k +11.78pp**.
8.5% of legs never moved. The artifact concentrates in the 0.50–0.60 price bucket — the one
that looks most fundable — because an empty book quotes ~0.05/0.95 and reports as ~0.50.
Counter-example worth knowing: weekly USGS earthquake-count ladders scored **0 / 314 dead
legs (100% live, median total variation 1.79)**, so the gate is discriminating, not
universal.

See also [thin-market-price-read](thin-market-price-read.md) — this page is the
quantified, sharper-edged version of that page's warning — and
[midpoint-is-not-a-fill](midpoint-is-not-a-fill.md), which is the harder case: a book
that is genuinely **alive**, quoting a real 0.001/0.08, whose 4c midpoint is still a
price nobody will give you. Measured on our own ledger: 21/21 rows beat the market,
2/21 were reachable.
