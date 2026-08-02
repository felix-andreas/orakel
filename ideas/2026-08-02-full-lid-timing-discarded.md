# White House "full lid by 6:30 PM" per-day binaries

- **Date:** 2026-08-02
- **Status:** `discarded-idea`
- **Model:** claude-opus-5 (effort max)
- **Object:** Polymarket series `white-house-call-a-full-lid` — weekly boards of **six
  independent per-day binaries**, "Will the White House call a full lid by 6:30 PM on
  \<date\>?"
- **Classification:** LEVEL claim (we estimate P(lid by 6:30 PM) better than the crowd),
  with a schedule/shape component.
- **Killed by:** **W2 (execution)** — on a *new axis* — with the statistical bound failing
  independently.

---

## The object

A "full lid" is the White House Press Office's announcement that the President has no
further public events that day. Polymarket runs a weekly board with one binary per day
(Mon–Sat), each resolving Yes if a full lid is called by **6:30 PM ET** on that date.

Resolution source, verbatim from the board description:

> This market will resolve according to the time listed by Roll Call of the first full lid
> called in the daily calendar (https://rollcall.com/factbase/trump/calendar/). If Roll Call
> does not list a lid time or is for any reason unavailable, this market will resolve
> according to Forth (https://www.forth.news/whpool).

Example live board (2026-08-02): `will-the-white-house-call-a-full-lid-by-630-pm-august-3-august-8-20260730202018839`,
six legs, Aug 3 – Aug 8, `feeType: politics_fees` (0.04).

**Why it looked good.** It is close to a textbook match for `wiki/market-selection.md`'s
SELECT FOR list: a bureaucratic scheduling variable with strong learnable structure
(day-of-week, presidential travel, holidays), a complete public historical record at the
resolution source itself, fast resolution (≤7 days), genuine uncertainty (mid-band prices),
state **not** glanceable in advance, and no bookmaker anywhere.

## Settled record assembled

23 boards, 2026-02-09 → 2026-08-08 (23 of 26 calendar weeks), **138 legs, 132 settled**.
Full trade tape pulled for all 132 legs; 10-day price history per leg (chunked well under
the 14-day `prices-history` window trap).

Base rate, 132 settled legs: **80/132 = 0.606**.

| | Mon | Tue | Wed | Thu | Fri | Sat |
|---|---|---|---|---|---|---|
| Yes | 14/22 | 14/22 | 13/22 | 15/22 | 8/22 | 16/22 |
| rate | 0.636 | 0.636 | 0.591 | 0.682 | **0.364** | **0.727** |

---

## Wall 3 — power. **PASS**, and it is the first family whose legs are genuinely independent.

Run first and free, per the funnel order.

The obvious worry is the nesting trap (`wiki/reference/nested-ladders-are-one-draw.md`): a
six-leg board might be one observation. **It is not.** These legs are six *different
calendar days*, not six nested thresholds on one underlying, and the data says so:

- One-way ANOVA on the 22 complete boards: **ICC = −0.008**, design effect
  `1 + 5·ICC = 0.96`, **n_eff = 132** against 132 legs.
- Per-board Yes counts are spread right across the range (2,2,2,2,3,3,3,3,3,3,3,4,4,4,4,4,5,5,5,5,5,6) —
  no week-level clustering to speak of.

Arrival, taken off the **settled** record (never the live cohort — 08-01's lesson):
23 boards in 26 weeks × 6 legs ⇒ **≈276 legs/year**. Of the 132 settled legs, 87 (66%)
had an executable ask at the T−24h checkpoint, so the *usable* arrival is ≈182/year.

Required n, from the numbers below (q̂ = 0.598, q\* = 0.531):
`1.96·√(0.598·0.402/n) < 0.067` ⇒ **n > 206 legs**.

So W3 passes on arrival — **~1.1 years to the required sample** — but the sample we have
today (87 usable legs) is under half of it. This is the funnel's first *marginal* W3: not
the 5.9-year fail of object 14, not the comfortable clear of object 16.

## Wall 2 — incumbent (W1). **PASS — "none found", in the sharpest form yet.**

Kalshi's catalogue, one call, **12,369 series**. Three full-lid series exist:

| ticker | title | settlement_sources | markets **ever** |
|---|---|---|---|
| `KXFULLLIDBEFORE630PM` | Will the White House Press Office call a full lid before 6:30PM | `forth.news/whpool`, `rollcall.com/factbase/trump/calendar/` | **0** |
| `KXFULLLIDBEFORE8PM` | …before 8PM | `forth.news/whpool`, `rollcall.com/factbase/trump/calendar/` | **0** |
| `KXFULLLID8PM` | …before 8pm | `kalshi.com` | **0** |

`KXFULLLIDBEFORE630PM` is not an approximate twin. It is the **identical contract** — same
6:30 PM threshold, and it declares *both* of our resolution URLs, in our order. And it has
never had a single market.

This is the strongest possible reading of "no incumbent": a peer venue wrote the rules for
this exact object, down to the settlement page, and declined to list it. It is not that
Kalshi has not noticed the White House press operation, either — adjacent series are live
with real size: `KXWHPRESSBRIEFING` **50,609** contracts / 11,584 OI, `KXPRESSBRIEFINGCOUNT`
**21,368** / 15,746.

No bookmaker prices lid times. No specialist publishes a lid-time forecast — Roll Call
Factbase publishes the historical *record*, which is the data, not a competing forecast.
Fifth object in the funnel to reach "none found".

## Wall 2 — execution. **FAIL. This is the kill, and it is a new coordinate.**

Walked before the modelling, per the process.

### (a) 85.5% of the tape trades after the outcome is already determined

Full Data-API tape, all 132 settled legs, timestamps measured against each leg's own 6:30 PM
ET (22:30Z) deadline:

| | taker notional |
|---|---:|
| traded **before** the 6:30 PM deadline | **$156,580** |
| traded **after** the 6:30 PM deadline | **$922,456** |
| post-resolution share | **85.5%** |

Median post-deadline trade price: **0.994**. The family's $1.79M Gamma headline volume is
not a forecasting market — it is a **settlement-window carry tape**, people buying a decided
outcome for the last fraction of a cent while UMA settles (median `closedTime` − deadline =
**6.7h**).

### (b) At the checkpoint where the outcome is still uncertain, there is nothing there

Taker notional per leg, before each checkpoint:

| checkpoint | median | mean | p90 | legs with **zero** |
|---|---:|---:|---:|---:|
| T−48h | $11 | $262 | $689 | **59 / 132** |
| T−24h | $81 | $405 | $1,145 | **38 / 132** |
| T−12h | $157 | $572 | $1,393 | 17 / 132 |
| pre-deadline | $507 | $1,186 | $2,546 | 7 / 132 |

Only **26 of 132** legs reach $500 of tape before T−24h. Total ask-side notional available
across the **entire six-month settled record** at T−24h: **$25,606**.

### (c) The live book, walked at our own band and size (2026-08-02)

| leg | best bid | best ask | spread | **total** ask-side notional | VWAP to buy $2,000 |
|---|---:|---:|---:|---:|---:|
| Aug 3 | 0.15 | 0.94 | 79c | **$67** | unfillable ($67 max) |
| Aug 4 | 0.08 | 0.55 | 47c | **$146** | unfillable ($146 max) |
| Aug 5 | 0.05 | 0.11 | 6c | $64,142 | **0.553** (vs 0.11 best ask) |
| Aug 6 | 0.06 | 0.94 | 88c | **$132** | unfillable |
| Aug 7 | 0.06 | 0.94 | 88c | **$64** | unfillable |
| Aug 8 | 0.06 | 0.95 | 89c | **$127** | unfillable |

Five of six legs cannot absorb $200. The sixth has a real ladder and costs a **55.3c VWAP
for $2,000 against an 11c best ask** — 44c of slippage.

For comparison: object 14 cleared this wall at $264 at the bid, object 16 at **$19.4M** with
zero slippage to $10,000.

### (d) Gamma's own quote fields are stale, and they hide it

Gamma reported the Aug 3 leg at **bid 0.31 / ask 0.38 (7c spread)** while the live CLOB book
was **0.15 / 0.94 (79c)**. It reported three legs at a `0.50` mid — which is
`(0.06 + 0.94)/2`, a phantom. Every Gamma field on the board carried the same
`updatedAt: 2026-08-02T01:22:52Z` across repeated pulls.

A board-level liquidity gate reading Gamma would have scored this family as a 5–7c-spread
market. It is a 79–89c-spread market.

## The bound — fails independently, and it is one weekday

Because the tape carries `side` and `outcomeIndex`, the executable price is directly
observable. Mapping to YES-equivalents (buying NO = selling YES):

| checkpoint | mean YES **ask** crossed | mean YES **bid** hit | spread | buy YES at the ask, hold | sell YES at the bid, hold |
|---|---:|---:|---:|---:|---:|
| T−48h | 0.5020 | 0.4019 | 10.0c | **+10.22c/sh** | −24.08c/sh |
| T−24h | 0.5345 | 0.3934 | 14.1c | **+8.43c/sh** | −23.02c/sh |
| pre-deadline | 0.6500 | 0.3927 | 25.7c | +7.61c/sh | −16.39c/sh |

**This is not object 12's spread artifact.** The mid bias is real (+16.2pp at T−24h,
t = +4.23) *and* it survives crossing a 14c spread: one direction genuinely wins at
executable prices, the mirror loses much more than symmetrically. That is the mirror test
passing.

And it still fails the bound, which is the number that decides:

> On the **87 legs** with an executable ask at T−24h: **52 wins / 87 = 0.598** against
> break-even **q\* = 0.5309** (mean executable ask 0.5209 + politics fee 0.0100).
> **Wilson 95% lower bound = 0.4926 < 0.5309. FAILS.**
> Point estimate +6.90c/share, **t = +1.44**.

Decompositions, all of which make it worse:

- **By weekday, the edge is Saturday.** Sat +19.26c/sh (t = +2.31, 16/20); Tue +18.58c
  (t = +1.21); Mon +7.85c; Thu +3.64c; Wed +2.84c; **Fri −10.00c** (6/17). Even the
  Saturday subset fails its own bound: Wilson lower 0.5840 vs q\* 0.6094. Twenty Saturdays
  is not a strategy.
- **By regime, the edge is the second half and the size is the first.** Feb–Apr +1.91c/sh
  (t = +0.29) on **$19,712** of reachable ask; May–Aug +11.78c/sh (t = +1.70) on
  **$5,894**. The apparent edge grew as the reachable size fell 70%.
- Leave-one-board-out worst case: +5.04c/share. Not one board — but one weekday.

Capacity ceiling, stated plainly: capturing **100%** of every ask-crossing fill in the whole
six-month record at T−24h would have earned **$3,928** on **$25,606** of notional — and we
would have had to be the counterparty to every taker who actually made those fills.

## Wall 4 — carry, and the escape route

Not binding on the forecasting object (≤7-day horizon).

The obvious escape route is that the money is visibly *somewhere* on this family — 85.5% of
it, in the settlement window at 0.994. Priced properly it dies the same way object 16 did:

- Post-deadline favourite-side fills: 2,835 fills, $1,268,162 notional, +0.133% net on
  notional over a median **6.75h** lock ⇒ +172% annualised. Tempting at fill level.
- **Clustered to legs, which is the honest n:** the favourite won **120 of 121 legs =
  0.9917**, mean entry **0.9883**, so break-even is 0.9883 and the **Wilson 95% lower bound
  is 0.9547 — fails by −3.37pp.** One losing leg in 121 is the whole margin.
- It is also a speed race inside a 6.75h window opening at 00:30 Berlin, against our daily
  cadence.

Same shape as `wiki/reference/rare-event-edges-need-rare-event-samples.md`, reached from a
third direction.

## Falsification sketch (what would reopen this)

Pre-registered, if anyone wants to revisit:

1. Rebuild lid times for ≥3 settled boards directly from `rollcall.com/factbase/trump/calendar/`
   and confirm they reproduce Polymarket's settlement exactly.
2. Fit a hazard model of lid time on day-of-week × presidential travel (public daily
   guidance) × holiday, on the Factbase archive rather than on 132 board legs.
3. Re-walk the CLOB book at T−24h **for 20 consecutive live legs**, recording total ask-side
   notional within 10c of the model price. The family reopens only if the median clears
   ~$500/leg — it is $76 today.
4. Then, and only then, require Wilson-lower > q\* on ≥206 legs.

Realistically (3) is the binding one and it is a property of who trades this board, not of
our modelling.

## Verdict

**Discarded.** Closest miss after object 16, and closest for a different reason: it is the
first object to clear W1 outright *and* survive the mirror test at executable prices, and it
dies on **size** and on **the bound**, not on being already priced.

The one-line summary: the market exists, nobody else prices it, the crowd really is ~16pp
too low at the mid and ~8pp too low at the ask — and there is **$76** there.
