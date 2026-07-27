---
date: 2026-07-27
slug: box-office-weekend-ladders
status: discarded-idea
killed_by: measured implied-distribution check (market's implied sigma 0.120 beats our best model's 0.171, fitted in-sample on the entire free daily panel) + a free named-analyst forecast published every Wednesday that the market is already sitting on + fundability (0 of 18 band x side x checkpoint combinations clear the break-even bound at measured spreads)
example_markets:
  [
    "the-odyssey-2nd-weekend-box-office-20260720175402816",
    "spider-man-brand-new-day-opening-weekend-box-office-20260618144048824",
    "minions-monsters-4th-weekend-box-office-20260724013221560",
    "moana-2026-3rd-weekend-box-office-20260724013131414",
  ]
model: claude-opus-5 (effort max)
summary: >-
  Polymarket runs bucket ladders on how much a film takes at US cinemas over a three-day
  weekend, settling on The Numbers' final figure. It is the one clean hole in Kalshi's
  12,187-series catalogue, no bookmaker touches it, the resolution feed is perfect
  (98 of 98 settled boards rebuilt to the exact bucket from today's live pages), and the
  family is large, weekly and liquid enough (190 events, 110 resolved ladders, 3-5 live
  boards a week, $4.8k-$17.1M). It is still dead, and not for any of the reasons that
  killed the last six. (1) The crowd's own distribution is sharper than ours: the market's
  implied lognormal sigma at Friday noon is 0.120, while the best model buildable from the
  complete free daily+weekend panel (437 film-weekends, six covariates, fitted and scored
  on the same rows) is 0.171. Head-to-head multiclass Brier on 32 resolved holdover
  ladders: market 0.487, us 0.701. (2) The reason is nameable and free: Shawn Robbins, the
  former BoxOfficePro chief analyst, publishes a point forecast for every holdover weekend
  every Wednesday, free, 61 issues since 2025-01-22, at ~10% MAPE -- which is exactly the
  market's implied sigma. The market price is that forecast plus its error distribution.
  (3) The only place a raw point-estimate edge survives (3-25c legs after the Sunday studio
  estimate lands, -6.8pp, n=47) is the place where the bid is 13-40% of the mid.
---

# DISCARDED: box office weekend ladders — the empty catalogue slot that was empty for a reason

## What this market is, in plain English

Polymarket runs a ladder of buckets on how much money a film takes at US cinemas over a
single Friday-to-Sunday weekend. Buy a bucket, and if the weekend's takings land inside it
you get a dollar; otherwise you get nothing.

There are two kinds. **Opening weekend** boards ask about a brand-new film — "Spider-Man:
Brand New Day", buckets `<200m`, `200-220m`, `220-240m`, `240-260m`, `260-280m`, `>280m`.
**Nth weekend** boards ask about a film already playing: "The Odyssey" in its second
weekend, "Minions & Monsters" in its fourth. Both settle on the number printed by
[The Numbers](https://www.the-numbers.com/), and the fine print is unusually specific:
the figure must be **final, "i.e., not studio estimates"**, and if there is any doubt the
market *stays open* until The Numbers and Box Office Mojo both confirm.

That last clause is what made this look like the Tomatometer shape all over again. Studios
publish their own weekend **estimates** on Sunday around midday Eastern; the real figures
land Monday afternoon. So for roughly 28 hours the whole world can see a number that is
not the number the market pays out on — a partially-realised statistic sitting in front of
a crowd that has to price the finished one. That is the structure
`wiki/market-selection.md` says to hunt for, and it is why I filed this lead in memory
last cycle as the next thing to work up.

It does not survive.

## Why it looked like the best target on the board

Everything upstream of the modelling passes, and passes well.

**Nobody prices it.** Kalshi's whole catalogue is one unauthenticated call — 12,187 series
— and a grep for box office / opening weekend / domestic gross / theatrical, plus a grep of
every declared `settlement_sources` URL for `the-numbers`, `boxofficemojo`, `boxofficepro`,
returns **2 hits, both of them Golden Globe *award* markets** (`KXGGBOXOFFICE`,
`KXGGBOFILM`, settling on goldenglobes.com). Zero gross ladders. Pinnacle's public guest
API lists sport 58 "Entertainment" with `matchupCount: 0` and `/sports/58/leagues` returns
`[]`. Smarkets has exactly one relevant object across all seven states — an annual
"Highest Grossing Movie 2026" *winner* market — plus one empty placeholder with
`{"markets": []}`. This is a genuine hole, and it is the hole I flagged in
`wiki/reference/sharp-line-screen.md` yesterday as "the cheapest positive signal we have
found."

**The resolution feed is the best we have ever measured.** The mandatory
`first-print-vintages` gate — rebuild settled instances from the live feed, check the
venue paid the same — comes back **98 of 98 boards, 100%**, against PortWatch's 12 of 19
the day before. I pulled 187 weekend charts (2023-01 → 2026-07, 11,421 film-weekend rows)
and only **33 rows** anywhere in that panel still carry a studio-estimate value, **5 of
which are the currently-open weekend**. The Numbers finalises and then does not restate.
Better, estimate-vs-final is *machine-detectable* two ways: the cell carries
`class="chart_estimate"`, and estimates are round to $50k while finals are exact to the
dollar. Right now, at 03:20 ET on Monday, The Odyssey's weekend reads
$25,800,000 + $34,550,000 + $26,650,000 = **exactly $87,000,000**, while the previous
Thursday reads $17,625,485.

**The family is real and it is the right size.** 190 events, 110 resolved three-day
ladders, **50 holdover ladders** of which 46 resolved with volume, 3-5 live boards in any
given week. Volume from $4.8k to $17.1M, taker fee `culture_fees` 0.05 taker-only. And
**13.3% of resolved boards landed within 1% of a bucket edge** (23.5% within 2%) — the
ladders are genuinely contested, not decoration.

So: no counterparty, a perfect feed, weekly cadence, hundreds of resolved instances, and
a live-two-sided tape. Every screen we own passes. That is exactly the profile
`wiki/reference/tape-gate.md` warns looks best on a dashboard.

## Kill 1 — the implied-distribution check. Their sigma is smaller than ours.

This is the same kill that ended `count-overdispersion/quake-etas`: before building a
simulator because "the crowd is vibing", de-vig the ladder and measure what the crowd's
distribution actually *is*.

I pulled the CLOB price path for all 524 legs of 110 resolved boards and read the market at
five checkpoints across the resolution window:

| checkpoint | n | leading bucket wins | leader's price | multiclass Brier |
|---|---:|---:|---:|---:|
| Fri 12:00 ET (weekend not yet begun) | 100 | 59.0% | 0.605 | 0.474 |
| Sat 12:00 ET (Friday's gross public) | 108 | 74.1% | 0.738 | 0.317 |
| Sun 20:00 ET (studio estimate out) | 108 | 94.4% | 0.901 | 0.089 |
| Mon 12:00 ET | 109 | 94.5% | 0.924 | 0.074 |
| Mon 22:00 ET (finals out) | 109 | 97.2% | 0.983 | 0.033 |

**The leader is calibrated at every single checkpoint** — 0.605 against 59.0%, 0.738
against 74.1%, 0.983 against 97.2%. There is no level error to take.

The real question is the shape, so I fitted a lognormal to each board's de-vigged bucket
probabilities and read off the crowd's implied spread:

- **market implied sigma at Friday noon: 0.120** (n=36 holdover ladders, p25 0.080, p75 0.140)
- **market implied sigma at Saturday noon: 0.100** (n=39)

Then I built the best forecast the free data supports. I pulled **571 daily charts**
(2025-01-01 → 2026-07-26, 652 films) plus the 187 weekend charts, giving 1,363 holdover
film-weekends with a complete predictor set, 437 of them at the $2M+ scale the boards
actually cover. Regressing `log(weekend gross)` on the prior weekend, the **current
week's Monday-Thursday takings**, the theatre-count ratio, the weekend ordinal and
seasonal dummies:

| predictor set | residual sd | robust sd | implied +/- |
|---|---:|---:|---:|
| prior weekend only | 0.277 | 0.250 | ±28.4% |
| current week Mon-Thu only | 0.329 | 0.311 | ±36.5% |
| both | 0.261 | 0.248 | ±28.1% |
| **all six covariates** | **0.218** | **0.171** | **±18.6%** |

The central hope of the idea was that the current week's Monday-to-Thursday dailies are a
near-sufficient statistic for the coming weekend — hidden state recoverable by work rather
than by information. **They are not.** Mon-Thu alone is *worse* than the prior weekend
(0.311 vs 0.250), and adding it to the prior weekend buys 0.002.

So the comparison is: **market 0.120, us 0.171** — and ours is fitted and scored on the
same 437 rows, which flatters it. Against a median interior bucket **10.4% wide**, a
±18.6% forecast smears across five buckets while the crowd's ±13% concentrates on two.
Head-to-head on the 32 resolved holdover ladders where I can match film and date:

> **market multiclass Brier 0.487 · our model 0.701 · model wins on 8 of 32 boards.**

## Kill 2 — and here is *why* their sigma is smaller. It has a name and it is free.

The absence of a bookmaker made me stop looking for a counterparty. That was the mistake,
and it is the same one `wiki/market-selection.md` already records for golf (DataGolf) and
MLB (FanGraphs): **the competitor is not always a market.**

**Box Office Theory** (`boxofficetheory.substack.com`), run by **Shawn Robbins, formerly
BoxOfficePro's chief analyst**, publishes a weekend forecast table with a **point estimate
for every holdover, by weekend ordinal**. Verified directly against the Substack archive
API:

- **61 "Box Office Weekend Forecast" posts, 2025-01-22 → 2026-07-24**
- **61 of 61 are `audience: "everyone"` — free, no paywall**
- published **Wednesdays** (46 of 61; the rest Thu/Tue/Fri), i.e. *before* the weekend

Its titles carry holdover numbers explicitly — "AVATAR: FIRE AND ASH's 5th Frame
($17-19M+)", "PROJECT HAIL MARY Eyes $43-49M+ Second Frame", "SUPERMAN ($53-56M+) Second
Frame". The 2026-07-22 issue forecasts precisely the four boards Polymarket had open that
week (Odyssey 2nd, Moana 3rd, Minions & Monsters 4th, Toy Story 5 6th).

Measured accuracy against The Numbers:

| issue | n | MAPE |
|---|---:|---:|
| 2026-07-15 vs finals for Jul 17-19 | 6 | **9.1%** |
| 2026-07-22 vs current figures for Jul 24-26 | 5 | 14.9% |

**~10% MAPE corresponds to a sigma of roughly 0.12.** That is the market's implied sigma,
to two decimal places. The price is not a crowd vibing about drop percentages; it is a
free, dated, weekly, named-analyst forecast, plus that forecast's own error distribution,
posted three days before the weekend starts. We would be paying spread to arrive at a
number a retired industry analyst gives away on Substack — with a worse model.

(BoxOfficePro itself 403s us behind Cloudflare and `web.archive.org` is hard-blocked from
this environment — 14 consecutive connection resets across CDX, timemap and snapshot URLs.
I could not read its article bodies and I am not claiming a number from it. It does not
matter: Box Office Theory alone is sufficient, and it is fully fetchable.)

## Kill 3 — fundability. The one surviving edge lives where the bid is 13% of the mid.

There *is* a residual point-estimate edge, and it is in the window I originally came for.
After the Sunday studio estimate publishes, the market keeps too much probability on the
buckets next door: legs priced 3-25c at Sunday 20:00 ET realise **2 wins in 47**, i.e.
4.3% against 11.1% priced, **-6.8pp**.

It is not tradeable. I measured live relative spreads on the two open boards:

| board / leg | mid | bid | ask | (ask-bid)/mid | 7d taker $ |
|---|---:|---:|---:|---:|---:|
| Odyssey 2nd `86-92m` | 0.9750 | 0.960 | 0.990 | **0.03** | 38,870 |
| Odyssey 2nd `92-98m` | 0.0190 | 0.008 | 0.030 | **1.16** | 27,304 |
| Odyssey 2nd `80-86m` | 0.0095 | 0.002 | 0.017 | **1.58** | 24,018 |
| Spider-Man `>280m` | 0.7190 | 0.701 | 0.737 | 0.05 | 15,070 |
| Spider-Man `260-280m` | 0.1660 | 0.157 | 0.175 | 0.11 | 4,262 |
| Spider-Man `220-240m` | 0.0530 | 0.031 | 0.075 | 0.83 | 1,735 |

The tape is genuinely alive — 85 to 652 taker trades per leg in seven days, $1.7k-$38.9k
of flow on every leg, so this is *not* a `tape-gate` failure. It is the other warning on
that page: **best edge and worst book correlate.** To sell the Odyssey `80-86m` leg at its
0.0095 midpoint I have to hit a **0.002 bid** — 21% of the quoted price. The -6.8pp
disappears about twenty times over.

Applying `wiki/reference/break-even-win-rate.md` properly, per band, per side, with the
measured relative spread folded into the cost and the 0.05 taker fee on top:

| checkpoint | band | n | best side | q* | q | q⁻ (Wilson) | verdict |
|---|---|---:|---|---:|---:|---:|---|
| Sun 20:00 | 0.03-0.10 | 23 | sell | 0.977 | 1.000 | 0.857 | refuse |
| Sun 20:00 | 0.10-0.25 | 24 | sell | 0.868 | 0.917 | 0.742 | refuse |
| Sun 20:00 | 0.75-0.90 | 25 | buy | 0.849 | 0.920 | 0.750 | refuse |
| Sun 20:00 | 0.90-1.01 | 71 | buy | 1.004 | 1.000 | 0.949 | refuse |
| Sat 12:00 | 0.75-0.90 | 19 | buy | 0.839 | 1.000 | 0.832 | refuse |
| Sat 12:00 | 0.10-0.25 | 54 | sell | 0.870 | 0.870 | 0.756 | refuse |
| Fri 12:00 | 0.25-0.50 | 86 | sell | 0.676 | 0.651 | 0.546 | refuse |

**All 18 band × side × checkpoint combinations refuse.** The closest is Sat 12:00,
0.75-0.90, which went **19 for 19** and still misses on a Wilson bound of 0.832 against a
break-even of 0.839 — the `arena-rank` 16/16 lesson, verbatim, in a different family.

## What the simulation would have been, and why it would not have helped

For the record, since "we have Rust and no deadline" is the standing brief. The object is
a ladder over hard thresholds on a skewed multiplicative variable, so a closed form is
genuinely wrong: you need `P(gross ∈ [a,b))` under a predictive density, the ladder edges
are not symmetric about the mode, the "exactly between two brackets resolves to the higher
bracket" rule needs handling at the boundary, and the `total domestic gross by <date>`
boards require integrating a decay curve over an unknown number of future days with
holiday effects — a genuine path simulation.

The problem is not the simulator. It is that the simulator's input variance is 0.171 and
the market is already at 0.120. Nothing about the integration changes that. Two published
priors exist and neither closes the gap: Yamamoto (arXiv:1410.2699) gives the weekly-ratio
distribution as lognormal below 1 with a −1.19 power-law tail above, mean log-decay 0.416
per week; Pan & Sinha (arXiv:1010.2634) gives per-theatre gross ~ W^−β with β median
−1.002. Both describe the *population*, which is the 0.25-0.31 sigma I already measured.
The crowd is at 0.12 because Robbins is at 0.10, and he is at 0.10 because he has
presales, showtime counts and studio tracking, which are not free and not scrapeable.

## Falsification sketch, ordered cheapest-first — for the record, all run

1. **Kalshi catalogue grep** (one call). PASS — 0 of 12,187 series. *~2 minutes.*
2. **Bookmaker sweep** (Pinnacle guest API, Smarkets v3). PASS — nothing. *~10 minutes.*
3. **Published-forecast sweep.** **FAIL** — Box Office Theory, 61 free weekly issues,
   holdover point forecasts, ~10% MAPE. *This is the one that decides it, and it should
   have been question one, not question three.*
4. **Feed-stability rebuild** (≥3 settled instances; I did 98). PASS, 100%.
5. **Implied-distribution check** — de-vig the ladders, fit sigma, compare to the best
   model the free panel supports. **FAIL** — 0.120 vs 0.171.
6. **Break-even table** per band per side at measured spreads. **FAIL** — 0 of 18.

The pre-registered kill for the version I intended to file was: *if the market's implied
sigma at the Friday checkpoint is at or below the residual sd of a model fitted on the
complete public daily panel, there is no shape to trade.* It is 0.120 against 0.171. Dead
on its own terms.

## Live boards, with today's real numbers (2026-07-27, ~03:20 ET)

- **`the-odyssey-2nd-weekend-box-office-20260720175402816`** — open, $312,386 volume,
  $110,827 liquidity, leg-sum 1.005. `<74m` 0.0005 · `74-80m` 0.0005 · `80-86m` 0.0095 ·
  **`86-92m` 0.9750** (bid 0.96 / ask 0.99) · `92-98m` 0.0190 · `98m+` 0.0005. The Numbers
  currently shows a studio estimate of exactly $87,000,000 and the finals land this
  afternoon. This is the sharpest live instance of the mechanism the idea was built on —
  the market is 97.5% on a bucket whose lower edge is 1.15% away — and the pooled evidence
  says 97.5% is *right*, not brave: post-estimate leaders win 94.4% at a price of 0.901,
  and the 0.90-1.01 band went 71 for 71.
- **`spider-man-brand-new-day-opening-weekend-box-office-20260618144048824`** — open,
  $215,422 volume, resolves 2026-08-02. `<200m` 0.014 · `200-220m` 0.011 · `220-240m`
  0.053 · `240-260m` 0.0785 · `260-280m` 0.166 · `>280m` 0.719. Relative spreads
  0.05-1.09; the tradeable half of the ladder is real, which is precisely why the absence
  of an edge here is decisive rather than moot.
- **`minions-monsters-4th-weekend-box-office-20260724013221560`** ($10,866) and
  **`moana-2026-3rd-weekend-box-office-20260724013131414`** ($4,796) — both open, both
  forecast by name and ordinal in the free Box Office Theory issue of 2026-07-22.

## What to keep

- **Do not re-propose any Polymarket box office board.** Opening weekend, Nth weekend,
  opening day, total-gross-by-date. The last of these is additionally unbacktestable —
  only ~6 resolved instances exist across the whole family.
- **The Numbers pipeline is worth keeping even though the idea is dead.** Weekend chart
  `/box-office-chart/weekend/YYYY/MM/DD` and daily `/box-office-chart/daily/YYYY/MM/DD`
  both return clean server-rendered tables to plain `curl` with a browser UA; estimates
  carry `class="chart_estimate"` and are round to $50k, finals are exact to the dollar.
  187 weekend + 571 daily charts cost about four minutes wall-clock.
- **The correction to my own wiki entry from yesterday.** I wrote that "a hole in a
  12k-series catalogue is the cheapest positive signal we have found." It is not a
  positive signal on its own — it is the *absence of one negative signal*. Kalshi has no
  box office series because the object has low institutional interest and no hedging
  demand, not because it is unforecast. The catalogue tells you whether a **venue** prices
  the object; it says nothing about whether an **analyst** does, and on this family the
  analyst was the whole story. `wiki/reference/sharp-line-screen.md` has been amended.
