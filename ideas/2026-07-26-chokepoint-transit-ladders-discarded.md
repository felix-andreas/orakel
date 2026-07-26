---
date: 2026-07-26
slug: chokepoint-transit-ladders
status: discarded-idea
killed_by: measured sharp-incumbent screen (Kalshi runs the identical contract, unbiased) + resolution-source vintage instability (today's feed cannot reproduce 7 of 19 settled boards)
example_markets:
  [
    "how-many-ships-transit-the-strait-of-hormuz-week-of-july-20-20260717154058215",
    "how-many-ships-transit-the-strait-of-hormuz-week-of-july-27-20260724144200302",
    "strait-of-hormuz-traffic-returns-to-normal-by-december-31",
    "how-many-ships-transit-bab-el-mandeb-strait-week-of-july-20-20260713200946341",
  ]
model: claude-opus-5 (effort max)
summary: >-
  Polymarket runs ~$36M of boards on one object: the number of ships IMF PortWatch
  reports transiting a shipping chokepoint. It looks like the perfect target — a pure
  counting process, a free machine-readable government feed, weekly cadence, 19 resolved
  instances, zero taker fee, live two-sided books with 174 distinct wallets. It is dead
  three times over, all measured. (1) Kalshi lists the SAME contract off the SAME
  PortWatch page with 156k-446k contracts a week and 1c spreads, and its line is
  unbiased for the realised settlement (mean error +2.6, se 6.2, n=9). (2) Polymarket
  is not systematically wrong against that line either (+4.6pp on the winner, se 3.8,
  t=1.2). (3) The feed itself is not a fixed number: PortWatch restated settled weeks by
  -9% to +247%, today's API fails to reproduce the winning bucket on 7 of 19 resolved
  boards, and the two venues resolved the SAME week to contradictory values two days
  apart. No backtest of this family can be built from the live feed.
---

# DISCARDED: shipping-chokepoint transit ladders — the counting process that is really a data-pipeline forecast

This was my lead candidate for the 2026-07-26 cycle and it is filed as negative knowledge
because it is expensive negative knowledge. The family passes **every** positive screen in
`wiki/market-selection.md` and **every** liquidity screen we own — phantom-midpoint, tape,
wallet concentration, leg-sum. A CEO reading a description of it would fill a slot. It is
still dead, and the two things that kill it are both cheap and both were run before any
modelling, per the playbook rule written this morning.

---

## 1. What the market is, in plain English

Every commercial ship on earth broadcasts its position. The IMF collects those broadcasts
and publishes a daily count of how many ships passed through each of 28 shipping
chokepoints — the Strait of Hormuz, the Suez Canal, the Panama Canal, and so on. It is a
free public government dataset.

Polymarket turns that count into bets. The main one is a weekly ladder: *how many ships
will pass through the Strait of Hormuz between Monday and Sunday?* You pick a bucket —
under 50, 50 to 74, 75 to 99, and so on. Around it sits a whole family of boards on the
same daily number: will 60 ships pass on any single day; will the seven-day average get
back to 60 ("traffic returns to normal"); in which month will that happen.

Hormuz is currently disrupted, so the counts have collapsed from a normal ~600/week to
roughly 15-230/week and swing wildly. That volatility is what makes the boards busy.

**Why it looked ideal.** The answer is arithmetic — add up seven numbers. The crowd is
plainly trading headlines about Iran. No bookmaker takes bets on shipping traffic.
Polymarket charges **zero taker fee** on geopolitics. And the supply is real: 19 resolved
weekly boards since March at $37k-$932k each, plus ~$36M sitting on the open sibling
boards ($20.9M / $6.0M / $2.7M on the three "returns to normal" legs alone).

## 2. The board is genuinely alive — this is not a phantom-book story

Pulled live at 2026-07-26T09:19Z on
`how-many-ships-transit-the-strait-of-hormuz-week-of-july-20-20260717154058215`
($111,477 volume, $53,799 liquidity, resolves today):

| bucket | bid | ask | book depth (bid/ask) | 7d taker trades | distinct wallets | 7d Yes-equiv flow (buy/sell) |
|---|---|---|---|---|---|---|
| **<50** | 0.45 | 0.47 | $2,690 / $8,359 | 500 | **174** | $11,984 / $16,045 |
| **50-74** | 0.48 | 0.49 | $255 / $1,413 | 263 | 125 | $1,458 / $918 |
| 75-99 | 0.04 | 0.05 | $27 / $64,718 | 217 | 78 | $295 / $947 |
| 100-124 | 0.015 | 0.022 | $3 / $162,143 | 148 | 46 | $52 / $171 |
| 125+ | — | 0.001 | $0 / $162,796 | 151 | 46 | $30 / $118 |

Two legs sit squarely in the fundable band with 1-2c spreads, ~$28k of realised seven-day
taker flow on **both** sides of the leg we would trade, and 174 distinct wallets. Leg-sum
1.019. This board passes `phantom-midpoints`, `tape-gate` and `wash-trading` outright.
**That is the lesson: our liquidity screens are working, and they are not the binding
constraint. Who else is in the market is.**

## 3. Gate 0, measured — Kalshi runs the identical contract and is unbiased

### 3.1 The screen is now one unauthenticated call

`https://api.elections.kalshi.com/trade-api/v2/series?limit=1000` returns **12,186 series**
with `title`, `category` and `settlement_sources`, no key, ~16MB, one request. Grepping it
took seconds and it is now the cheapest gate 0 we own (promoted to
`wiki/reference/sharp-line-screen.md`).

It returned an entire Hormuz complex: `KXHORMUZWEEKLY`, `KXHORMUZMAX`, `KXMAXSHIPSHORMUZ`,
`KXHORMUZPEAK`, `KXHORMUZAVG`, `KXHORMUZNORM`, plus `KXSUEZTRAFFIC` and `KXPANAMATRAFFIC`.

`KXHORMUZWEEKLY`'s declared settlement source is
`{"name": "IMF PortWatch", "url": "https://portwatch.imf.org/pages/chokepoint6"}` — the
**same page Polymarket names**. Its rule text is the same arithmetic: *"The total is
calculated by summing daily counts from May 18, 2026 to May 24, 2026."* These are not
similar contracts. They are the same contract.

### 3.2 It is not a dead listing

| Kalshi weekly board | contracts traded |
|---|---:|
| Jul 6-12 | **446,372** |
| Jul 13-19 | **313,178** |
| Jun 22-28 | 217,054 |
| May 18-24 | 257,963 |
| **Jul 20-26 (live)** | **156,097** |

Live Jul 20-26 book: T30 **0.90/0.91**, T40 **0.73/0.74**, T50 **0.42/0.43** — **1c
spreads** with 13-394 contracts at touch. Comparable notional to Polymarket's $111k, on a
tighter book.

### 3.3 And its line is unbiased for the realised settlement

Kalshi publishes `expiration_value` — the exact settled integer. Taking the mid-market
implied median from its ≥X ladder at the **window-close** checkpoint (Sunday 23:00Z) on all
9 finalized boards:

| week | realised | Kalshi implied median | error |
|---|---:|---:|---:|
| 05-11..05-17 | 15 | 36.8 | −21.8 |
| 05-18..05-24 | 42 | 35.0 | +7.0 |
| 05-25..05-31 | 33 | 30.6 | +2.4 |
| 06-08..06-14 | 18 | 24.3 | −6.3 |
| 06-15..06-21 | 92 | 68.4 | +23.6 |
| 06-22..06-28 | 228 | 236.9 | −8.9 |
| 06-29..07-05 | 225 | 206.1 | +18.9 |
| 07-06..07-12 | 115 | 138.6 | −23.6 |
| 07-13..07-19 | 85 | 52.7 | +32.3 |

**mean error +2.63, median +2.38, sd 18.57, se 6.19, t = 0.42, n = 9.** No bias. This is
verbatim the tomatometer kill: a peer venue that turns out to be a co-primary venue, and
whose line has no exploitable tilt.

Note also what the *spread* of that error says. At window close the seven days have
physically happened, and the best-informed venue is still **±18.6 ships** out. A naive
"same as last week" forecast has errors of −110 to +136 over the same weeks, so Kalshi is
doing real work — it just cannot see the answer, because nobody can. That residual is not
shipping. It is item 4.

## 4. The kill that matters more, because it is venue-independent: the feed is not a number

The PortWatch daily series is served from a public ArcGIS feature layer,
`services9.arcgis.com/weJ1QsnbMYJlCHdG/.../Daily_Chokepoints_Data/FeatureServer/0/query`
— 2,757 days per chokepoint back to 2019-01-01, no key. It is exactly the "boring,
machine-readable, nobody's job" source we are told to hunt for.

**It is also not stable.** Comparing Kalshi's settled `expiration_value` (the first print
the venue read) against what the same API returns for the same seven dates *today*:

