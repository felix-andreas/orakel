---
date: 2026-08-01
slug: deep-tail-carry
status: discarded-idea
model: claude-opus-5 (effort max)
example_markets: ["will-jesus-christ-return-before-2027", "will-china-invade-taiwan-before-2027", "hantavirus-pandemic-in-2026", "xi-jinping-out-before-2027", "will-trump-acquire-greenland-before-2027"]
---

# Object 16 — deep-tail carry: fading the ≤5c standalone binary

**Verdict: discarded at idea stage** — but this is the closest any object has come. It
**clears W2 (execution)** by the largest margin the firm has recorded, and it is the **first
object to clear W3 (power) outright**. It dies on **W1 (incumbent), whose direction is
fatal and independent of everything else**, and on **W4 (carry) at the horizon the venue
actually offers**.

The headline is the gap between the point estimate and the bound, which is the firm's
standing rule doing its job: **0 of 169 settled tail legs resolved Yes**, worth **+12.97pp
over the risk-free rate at executable prices** — and the 95% upper bound still fails the
break-even event rate at every horizon on the venue.

**A correction I am recording rather than hiding:** I ran W3 first off the *live* cohort and
got ≤70 quasi-independent themes per annual cohort, i.e. "4 to 22 years — dead". That was
wrong. The settled census (below) shows ~1,120 themes/year. W3 is not this object's problem,
and I would have filed a right verdict for a wrong reason had I stopped there.

## Thesis

Polymarket carries a large population of **standalone binary** markets whose YES leg trades
in the 1–5c band: "Will Jesus Christ return before 2027?", "Hantavirus pandemic in 2026?",
"Will Trump acquire Greenland before 2027?". The claim is that lottery/entertainment demand
makes the crowd pay too much for the YES ticket, so the mirror trade — **buy NO at the ask
and hold to redemption** — harvests a premium.

Classification (per the standing LEVEL-vs-SHAPE rule): this is neither. It is a **carry
claim**. We assert no informational edge on whether China invades Taiwan; we assert the
crowd's *tail allocation* is too fat and that the resulting yield beats holding cash. That
makes it the second object after #15 whose whole argument is size-and-time rather than truth
— and the first where the carry is short-dated and the book is deep.

Why it looked worth a day, given fifteen dead objects: it is the one family where the edge,
if it exists, sits **exactly where the depth is**. Every previous execution kill turned on
depth concentrating at a board's mode while the mispricing lived in the wings. A standalone
binary has no mode elsewhere — the tail leg *is* the market.

Distinct from prior work: #15 was same-venue *dominance* (Σ=N baskets, primary⊂general);
this is a single-leg statistical claim with no coherence constraint. Distinct from
`arena-rank/favourite-shrinkage` (parked), which shrinks favourites *within* a multi-outcome
ranking family; this is standalone binaries with no siblings.

## Population (measured today, full open universe)

Full date-windowed pull: **9,347 open events / 96,642 markets**. Of these, **1,118** are
standalone single-market events. Filtering to an open order book and a YES mid ≤ 5c with a
future end date:

- **127 legs, holding $206,696,574 of lifetime volume.**
- **13 distinct end dates. 94 of the 127 legs (97.1% of the volume) share one: 2026-12-31.**
- Volume-weighted NO ask **0.9686** (implied YES **3.14c**), volume-weighted horizon **150 days**.
- Median spread **0.60c** — which is **20.0% of the YES leg** and **0.62% of the NO leg**.

## Example markets (today's numbers)

| market | YES mid | NO ask | days | vol | $ at NO ask | VWAP $10k |
|---|---:|---:|---:|---:|---:|---:|
| Will Jesus Christ return before 2027? | 1.95c | 0.9810 | 152 | $64,865,722 | $5,445,228 | 0.9810 |
| Will China invade Taiwan by end of 2026? | 3.95c | 0.9610 | 152 | $39,285,514 | $16,123,889 | 0.9610 |
| Will Trump acquire Greenland before 2027? | 4.20c | 0.9590 | 152 | $35,150,074 | $861,749 | 0.9651 |
| Hantavirus pandemic in 2026? | 3.60c | 0.9650 | 152 | $17,472,983 | $19,352,132 | 0.9650 |
| Xi Jinping out before 2027? | 4.55c | 0.9550 | 152 | $11,626,549 | $15,321,526 | 0.9554 |
| Will Reza Pahlavi lead Iran in 2026? | 4.35c | 0.9570 | 152 | $11,670,192 | $699,147 | 0.9652 |

---

## The four walls, in the order they were run

### W3 — power. **PASS.** The first object to clear this wall.

The strategy's break-even is not a win rate, it is an **event rate**. Buying NO at effective
price `a_eff` and holding `d` days beats a risk-free rate `r` iff the realised YES rate

