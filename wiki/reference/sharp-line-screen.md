# The sharp-line screen — check against a real bookmaker before spending a slot

For any market that a professional betting operation also prices — **sports, esports,
elections, anything with a bookmaker or a betting exchange** — fetch the sharp line
*first*. It costs minutes and it is the cheapest kill available anywhere in our process.

Proven 2026-07-25: `series-shape/bo3-derivatives` was killed on day 1 by this screen
alone. The idea claimed +6 to +14pp of edge across three legs. Against Pinnacle, on
books with ≤2c spread, Polymarket's mean deviation was **−0.13pp (se 0.34)**, median
|Δ| ≈ 1pp, 28–30 of 33 matched markets inside 3pp. Every claim inverted or vanished.

## Run this first: Kalshi's whole catalogue is ONE unauthenticated call

```
GET https://api.elections.kalshi.com/trade-api/v2/series?limit=1000
```

Returns **12,186 series** (~16MB, no key, verified 2026-07-26) each carrying `ticker`,
`title`, `category` and — the load-bearing field — **`settlement_sources`**, the URLs Kalshi
itself resolves on. Grep it before anything else; it takes seconds and it answers "does a
peer venue price this object, and off which page?" in one shot.

Then, per matched series:

```
GET /trade-api/v2/markets?series_ticker=<T>&limit=1000      # volume_fp, open_interest_fp,
                                                            # yes_bid_dollars/yes_ask_dollars,
                                                            # floor_strike, expiration_value
GET /trade-api/v2/series/<T>/markets/<ticker>/candlesticks?start_ts=&end_ts=&period_interval=60
```

**`expiration_value` is the gift**: the exact settled number, per market. It converts
"is the incumbent sharp?" from an argument into a regression, and it doubles as a
point-in-time vintage record of the resolution source (see
[first-print-vintages](first-print-vintages.md)).

**Measured 2026-07-26 (chokepoint-transit-ladders kill).** Polymarket's Strait-of-Hormuz
weekly ladders looked untouched — a free IMF feed, no bookmaker, zero taker fee, 174 distinct
wallets on a 1-2c book. One call showed Kalshi's `KXHORMUZWEEKLY` declaring
`settlement_sources: IMF PortWatch, portwatch.imf.org/pages/chokepoint6` — **our exact
resolution URL** — with 156k-446k contracts a week and 1c spreads. Its implied median at
window close against 9 realised `expiration_value`s: **mean error +2.63, se 6.19, t = 0.42.**
Unbiased. Same kill as `tomatometer/arrival-drift` a day earlier, found before any modelling.

Two corollaries worth keeping:

- **Absence in the catalogue is informative — but far weaker than it looks.** Screening all
  12,186 series against our structural families found Kalshi covering Rotten Tomatoes (244),
  Netflix ranks (25), MrBeast views, GPU prices, metro home values, reality-TV eliminations,
  chess, earthquakes, Emmys — and **not domestic box office** (2 hits, both Golden Globe
  award markets). That was written up as "the cheapest positive signal we have found". It is
  **not a positive signal at all** — see the next section.
- **Agreement between two retail venues is weak evidence; a venue naming your resolution URL
  is strong evidence.** Check `settlement_sources`, not the title.

## The screen has a blind spot: the counterparty is not always a market

**Measured 2026-07-27 (box-office-weekend-ladders kill), and it cost a full research day.**
Domestic box office is the one clean hole in Kalshi's 12,186 series. Pinnacle lists sport 58
"Entertainment" with `matchupCount: 0` and `/sports/58/leagues` → `[]`. Smarkets has one
annual *winner* market and one empty placeholder. No venue anywhere prices a weekend-gross
ladder. Polymarket's feed is perfect too — 98 of 98 settled boards rebuilt to the exact
bucket from the live pages.

And the family is still dead, because **a retired industry analyst gives the forecast away
for free**. Shawn Robbins (ex-BoxOfficePro chief analyst) posts a point forecast for every
holdover weekend, by film and weekend ordinal, on a free Substack: 61 issues since
2025-01-22, every one `audience: "everyone"`, published Wednesdays, **~10% MAPE**. The
market's implied lognormal sigma is **0.120** — which is that MAPE. The price *is* his
forecast plus its error distribution.

Rules that follow:

1. **Run "does a specialist publish this free?" as question one, not question three.** It is
   the same kill as DataGolf (golf) and FanGraphs (MLB); this is now the third family lost to
   it, and the first where no venue check could ever have found it.
2. **An empty catalogue slot answers "does a *venue* price this?", never "is this
   unforecast?"** Kalshi lists what people want to hedge or gamble on, not what is hard.
   A hole means *the cheapest kill is unavailable*, so the remaining gates must carry more
   weight — the opposite of the encouragement it feels like.
3. **Look for the forecast in places that are not web pages.** Robbins' numbers live in a
   PNG table inside a Substack post — invisible to any title grep, any `settlement_sources`
   scan, and any search for "API". Check newsletters, forums and podcasts in the object's
   own subculture before concluding nobody is modelling it.
4. **The market's implied sigma names the incumbent.** If the crowd's de-vigged distribution
   is tighter than anything your data supports, someone published the number. Fit the sigma
   first; it tells you a specialist exists before you have found them.

## Free, read-only sources (verified working from our environment)

- **Kalshi** — see above; open, unauthenticated, and the single highest-yield check we own.
- **Pinnacle** — public guest API: `guest.api.arcadia.pinnacle.com/0.1/sports/{id}/matchups`,
  then `/matchups/{id}/markets/related/straight`. Publishes spreads and totals including
  `bestOfX` series markets. Pinnacle is the reference sharp book; it moves on money.
- **Smarkets v3** — an **exchange**, so the back/lay midpoint carries no vig at all and
  needs no de-vigging assumption.
- Retail books (server-rendered HTML) as a cross-check on the above.

De-vig yourself (normalisation *and* a power fit) rather than trusting anyone's
pre-computed "fair" number, and match markets by their *semantics*, not their titles.

**Replicated same day on tennis** (market-researcher cycle 3, 2026-07-25): Pinnacle lists
tennis **total games** as a *separate matchup* from sets — participants carry a `(Games)`
suffix (79 of 161 pre-match matchups). Across 27 matched total-games lines on 13 live
matches, Polymarket's mean deviation was **+0.32pp (se 0.12), median |Δ| 0.36pp, 27/27
inside 3pp**; on Polymarket books with spread ≤3c, **+0.07pp (se 0.13)**. A 14-leg
derivative ladder that looked untouched — $27k of depth inside 5c on a leg with **$0**
traded volume — turned out to be the Pinnacle line mirrored by a market maker. Deep
resting orders with no taker volume are evidence of a *mirrored sharp line*, not of
absent price discovery.

## How to use it

- **Before a slot is spent**, not after: if the sharp line agrees with the venue within
  a few pp, your claimed edge is a measurement error in your own pipeline.
- Disagreement is not automatically edge — first ask whether the two markets resolve on
  exactly the same event (settlement rules, cancellations, overtime, forfeits differ).
- **When no sharp line exists** — LMArena rankings, weather stations, obscure indices —
  say so explicitly in the idea file. The absence of a professional counterparty is one
  of the few genuinely good reasons to expect an edge to survive (see
  [market-selection](../market-selection.md)), but it also removes your cheapest check,
  so the remaining gates must carry more weight.
