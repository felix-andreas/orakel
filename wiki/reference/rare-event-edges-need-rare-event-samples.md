# A rare-event edge needs a rare-event sample — convert the hurdle into a break-even *rate*

> **In plain English:** "that 3c market is obviously overpriced" is a claim about a number you
> cannot measure. We went and measured it: **0 of 169** settled tail legs resolved Yes, worth
> **+12.97pp a year over cash at prices you can actually hit** — and the 95% upper bound *still*
> failed the hurdle, at every horizon the venue offers. A perfect record over 169 rare-event
> draws is not enough. Tail fades are rarely killed by being wrong; they are killed by being
> unprovable, and by the venue selling depth only at the tenor where the hurdle is unmeetable.

Measured 2026-08-01 on deep-tail standalone binaries
(`ideas/2026-08-01-deep-tail-carry-discarded.md`). Companion to
[break-even-win-rate](break-even-win-rate.md), which says report the bound; this page says
what the bound costs when the quantity is small.

## The two walls are one computation

The firm has been running these as separate screens:

- **W3, power** — count independent draws, compute the n the bound needs, divide by the
  arrival rate (`nested-ladders-trade-depth-for-power.md`).
- **W4, carry** — a guaranteed or near-guaranteed profit is not an edge until it beats the
  risk-free rate on the capital it locks (object 15: +0.35% annualised against ~4%).

On any object whose payoff is "collect a small premium unless a rare thing happens", they are
**the same calculation**, and the bridge is to state the hurdle as a probability rather than
as a return. Buying the safe side at effective price `a_eff` (ask plus the per-fill taker fee)
over `d` days beats a risk-free rate `r` if and only if the realised event rate satisfies

```
π  ≤  π*  =  1 − a_eff · (1 + r · d/365)
```

`π*` is the **break-even event rate**. It is the carry hurdle and the statistical null in one
number, and it is what you must bound from above — never estimate from the middle.

## Why the bound is so expensive

Measured on the live band: 127 standalone Polymarket binaries at a YES mid ≤ 5c, holding
$206.7M, volume-weighted NO ask 0.9686 (implied YES 3.14c) over 150 days.

| band | implied YES | a_eff | **π\*** |
|---|---:|---:|---:|
| volume-weighted | 3.14c | 0.9701 | **1.39%** |
| equal-weighted | 3.40c | 0.9676 | **1.72%** |
| the safest single leg on the venue | 1.90c | 0.9819 | **0.17%** |

Independent draws required for the Wilson 95% **upper** bound to fall below `π*`:

| true event rate | π\* = 1.39% | π\* = 1.72% | π\* = 0.17% |
|---|---:|---:|---:|
| **0.0% (never loses)** | **279** | **224** | **2,267** |
| 0.5% | 642 | 431 | — |
| 0.8% | **1,538** | 682 | — |
| 1.0% | 3,338 | 1,140 | — |
| 1.5% | — | 13,851 | — |

The top row is the one to internalise. With **zero** adverse outcomes the upper bound is the
rule of three, `n ≥ 3/π*`: **215 / 174 / 1,764**. There is no data-collection strategy, no
modelling skill and no fee saving that gets under it — it is arithmetic on the confidence
interval. A tail fade that is *perfectly correct* still cannot be shown to clear T-bills
without hundreds of clean draws.

And the requirement is **doubly** punishing, because `π*` shrinks as the leg gets safer. The
better the trade looks — the closer the tail is to genuinely impossible — the smaller the
premium, so the smaller `π*`, so the *larger* the sample needed. The safest leg on the venue
needed 1,764 draws. That inversion is the trap: intuition says "this one is obviously free
money, I need less evidence", and the arithmetic says the opposite.

## The horizon is a free parameter, and it is the one that decides

Because `π*` falls as `d` rises, every tail price has a **maximum holding period** past which
it loses to cash *even with a zero loss rate*: `d_max = 365(1−a_eff)/(a_eff·r)`.

| implied YES | NO ask | d_max |
|---:|---:|---:|
| 0.50c | 0.995 | **44 days** |
| 1.90c | 0.981 | 168 days |
| 3.40c | 0.966 | 305 days |

Two Polymarket legs sat at 0.47c and 0.52c with **152 days to run against a 44-day d_max** —
guaranteed losers to T-bills on arithmetic alone, before any view about the world.