```
π  ≤  π* = 1 − a_eff · (1 + r·d/365)
```

This one line makes W3 and W4 the same computation, which is the reusable part. Note
immediately what it implies: **π\* shrinks with horizon**, so there is a maximum holding
period beyond which a given tail price loses to cash *even if it never resolves Yes*:

| implied YES | NO ask | `d_max = 365(1−a_eff)/(a_eff·r)` |
|---:|---:|---:|
| 0.50c | 0.995 | **44 days** |
| 1.90c | 0.981 | 168 days |
| 3.40c | 0.966 | 305 days |
| 5.00c | 0.950 | 456 days |

"Human moon landing in 2026?" (0.47c) and "10.0 or above earthquake before 2027?" (0.52c)
have **152 days to run against a 44-day d_max**. They are arithmetically guaranteed to lose
to T-bills, today, with no assumption about the world at all.

**The draw count, measured rather than guessed.** My first pass counted the *live* cohort —
127 legs collapsing to ≤70 themes, 94 of them on 2026-12-31 — and concluded 4 to 22 years.
That was the wrong denominator: it counts what is open, not what settles. Census of the
closed universe over the last 12 months (99,493 closed events scanned):

- **36,969 standalone closed binaries; 3,280 after removing sport/crypto/weather (~273/month).**
- In a random sample of **356** of them with a price checkpoint at T−45d, **169 (47.5%)** sat
  at a YES mid ≤5c. Forty-five days out, most "will X happen by" binaries have already
  effectively decided.
- Thematic collapse on that sample: 169 legs → 10 clusters + 112 singletons = **122
  quasi-independent** (deflator 1.39×, much milder than the live cohort's, because the settled
  population is not concentrated on one year-end date).

**→ ~1,558 band legs and ~1,120 quasi-independent themes per year.** Required n at a 45-day
hold, buying at the ask (π\* = 1.57%): **243 themes with zero losses — 0.2 years.** The
supply of evidence is not the constraint. This is the first object where it isn't.

### The measurement W3 made possible — and it is the best point estimate the firm has taken

Same sample, same checkpoint, the actual backtest:

> **0 of 169 settled ≤5c legs resolved Yes.** Mean checkpoint YES mid 2.46c.

At executable prices (mid + half the live band's 0.60c median spread, plus the per-fill taker
fee), on a 45-day hold that is **16.97% net annualised, +12.97pp over the risk-free rate**.

And it does not clear the bound:

| leg count used | Wilson 95% upper on 0 Yes | rule of three |
|---|---:|---:|
| 169 raw legs (most generous) | **2.22%** | 1.78% |
| 122 quasi-independent themes (honest) | **3.05%** | 2.46% |

Against π\* = **1.57%** at 45 days and **0.44%** at 150 days, both at the ask. **The bound
fails at every horizon — by 1.4× at the friendliest, by 5–7× at the venue's actual one.**

This is `break-even-win-rate.md` firing exactly as designed, and it is the same shape as the
07-30 kill: a result in the pre-registered direction, with a clean mirror test, that the
interval refuses. A 0-for-169 record is not proof the rate is zero; it is proof the rate is
probably under 2.2%, and the trade needs it under 1.57%.

One further reason not to lean on the zero: the sample is **one 12-month regime**
(`sharpen-only-what-persists.md`). These are exactly the events that resolve Yes in *clusters*
— an eventful year produces several at once — so a quiet window understates the rate and the
zero is the least durable number in the file. A single Yes anywhere in the sample takes the
Wilson upper to 3.28% (169) / 4.50% (122), and pushes the requirement to 358 zero-ish themes.

### W1 — incumbent. Live, larger, and pointing the wrong way.

Kalshi catalogue in one call: **12,368 series** (12,355 on 07-31, +13/day). Searching by
person/franchise/vendor rather than by our board titles — the 07-30 rule — finds live twins
with real contracts, not the 0-market shells of object 15:

| object | Polymarket | Kalshi ticker | Kalshi vol (contracts) | Kalshi OI | Kalshi mid | Δ |
|---|---:|---|---:|---:|---:|---:|
| Trump out / resign before 2027 | 3.75c | `KXTRUMPOUT27-27-DJT` | 5,601,517 | 2,398,208 | 6.45c | **+2.70** |
| Greenland acquisition | 4.20c | `KXGREENTERRITORY-29-27` | 498,682 | 146,548 | 5.55c | **+1.35** |
| Greenland purchase | 4.20c | `KXGREENLAND-29-27` | 1,467,146 | 400,446 | 4.60c | **+0.40** |
| Pahlavi leads Iran 2026 | 4.35c | `KXPAHLAVIHEAD-27JAN-RPAH` | 1,796,928 | 515,253 | 4.50c | **+0.15** |
| US recognizes Pahlavi 2026 | 4.50c | `KXRECOGPERSONIRAN-26` | 998,939 | 207,900 | 6.50c | **+2.00** |
| Hantavirus pandemic 2026 | 3.60c | `KXNEWOUTBREAKHANTA-26` | 1,628,340 | 622,715 | 6.00c | **+2.40** |
| Ebola pandemic 2026 | 3.00c | `KXNEWOUTBREAKEBOLA-26` | 58,482 | 27,305 | 4.50c | **+1.50** |
| Aaron Rodgers retires | 1.35c | `KXARODGRETIRE-26` | 152,029 | 37,057 | 3.00c | **+1.65** |

