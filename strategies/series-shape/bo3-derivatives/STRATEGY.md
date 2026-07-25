# series-shape/bo3-derivatives

> Thesis (from `ideas/2026-07-25-esports-series-shape-2.md` — read it fully): take the
> deep esports moneyline as the level; trade the thin map-handicap and totals legs whose
> implied series-score distribution is mis-allocated. Three stacked distortions:
> moneyline favourite-longshot (+6.1pp), a **convex transfer** amplifying it 2.5–3.3× on
> the sweep leg (0.80–0.90 band: ML +5.6pp → handicap +13.8pp), and an independent
> Over-2.5 "goes the distance" premium (8/8 months, t=−5.16). **Shape claim.**

## Method

DAY-1 STATE — to be established. **Gate 5 runs first**: an external closing bookmaker
map-handicap line is the cheapest possible kill (if Pinnacle agrees within 3pp, the
edge is our misreading, not the crowd's). The idea's remaining gates follow, and gate 0
is an explicit artifact hunt — an edge this size on 1c-spread books should not exist,
so the null is "we are wrong" until it survives.

Fee reality (new, `wiki/recipes/polymarket-api.md`): sports rate 0.05 →
`shares × 0.05 × p × (1−p)`, ~1.2c/share at p=0.5. Every claimed edge must clear it at
the traded price; the live example nets ≈+13.3c after 1.21c of fee.

## Applicability

A market fits when: it is a handicap or totals leg of a BO3 series whose moneyline is
deep (≥$20k, ≤2c spread), the derivative leg quotes ≤5c spread with real depth, and the
favourite sits in a band where the empirical transfer is measured. Legs must be typed
from Gamma's `sportsMarketType` and each `description` — never from titles. The identity
`HC_cover ⇔ (fav wins) ∧ (Under 2.5)` held 6,705/6,710 and is the coherence check.

## How to run

(to be written with the first scripts in `src/`)

## Evidence

- (day-1 results land in `results/`)

## Changelog

- 2026-07-25 — created from the idea; slot 3 trial started.
