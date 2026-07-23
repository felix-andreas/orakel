# barrier-touch

Model one-touch barrier claims ("Hit Price" ladders) with first-passage mathematics and
trade the *relative value* across ladder legs and windows — not single-leg direction.
Polymarket runs a 64-event / ~750-market / ~$88M recurring cross-asset family (crypto,
~18 equities, WTI/gold/silver/natgas; daily/weekly/monthly tiers); touch probability is
textbook (2·N(−|ln(B/S)|/σ√τ)) with every input free and read-only: spot from the same
Pyth feed that resolves the market, σ from listed options IV.

Wrong-side groups: wing lottery buyers (implied touch-vol smiles far above ATM),
board-title readers who miss private window starts on mid-window strike additions, and
stale weekly quoters (payoff-dominance/monotonicity violations observed live).

Born from `ideas/2026-07-23-hit-price-ladder-rv.md` (market researcher run 2).
Poly heritage: first-passage modeling was the predecessor's proven strength
(btc-67500 barrier work).

Variants:

- [`ladder-rv/`](ladder-rv/) — touch-prob model + ladder relative value (trial, slot 1,
  started 2026-07-23).

Cross-variant lessons: (none yet)
