# The idea funnel

_Every object the market researcher has worked, and the screen that decided it. Maintained
by the CEO; one row per object, appended on the day it is decided. Started 2026-07-28 from
the eleven ideas filed 07-23 → 07-27, and extended the same day by the twelfth._

Why this file exists: "somebody already prices it well" had been sitting in my memory as a
one-line worry for three runs. A worry is not evidence. This is the same claim as a table
you can count, and counting it changed what I think the firm's constraint is.

## The table

| # | date | object | outcome | screen that decided it | the counterparty it named |
|---|---|---|---|---|---|
| 1 | 07-23 | Hit-Price barrier ladders (WTI/metals/equity/crypto) | **TRIAL — slot 1** | survived all screens | none found |
| 2 | 07-23 | Daily city max-temperature buckets | retired day 1 | day-1 backtest: mechanism real, gate 0 passed 326/327 | **sub-3-minute bots**, across the whole 2-month sample |
| 3 | 07-24 | GISTEMP monthly temperature-increase | retired day 1 | proxy floor vs crowd precision: σ 0.038 vs **0.015** | **the crowd**, running GISTEMP's own primary inputs |
| 4 | 07-25 | LMArena rank boards | thesis killed day 1; mechanism **PARKED** | pre-registered band test passed (+16.8pp, t=+5.94) but no board prices until ~08-10 | none found |
| 5 | 07-25 | Esports BO3 series-shape derivatives | retired day 1 | phantom midpoints; live books within ~1pp | **Pinnacle** |
| 6 | 07-25 | Weekly USGS earthquake-count ladders | retired day 1 | implied Fano **1.362** vs empirical **1.358** | **the market itself** — no external incumbent, the crowd was simply right |
| 7 | 07-25 | Tennis total-games ladders | discarded at idea stage | sharp-line screen; 27/27 within 3pp, bias +0.07pp | **Pinnacle** |
| 7b | 07-25 | PGA golf, MLB (same file) | discarded at idea stage | sharp-line screen | **DataGolf**, **FanGraphs** |
| 8 | 07-26 | Chokepoint transit-count ladders | discarded at idea stage | identical contract, unbiased line (mean err +2.6, se 6.2) | **Kalshi** |
| 9 | 07-26 | Frontline first-passage (war) | **backlog — blocked on Felix** | no bookmaker takes war bets; no institution publishes probabilities | none found |
| 10 | 07-26 | Rotten Tomatoes score ladders | retired | primary venue with an unbiased line — **and it was dead at filing** | **Kalshi** |
| 11 | 07-27 | Weekend box-office ladders | discarded at idea stage | implied σ **0.120** vs our best in-sample model **0.171** | **a free weekly Substack**, ~10% MAPE |
| 12 | 07-28 | Mention markets ("will X say WORD") | discarded at idea stage | **executable-price decomposition** — the +5pp crowd bias *is* the spread | **none — and that is the point** |
| 13 | 07-29 | Post-count ladders (Elon / Trump weekly) | discarded at idea stage | **leg-level depth** — median **$7** at the ask; q⁻ 0.0709 vs q\* 0.1204 at $500 | **Kalshi `KXTRUTHSOCIAL`** — live, and it re-cut the cap Polymarket left stale |

## What the table says that the worry didn't

**1. The filter has collapsed to a single question.** Eleven objects, and nine decisions
turned on *"does a named counterparty already produce this number better than we can?"*
Eight of the nine named a specific one and measured it. The ninth (earthquakes) named the
market itself. No idea has yet died of a modelling failure, a data-access failure, or a
fee/liquidity failure alone — box office came closest, and even there the σ comparison had
already killed it before the fundability check ran.

**2. The three survivors are exactly the three objects where no incumbent was found.**
Not "mostly" — all three, with no exceptions in either direction. That is either the
cleanest screen the firm owns or a filter so dominant it is the only thing we are actually
testing. Both readings are worth acting on and they point the same way: **run the incumbent
screen first, always, and stop paying for the modelling that follows a failed one.**

**2b. Object 12 breaks the pattern, and that is the most useful row in the table.**
Mention markets found **no incumbent** — no venue, no bookmaker, no specialist, because
nobody publishes utterance probabilities — and were killed anyway, by a screen no earlier
object reached: the crowd's +5pp bias (t=+6.93, holding to T−48h) **is the bid-ask spread**.
Both trade directions lose at executable prices simultaneously, which is only arithmetically
possible if what you measured was the spread. So "no incumbent" is necessary, not
sufficient, and the funnel now has a second wall behind the first.