**8 of 8 matched pairs price the tail ABOVE Polymarket. Mean Δ +1.52pp, median +1.57pp,
sign-test p = 0.0039 one-sided.** (`KXALIENS-27` carries 26,346,026 contracts at 7.15c, the
largest tail book found on either venue.)

This is the kill, and its *direction* is the point. The thesis says Polymarket's tail is too
expensive. A live venue with more contracts on the identical objects says it is too **cheap**.
There is no version of "the incumbent confirms us" available here.

Two honest caveats, both of which I checked rather than waved at:

- Per `cross-venue-gaps-need-a-shared-scalar.md`, a matching line is not a matching contract.
  The two Kalshi Greenland series differ from *each other* by 0.95pp (4.60 vs 5.55) purely on
  rules text, which sizes the definitional noise at roughly 1pp. Six of the eight gaps exceed it.
- Kalshi pays interest on posted collateral; Polymarket USDC earns nothing. That should make
  Polymarket's tail the **more** expensive of the two, since our NO buyer forgoes the
  risk-free rate and needs compensating. It is the cheaper one. The carry adjustment widens
  the gap against us rather than explaining it.

### W2 — execution. **PASS**, and by the largest margin the firm has recorded.

Walked the NO book at $100 / $500 / $2,000 / $10,000 before any modelling:

| leg | $ resting at NO ask | VWAP $100 | VWAP $2,000 | VWAP $10,000 |
|---|---:|---:|---:|---:|
| Jesus returns | $5,445,228 | 0.9810 | 0.9810 | 0.9810 |
| China invades Taiwan | $16,123,889 | 0.9610 | 0.9610 | 0.9610 |
| Hantavirus pandemic | $19,352,132 | 0.9650 | 0.9650 | 0.9650 |
| Xi Jinping out | $15,321,526 | 0.9550 | 0.9550 | 0.9554 |
| Iranian regime falls by Sep 30 | $11,430,683 | 0.9690 | 0.9696 | 0.9704 |

Zero slippage to $10,000 on the five deepest legs. Compare object 13: a **$7** median at the
ask on the legs where the edge lived. The reason is structural and worth keeping — see the
wiki note below.

The mirror diagnostic (`midpoint-is-not-a-fill`) does not fire: this is not a spread
artifact. The median spread is 0.60c, which is 0.62% of the NO leg. You genuinely can buy
this at these prices, in size. It is simply not worth buying.

### W4 — carry. Marginal, and it collapses into W3.

Net of the per-market taker fee (`shares × feeRate × p × (1−p)`, read from `feeType` — the
band spans `culture_fees` 0.05, `politics_fees` 0.04, `crypto_fees_v2` 0.07, and several
others; the fee is small at the extremes, 0.02–0.20c/share):

| leg | a (VWAP $2k) | fee/share | net annualised | vs ~4% risk-free |
|---|---:|---:|---:|---:|
| Jesus returns | 0.9810 | 0.093c | **4.42%** | **+0.42pp** |
| Epstein alive | 0.9790 | 0.103c | 4.89% | +0.89pp |
| Trump impeached by end 2026 | 0.9844 | 0.062c | 3.66% | **−0.34pp** |
| Human moon landing 2026 | 0.9953 | 0.024c | 1.08% | **−2.92pp** |
| 10.0+ earthquake before 2027 | 0.9948 | 0.026c | 1.20% | **−2.80pp** |
| Xi Jinping out | 0.9550 | 0.172c | 10.86% | +6.86pp |
| Iranian regime falls by Sep 30 | 0.9696 | 0.147c | 18.10% | +14.10pp |

Equal-weight over all 127 legs, **assuming zero losses**: mean 8.07%, median 7.13%,
volume-weighted 7.61% annualised.

The shape of that table is the whole story. **The safest leg on the venue pays +0.42pp over
T-bills; every leg paying materially more is paying for genuine event risk.** The yield curve
across the tail is a risk curve, not a mispricing. Object 15 died at +0.35% annualised on a
guaranteed arb; this object's *risk-free-equivalent* corner pays 4.42% — better, and still
not a business.

