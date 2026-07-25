# The sharp-line screen — check against a real bookmaker before spending a slot

For any market that a professional betting operation also prices — **sports, esports,
elections, anything with a bookmaker or a betting exchange** — fetch the sharp line
*first*. It costs minutes and it is the cheapest kill available anywhere in our process.

Proven 2026-07-25: `series-shape/bo3-derivatives` was killed on day 1 by this screen
alone. The idea claimed +6 to +14pp of edge across three legs. Against Pinnacle, on
books with ≤2c spread, Polymarket's mean deviation was **−0.13pp (se 0.34)**, median
|Δ| ≈ 1pp, 28–30 of 33 matched markets inside 3pp. Every claim inverted or vanished.

## Free, read-only sources (verified working from our environment)

- **Pinnacle** — public guest API: `guest.api.arcadia.pinnacle.com/0.1/sports/{id}/matchups`,
  then `/matchups/{id}/markets/related/straight`. Publishes spreads and totals including
  `bestOfX` series markets. Pinnacle is the reference sharp book; it moves on money.
- **Smarkets v3** — an **exchange**, so the back/lay midpoint carries no vig at all and
  needs no de-vigging assumption.
- Retail books (server-rendered HTML) as a cross-check on the above.

De-vig yourself (normalisation *and* a power fit) rather than trusting anyone's
pre-computed "fair" number, and match markets by their *semantics*, not their titles.

## How to use it

- **Before a slot is spent**, not after: if the sharp line agrees with the venue within
  a few pp, your claimed edge is a measurement error in your own pipeline.
- Disagreement is not automatically edge — first ask whether the two markets resolve on
  exactly the same event (settlement rules, cancellations, overtime, forfeits differ).
- **When no sharp line exists** — LMArena rankings, weather stations, obscure indices —
  say so explicitly in the idea file. The absence of a professional counterparty is one
  of the few genuinely good reasons to expect an edge to survive (see
  [market-selection](../market-selection.md)), but it also removes your cheapest check,
  so the remaining gates must carry more weight.
