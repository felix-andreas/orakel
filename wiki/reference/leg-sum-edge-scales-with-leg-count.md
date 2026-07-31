# A leg-sum edge measured at mid is K·s̄/2 of pure artifact

_Added 2026-07-31 (market researcher, claude-opus-5 effort max) from the nested-board
dominance screen — `ideas/2026-07-31-nested-board-dominance-discarded.md`._

## The claim

A board where exactly **N** of **K** legs resolve Yes must satisfy `Σ pᵢ = N`. The obvious
test is to sum the midpoints and compare to N. **Do not.** The mid-based sum carries a
mechanical artifact equal to half the total spread, and that artifact **grows linearly with
the leg count**:

```
executable edge  =  |Σmid − N|  −  K · s̄ / 2          (s̄ = mean bid-ask spread per leg)
```

This is an identity, not a fit. `Σmid = (Σbid + Σask)/2`, so on the sell side the executable
edge `Σbid − N` differs from the apparent edge `Σmid − N` by exactly `(Σask − Σbid)/2`, and by
the same amount on the buy side. **Both directions are overstated simultaneously and equally**
— which is why both can lose at once, the diagnostic from
[`midpoint-is-not-a-fill.md`](midpoint-is-not-a-fill.md).

## Why it is worse than it sounds

The artifact scales with K. The boards where a leg-sum edge looks biggest — many legs, so many
chances for the sum to drift — are exactly the boards where the fake component is biggest. A
20-leg board with a 4c mean spread manufactures **40c** of apparent edge on a target of 4.00,
which is 10% of the board. No plausible real incoherence is that large.

That is the same anti-correlation shape as
[`depth-lives-where-the-edge-is-not.md`](depth-lives-where-the-edge-is-not.md), applied to a
different statistic: the property that makes the measurement look attractive is the property
that makes it fake.

## Measured, four independent boards, 2026-07-31

Live books, Polymarket, all `negRisk: false` (negRisk mechanically enforces Σ=1 and does
nothing for N>1, so N>1 boards are where leg-sum incoherence *can* live):

| board | K | N | Σbid | Σmid | Σask | \|Σmid−N\| | K·s̄/2 | **predicted** | **actual** |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| Alaska Governor primary (top-4) | 20 | 4 | 3.6850 | 4.0825 | 4.4800 | 0.0825 | 0.3975 | **−0.3150** | **−0.3150** |
| Alaska At-Large primary (top-4) | 6 | 4 | 3.1650 | 3.5915 | 4.0180 | 0.4085 | 0.4265 | **−0.0180** | **−0.0180** |
| Brazil presidential runoff (top-2) | 9 | 2 | 1.9540 | 2.0280 | 2.1020 | 0.0280 | 0.0740 | **−0.0460** | **−0.0460** |
| France 2nd round (top-2) | 37 | 2 | 2.2410 | 2.6140 | 2.9870 | 0.6140 | 0.3730 | **+0.2410** | **+0.2410** |

Four decimals, four boards, both signs of the trade. The Alaska At-Large row is the cleanest
illustration: a **+40.85c** apparent edge on a 6-leg board, against a **42.65c** artifact
floor — buying the basket loses 1.8c.

## How to use it

**Before fetching anything beyond one book snapshot**, compute `K · s̄ / 2`. If the
incoherence you are chasing is smaller than that, stop — there is nothing to reach, and no
amount of history will change it. Like the draw-count gate in
[`nested-ladders-trade-depth-for-power.md`](nested-ladders-trade-depth-for-power.md), this
kill needs no backtest.

If it *does* clear, two things still stand between you and the trade, and the France row hit
both:

1. **Depth is set by the thinnest leg, not the board.** A basket needs every leg filled, so
   the smallest book caps the size of all K. France cleared the arithmetic at +23.90c top of
   book and died between **100 and 250 baskets** — the binding legs held 1,244–1,352 shares
   while the headline leg held 75,760. Total extractable: **\$8.88**.
2. **A guaranteed profit is not an edge until it clears the risk-free rate on the capital it
   locks.** France returned **+0.35% annualised** on capital locked to April 2027, against a
   ~4% short-term risk-free rate — a **negative-carry** trade. Statistical objects never
   raised this question because their edge had to be established first; a dominance arb is
   true by construction, so size and time are the *only* questions left. Put the hurdle rate
   in the screen.

## Where such boards exist at all

Scarce, and worth knowing before you plan around them. Over the **full 6,788-event open
Polymarket universe** on 2026-07-31 there were **four** boards with a hard Σ=N>1 constraint,
and **one** rule-implied cross-board nesting (Alaska governor). 85 US races list both a
primary and a general board, but **81 of the 85 general boards are two-leg party boards**, on
which candidate nesting is structurally impossible.

Two mechanical traps when counting them:

- **Placeholder legs named like candidates.** Party boards carry `Will A win…` through
  `Will E win…` — single capital letters, which the documented `Person [A-Z]` filter misses.
  Gate on `volumeNum == 0 && liquidityNum == 0`, never on the name pattern. Filtering by name
  reported "0 party-only boards" when the true answer was 81.
- **Dormant boards rank last by `volume24hr`.** A volume-ranked scan found 4 of the 85 races.
  Population counts must page the whole universe, and offset paging caps at 2,000 — use
  date-windowed paging (see [`../recipes/polymarket-api.md`](../recipes/polymarket-api.md)).
