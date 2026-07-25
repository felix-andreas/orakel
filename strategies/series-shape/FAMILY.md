# series-shape

Trade the **shape of a series-score distribution** using the deep moneyline as the level.
Polymarket lists each best-of-N as a bundle on one event: a deep moneyline (1c spreads,
$33k–$81k median) plus thin derivatives — map handicap (fav −1.5 ≡ 2-0) and totals
(O/U 2.5 maps ≡ goes the distance) at 5–20× less volume. We take the crowd's level and
claim its allocation across series scores is wrong.

Born from `ideas/2026-07-25-esports-series-shape-2.md`. **Shape claim, not level claim** —
no team model, no player data, no external ratings. That matters: the firm's two
survivors are shape claims and its two kills were level claims.

The mechanism has three stacked parts (n=2,000 resolved series at T−1h):
favourite-longshot on the moneyline (+6.1pp); a **convex transfer** — derivatives price
the sweep coherently with the biased moneyline, and P(sweep) is convex in P(match), so
the error amplifies 2.5–3.3×; and an *independent* "goes the distance" premium (Over-2.5
overpriced in 8/8 cohort-months, t=−5.16, present even in months where the moneyline bias
was ≈0).

Variants:

- [`bo3-derivatives/`](bo3-derivatives/) — handicap and totals legs of BO3 esports series
  (trial, slot 3, started 2026-07-25).

Cross-variant lessons: (none yet)