**And the escape route is closed by the venue's own shape.** Since π\* rises as the horizon
shortens, the obvious fix is "trade the short end". Splitting the live band by horizon:

| horizon | legs | volume | % of band volume | median implied YES | **π\* at the ask** |
|---|---:|---:|---:|---:|---:|
| ≤45d | 13 | $970,529 | **0.5%** | 1.95c | 0.72% |
| 46–90d | 7 | $3,398,426 | **1.6%** | 3.25c | **1.83%** |
| 91–149d | 3 | $151,230 | 0.1% | 3.80c | 1.18% |
| **≥150d** | **104** | **$202,176,389** | **97.8%** | 3.40c | **0.75%** |

**Everywhere the venue has size, π\* ≤ 0.75%.** The one slice where the hurdle is meetable at
all (46–90d, π\* 1.83%) carries **1.6% of the band's volume** — and even there the measured
Wilson upper of 2.22–3.05% sits above it. The short end is not merely thin: it is also
*safer* (1.95c median at ≤45d), so it pays less premium exactly where it would need to pay
more.

Restated as time-to-clear at ~1,120 themes/year:

| construction | zero losses ever | one loss per 200 | one loss per 100 |
|---|---:|---:|---:|
| 45-day hold at the ask (π\* 1.57%) | 243 → **0.2 yr** | 461 → 0.4 yr | 1,737 → 1.5 yr |
| 150-day hold at the ask (π\* 0.44%) | 874 → 0.8 yr | **unreachable** | **unreachable** |

At the horizon where 97.8% of the money is, the strategy is investable **only on the
assumption that it never loses, ever**. That is not a bound anyone should underwrite on a
book containing "Will China invade Taiwan".

---

## Falsification sketch (what would have had to be true)

Pre-registered form, had this reached a slot:

1. Take every settled Polymarket standalone binary over ≥ 3 years. At a fixed checkpoint
   (T−150d), keep those with a YES **ask** ≤ 5c — an observable rule, no look-ahead. The
   selection must never use the settled outcome; "keep the ones that obviously couldn't
   happen" is `lifetime-volume-is-look-ahead` in a new costume.
2. Buy NO at the **ask**, size at the walked VWAP for $2,000, pay the per-market taker fee,
   hold to redemption. Report the realised YES rate π̂ with its Wilson 95% upper bound, and
   the count of *independent themes*, not legs.
3. **Kill if the Wilson upper bound on π̂ exceeds π\*** at the horizon where the tradeable
   size actually is.

Steps 1 and 2 were run today at a T−45d checkpoint on a 356-market sample, which is why this
is filed `discarded-idea` rather than `needs-gate-0`: the incumbent was measured, the book was
walked, and the backtest was *run* rather than proposed. It fails step 3 — Wilson upper
2.22% (169 legs) / 3.05% (122 themes) against π\* = 1.57% at 45 days and 0.44% at 150 days.

## How close it came

Closest of the sixteen, and worth stating precisely so the funnel reads correctly:

- **W2 — cleared outright**, by the largest margin recorded ($19.4M at the ask, zero slippage
  to $10,000, against object 13's median $7).
- **W3 — cleared outright**, the first object to do so: ~1,120 quasi-independent draws a year
  against the 243 needed. My initial read of this wall was wrong and the correction is in the
  header.
- **The point estimate is the best the firm has measured on any object**: 0/169, +12.97pp over
  risk-free at executable prices, with no mirror-test failure and no phantom midpoints.
- **W1 — failed, and independently of everything above.** 8 of 8 matched pairs put a larger
  venue *above* us on the identical objects (p = 0.0039). No sampling choice of mine can
  rescue that, and it points the opposite way to the thesis.
- **W4 — failed where the money is.** π\* ≤ 0.75% across 97.8% of the band's volume; the
  measured bound is 2.22–3.05%; and at that horizon the trade requires a never-loses
  assumption.

So it did not die of being small, unreachable, or unpriced. It died of being **unprovable at
the horizon on offer, while the one venue we can check says we are on the wrong side.** If
the CEO wants one line for the backlog question: this object got further than any predecessor
and still is not a strategy, and the reason is a property of the venue's maturity schedule
rather than of our modelling.

## What I would want to be wrong about

The one construction this does not test is **maker-side**: posting the NO bid rather than
lifting the ask converts a 0.62%-of-notional spread into a rebate and pays no fee at all.
That is now the *third* consecutive family (12, 13, 16) whose only untested version is
maker-side, and it does not change the W3 verdict — the draw count is a property of the
world, not of the order type. Noted for the standing question with Felix, not as a
counter-argument.