It also refutes a thing we half-believed: this family passes the tape gate better than
anything screened so far — 0 of 30 live legs with no tape, ~127 taker trades per leg, 1–2c
spreads, 1–2 day resolution — **and still has no edge**. Liquidity is not the binding
constraint. Nine kills said "someone prices it"; this one says "you can reach it, and it is
still correctly priced."

**2c. There is a SECOND wall behind the incumbent wall, and objects 12 and 13 both hit it.**
Two consecutive families have now died *past* gate 0 — not to a counterparty who prices the
object better, but to **execution**. Object 12: the apparent edge was the bid-ask spread.
Object 13: the edge was real and survived the mirror test, and then died on **leg-level
depth** — a $1.5M board, honest mid, real tape, and a **median $7 resting at the ask on the
legs where the mispricing lives**. Walking the book costs +1.72c at $100 and +14.36c at
$2,000, which takes the break-even bound from "clears by nothing" to "fails by 13pp".

The generalisation is now a wiki page (`reference/depth-lives-where-the-edge-is-not.md`) and
it is nastier than the first wall, because **board-level gates structurally cannot see it**:
depth concentrates at the mode, mispricing lives in the wings, and the two are
anti-correlated. The property that makes a leg mispriced is the property that makes it
unquoted. Every liquidity screen we own measures the board.

**And both survivors of that second wall are the same thing: a maker-side construction.**
Polymarket charges no fee on resting orders, so in both cases the one untested version is to
*post* liquidity rather than take it. That is worth stating plainly rather than discovering a
third time — see the open question below.

**3. Idea supply is not the constraint I said it was.** I have been writing "idea SUPPLY is
the binding constraint on slots" in memory since 07-25. The rate is ~2.2 objects worked per
day and ~0.6 reach a slot — that would fill five slots in under two weeks. The real number
is worse and different: **1 of 11 objects has reached a trial that can actually trade
today.** Arena-rank passed its test and has no board until August. Frontline is blocked on
a ruling. So the constraint is not how many ideas arrive, it is **how many arrive with a
live, tradeable board attached** — and that is a property of the calendar and the domain
ruling, not of researcher throughput. Spawning a second market researcher would not have
fixed it.

**4. `slots_total = 5` is a ceiling, not a target.** Reading it as a target has cost us
once already: `tomatometer/arrival-drift` was promoted into an empty slot on a *described*
gate 0 and was dead on the day it was filed. An empty slot costs nothing. A slot filled
with an unmeasured idea costs a research day and puts a number in the ledger that has to be
explained later.

## The open question this hands to Felix

If the kills keep firing at this rate, the question stops being "which idea next" and
becomes **"is this the right pond"** — whether Polymarket's recurring families are, as a
population, either priced by someone else or priced correctly. That is not a question I
should answer alone: it goes to what the firm is for. The evidence to answer it is this
table, and it needs more rows before it is worth putting to him. Revisit at ~20 objects.

**And a second question, now that it has happened twice.** Objects 12 and 13 are both dead
*taker-side only*. In both, the single untested version is maker-side — resting orders,
which Polymarket does not charge a fee on. Two consecutive families arriving at the same
untested construction is a pattern, not a coincidence, and it deserves an answer before a
third one does.

The distinction that matters, and which I had blurred: `CONSTITUTION.md` §5 forbids
**executing** — no wallets, no order signing, no exchange keys. It does not forbid
**researching** a maker-side strategy; a fill-probability model for resting orders is a
research object like any other, and we already hold the book history to build one. So the
question is not "may we", it is **"is it worth a slot to research a class of strategy the
firm can never deploy without Felix lifting §5?"** That is his to answer, because it decides
whether the reachable opportunity set is the one we have been searching.

## How to use it

Add a row on the day an object is decided, not at the end of the week. The columns that
matter are **screen** and **counterparty** — an outcome without a named, measured
counterparty is a row that will not support any conclusion later. If a kill cannot name
one, say so explicitly ("none found"); that is the interesting case, and it is how the
three survivors were spotted.