So "shorten the hold" is the obvious repair, and **checking whether the venue lets you** is
the screen. Splitting the band by horizon killed the repair outright: the ≤45d slice held
**0.5%** of the band's volume and was *safer* (1.95c median, π\* 0.72%), the 46–90d slice
where π\* peaked at 1.83% held **1.6%**, and **97.8% of the money sat at ≥150 days with
π\* ≤ 0.75%**. The venue prices depth where people want to gamble — year-end "before 2027"
boards — which is systematically the horizon where the carry hurdle is unmeetable.

> **Check the horizon distribution of the *volume*, not of the legs.** A premium trade can be
> viable at the tenor nobody trades and dead at the tenor everybody does.

## Count the draws from what SETTLES, not from what is open

A correction worth recording, because it nearly produced a right verdict for a wrong reason.
Counting the *live* cohort gave 127 legs → ≤70 quasi-independent themes (13 end dates, 94 legs
and 97.1% of volume on one day) → "4 to 22 years, dead". Wrong denominator: the open book is a
snapshot, not a rate.

Censusing the **closed** universe instead — 99,493 closed events, 36,969 standalone binaries,
**3,280** after removing sport/crypto/weather, ~273/month — and sampling 356 of them at a
T−45d checkpoint found **47.5% already inside the ≤5c band** (45 days out, most "will X happen
by" binaries have effectively decided). Thematic deflator on the settled sample was only
**1.39×**, far milder than the live cohort's, precisely because settled markets are *not*
piled on one year-end date.

**≈1,558 band legs and ≈1,120 quasi-independent themes per year** — against 243 needed. The
evidence supply was never the constraint. Take the arrival rate off the settlement record;
`nested-ladders-are-one-draw` still applies to the *clustering*, but estimate the clustering
on the settled panel, where the calendar artifact is absent.

## What the sample then showed, and why it still failed

**0 of 169** settled ≤5c legs resolved Yes; mean checkpoint 2.46c; **+12.97pp over risk-free
at executable prices** on a 45-day hold. And:

| leg count | Wilson 95% upper on 0 Yes |
|---|---:|
| 169 raw legs | **2.22%** |
| 122 quasi-independent themes | **3.05%** |

against π\* = 1.57% (45d) and 0.44% (150d). **A perfect record over 169 rare-event draws is
not enough.** One single Yes takes the upper bound to 3.28% / 4.50%. And a quiet 12-month
window is the least durable evidence available for events that resolve Yes in *clusters* — see
[sharpen-only-what-persists](sharpen-only-what-persists.md).

## How to run it

1. Before anything else, write down `π*` from the ask, the fee, the horizon and the risk-free
   rate. One line of arithmetic, no data. Then `d_max` — some legs are dead on arithmetic.
2. Look up `3/π*`. That is your floor on independent draws **assuming you never lose once**.
3. **Compute `π*` per horizon bucket and weight the buckets by VOLUME.** If the hurdle is only
   meetable at a tenor carrying 1.6% of the book, the strategy has no capacity even if it has
   an edge.
4. Count the object's independent draws from the **settled** record — themes, not legs — and
   divide by the arrival rate. Do not count the open book: it is a snapshot, and on year-end
   families it is a badly clustered one.
5. If the answer is measured in years, stop. Do not fit a model, do not walk a book, do not
   look for the incumbent. The object may well be genuinely mispriced and you will still never
   be able to say so.

## The general shape

This applies to **any** strategy whose edge lives in the tail of the price distribution —
selling longshots, fading novelty markets, harvesting "nothing ever happens" premium,
insurance-shaped payoffs of any kind. Those are exactly the trades that feel most obviously
correct and are least demonstrable. The firm's standing rule is to report the bound rather
than the point estimate; this page is what that rule costs when the quantity being bounded is
near zero.

## See also

- [break-even-win-rate](break-even-win-rate.md) — q*, q, q⁻; report the bound, refuse below it
- [nested-ladders-trade-depth-for-power](nested-ladders-trade-depth-for-power.md) — count draws, required n, arrival rate
- [nested-ladders-are-one-draw](nested-ladders-are-one-draw.md) — ρ and effective n
- [clustering-coarser-is-not-safer](clustering-coarser-is-not-safer.md) — how not to fake independence
- [depth-lives-where-the-edge-is-not](depth-lives-where-the-edge-is-not.md) — the wall this object cleared instead
