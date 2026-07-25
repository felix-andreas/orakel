# series-shape

> **In plain English:** Bets on a best-of-three match: who wins, and separately whether it ends in a clean sweep. The two must agree with each other, and one is traded far more than the other.

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

**That mechanism is false, and the family is empty as of day 1.** Everything above is the
founding hypothesis, kept for the record; see the post-mortem below before reviving it.

Variants:

- [`bo3-derivatives/`](bo3-derivatives/) — handicap and totals legs of BO3 esports series.
  Slot 3, started **and retired** 2026-07-25.
  [Post-mortem](bo3-derivatives/results/backtest-2026-07-25.md).

## Cross-variant lessons

1. **The "thin derivative next to a deep moneyline" setup is not itself an edge.** On the
   BO3 legs that have a real book, Polymarket *is* the sharp line: median |Polymarket −
   Pinnacle| = **1.08pp** on the map handicap, **−0.13pp mean on ≤2c-spread books**, and
   the market→realised gap on 1,110 resolved live-book series is **0.0pp ± 1.5pp**. Book
   thinness relative to a sibling market is not evidence that the thin book is wrong.
2. **A pooled mean over Polymarket midpoints is a mean over *quotes*, and some quotes are
   not prices.** A leg with no resting orders reports a ~0.50 midpoint (mean of a 1c bid
   and a 99c ask). 23% of these handicap legs never moved pre-match; 85% were under $5k.
   Pooling them fabricated a +14pp "edge" in a family whose true edge is zero. **Any future
   variant in this family must decompose its headline number by pre-match price *movement*,
   not just by volume, before the number means anything.**
3. **Series format must be read off the LEGS.** A BO5 map handicap is also "wins 2 or more
   maps"; only handicap-margin 2 ∧ totals-threshold 3 identifies a BO3. Title parsing
   imports 1,597 BO5/BO7 series and breaks the `HC_cover ⇔ win ∧ Under` identity
   mechanically (99.9% → 97.2%).
4. **Sports claims now have a cheap external falsifier — use it on day 1.** Pinnacle's
   guest arcadia API and the Smarkets v3 exchange API both serve full pre-match lines
   read-only with no account (endpoints in `bo3-derivatives/memory/MEMORY.md`). Any future
   series-shape variant should be priced against them before a slot is spent, not after.