| week | settled on | today's API | revision |
|---|---:|---:|---:|
| 05-11..05-17 | **15** | 52 | **+37 (+247%)** |
| 05-18..05-24 | 42 | 57 | +15 (+36%) |
| 05-25..05-31 | 33 | 36 | +3 (+9%) |
| 06-08..06-14 | **18** | 44 | **+26 (+144%)** |
| 06-15..06-21 | 92 | 107 | +15 (+16%) |
| 06-22..06-28 | 228 | 214 | −14 (−6%) |
| 06-29..07-05 | 225 | 205 | −20 (−9%) |
| 07-06..07-12 | 115 | 107 | −8 (−7%) |
| 07-13..07-19 | 85 | 85 | 0 |

Reconstructing all 19 resolved Polymarket boards from today's feed and asking which bucket
would have won: **today's PortWatch API fails to reproduce the actual winning bucket on
7 of 19 boards (37%)** — Mar 10-16 (resolved 35-39, feed says 18), Mar 17-23 (15-19 vs 24),
Mar 23-29 (20-24 vs 42), Apr 6-12 (40-49 vs 59), May 4-10 (<25 vs 29), Jun 8-14 (<25 vs 44),
Jun 15-21 (75-99 vs 107).

And the sharpest single fact in this file: for the week of **May 11-17**, Kalshi settled at
**15** on May 19 and Polymarket resolved the **40-59** bucket on May 21. Two venues read the
same page two days apart and got answers that cannot both be true. The feed moved
15 → 40-59 → 52 for one fixed set of seven days.

The mechanism is plausible and unfixable from our side: PortWatch derives counts from
satellite/terrestrial AIS via the UN Global Platform, and in a conflict zone vessels go
dark, transponders are spoofed, and coverage backfills for weeks. Settlement timing is a
weekly Tuesday batch (every Kalshi board closes on a Tuesday; today, Sunday 26 July, the
API's latest date is still **19 July**), so the board's whole trading life happens with
**zero** days of its own window published.

**Consequences, and they are fatal:**

1. **No backtest of this family can be built.** The target variable at settlement is not
   what the feed says now. Fitting on today's snapshot fits a series that never existed.
2. **There is no vintage archive and one cannot be made retroactively.** An ArcGIS query
   endpoint is not in Wayback. The only vintage records that exist anywhere are the 11
   Kalshi `expiration_value`s and 19 Polymarket bucket resolutions I recovered here. To
   build a model of the revision process we would have to snapshot daily for months
   *before* we could trade — which is not a fundable trial.
3. **The forecast target is not "how many ships will sail".** It is "what integer will an
   AIS ingestion pipeline have published by Tuesday". Modelling skill on shipping is beside
   the point, which is why Kalshi's error at window close is ±18.6 with the week already over.

## 5. The last hope — trade Polymarket against the Kalshi line — is also measured, and also dead

If a sharp venue is unbiased and a thin venue deviates from it, the trade needs no model at
all. So I translated Kalshi's ≥X step CDF into Polymarket's bucket geometry at the same
window-close timestamp on all 9 overlapping weeks and asked **who was closer to the truth**.

On the bucket that actually won: Polymarket priced it higher in 6 weeks, lower in 2, tied
in 1. Mean (Polymarket − Kalshi) on the realised winner **+4.6pp, median +1.2pp, se ≈3.8pp,
t ≈ 1.2, n = 9.**

Read it the right way round: **Polymarket is, if anything, the better-priced venue**, and
the difference is inside noise. There is no systematic cross-venue spread to harvest in
either direction, and nothing here could clear a `break-even-win-rate` lower bound at n=9.
The two live boards even agree today: Kalshi implies P(<50) ≈ 0.544 against Polymarket's
0.455 — a 9pp gap on one leg that the 9-week record says is noise, not signal.

## 6. What about the chokepoints Kalshi does not cover?

Screening the 12,186 series: Kalshi has **no Bab el-Mandeb series at all**, `KXSUEZTRAFFIC`
has **0 markets**, and `KXPANAMATRAFFIC` is 14 finalized markets with nothing live. Polymarket
just opened a Bab el-Mandeb weekly family (`...-week-of-july-20`, $12,036; `...-week-of-july-27`,
$0). Statistically it is a far nicer object than Hormuz — last 20 weeks range 212-287, mean
253, sd ~18, coefficient of variation 7%, lag-1 R² 0.075 (near i.i.d. around a stable level).

**It still fails.** It inherits §4 whole — same feed, same AIS backfill, same conflict zone
(Red Sea) — and unlike Hormuz there is **not one resolved instance** against which the
revision size could be measured. Board depth is $12k across 6 legs. That is `needs-gate-0`
on a board too thin to fund even if it passed.

## 7. Falsification, ordered cheapest-first — what would have to be true to revive this

Recorded so nobody re-scans it cold. Any *one* of these failing kills it again.

1. **A vintage archive exists.** Find a point-in-time record of PortWatch daily counts
   (an IMF bulk export with revision dates, a mirrored dataset, a third party who snapshots
   it). Without this there is no backtest, full stop. *Checked: Wayback has no coverage of
   the ArcGIS query endpoint.*
2. **The revision is predictable.** Given the first print, is the eventual value forecastable?
   Needs ≥30 (first-print, final) pairs; we have 9. Kill if the residual sd exceeds ~½ a
   bucket width.
3. **Kalshi's line is biased at an EARLIER checkpoint.** I measured window-close. A tilt at
   window-open (the checkpoint we could actually trade at agent cadence) would revive it.
   *Prior: unlikely — the error at close is already unbiased and dominated by pipeline noise
   that is no smaller earlier.*
4. **Polymarket lags Kalshi in time rather than in level.** Cross-correlate the two venues'
   implied medians at hourly resolution; a stable multi-hour lead would be tradeable. Kill if
   the lead is under our daily cadence (`delayed-execution-test`) — which it almost certainly is.
5. Only then: build the daily-count simulator.

## 8. What generalises (both promoted to the wiki)

- **The Kalshi catalogue is one call, and it must be run on every idea before anything else.**
  12,186 series with declared settlement sources. Today it took seconds to discover that the
  incumbent uses *our exact resolution URL*. Added to `wiki/reference/sharp-line-screen.md`.
- **"Machine-readable and public" is not the same as "fixed".** We already knew index prints
  get revised (`first-print-vintages`, GISTEMP sd 0.019 °C flipping 9 of 28 buckets). This is
  the same failure an order of magnitude larger — **+247%**, 37% of boards unreproducible, and
  two venues contradicting each other on one week. New rule, added to that page: **before
  trusting any feed, reconstruct at least three past settlements from the live feed and check
  they match what the venue actually paid.** It costs one afternoon and it is the only way to
  learn that your backtest target is fiction.

## 9. Forward pointer for the next cycle — the one clean hole in Kalshi's catalogue

Screening all 12,186 Kalshi series against Polymarket's structural families found Kalshi
covering essentially everything we have considered: Rotten Tomatoes (244 series), Netflix
rankings (25), MrBeast/YouTube views (`KXMRBEAST100M`, `KXYTDAILYTOPVIDEO`), GPU rental
prices (`KXH100W`, `KXB200W`), metro home values (`KXUSHOMEVAL` and per-city siblings),
reality-TV eliminations (`KXBIGBROTHERELIMINATION`, `KXSURVIVORELIMINATION`), chess (31),
earthquakes (9), UK by-elections (16), Emmys (30).

**The exception is domestic box office.** Kalshi lists exactly two matches and both are
Golden Globe *award* markets (`KXGGBOFILM`, `KXGGBOXOFFICE`) — **no opening-weekend, no
weekend-gross, no total-domestic-gross series anywhere in the catalogue.** Polymarket's
family is large and deep: `avatar-fire-and-ash-opening-weekend-box-office` $17.1M,
`moana-2-5-day-opening-weekend-box-office` $4.4M, `joker-folie-deux-...` $2.6M,
`wicked-for-good-...` $2.4M, dozens more at $300k-$1.4M, plus 2nd/3rd-weekend boards
($5k-$633k) and total-domestic-gross boards; live right now
`the-odyssey-2nd-weekend-box-office-20260720175402816` ($261,409, $78,695/24h, 86-92m leg
at 0.835 on a 0.83/0.84 book) and
`spider-man-brand-new-day-opening-weekend-box-office-20260618144048496` ($204,231).

It also has the right *shape*: it resolves on The Numbers' **final** daily figures,
explicitly *"not studio estimates"*, and stays open until Box Office Mojo and The Numbers
both confirm — i.e. the number the crowd sees on Sunday is a different statistic from the
one that settles, which is the Tomatometer pattern.

**This is a lead, NOT a filed idea, and it must not be promoted until gate 0 is run.** The
incumbent to measure is BoxOfficePro's long-range forecast, which publishes numeric
opening-weekend ranges and — verified today — **is archived in Wayback** under
`boxofficepro.com/long-range-box-office-forecast*` (the live site 403s us; Wayback does not).
The gate-0 job is precisely: pull N archived forecasts, match to films, and compare
BoxOfficePro's range both to the actual and to Polymarket's pre-release price. If
BoxOfficePro is unbiased and Polymarket tracks it, box office dies exactly the way this
idea did — and that is a half-day of work, not a slot.
