---
date: 2026-07-28
slug: mention-markets
status: discarded-idea
killed_by: executable-price decomposition — the entire +4.6pp apparent YES-overpricing equals mean(last trade − bid) = 6.18c to the cent; at the bid both sides lose (buy-NO −2.5pp, buy-YES −7.9pp), 0 of 5 bands clear break-even under the firm's own relative-spread gate, and the one filter that rescues it (lifetime volume) is look-ahead
example_markets:
  [
    "will-warsh-say-projection-during-july-press-conference-20260720204",
    "will-warsh-say-inflation-20-times-during-july-press-conference-202",
    "will-boeing-say-guidance-during-earnings-call-20260724221853548",
    "will-paypal-say-google-during-earnings-call-20260724221739519",
  ]
model: claude-opus-5 (effort max)
summary: >-
  "Will X say WORD during Y" boards are a large recurring two-venue family — Kalshi runs 397
  Mentions series ($310M settled in 3 months), Polymarket 447 events ($301M lifetime). The
  crowd looks badly wrong: at a pre-event checkpoint, realised YES frequency sits 4.6-6.5pp
  BELOW the traded price in every band from 0.10 to 0.90, event-clustered t up to +6.9, and
  it survives out to T−48h so it is not intra-speech decay. It is not an edge. The gap is
  the spread: mean(last trade − bid) is 6.18c and the last-trade-minus-executable difference
  is 6.18pp, identical to two decimals. At the bid, buying NO returns −2.5pp and buying YES
  −7.9pp — both sides lose simultaneously, the signature of having measured a spread rather
  than a mispricing. Under the firm's relative-spread gate 0 of 5 price bands clear
  break-even. Polymarket is not a softer crowd: on 19 matched phrases with real books on both
  venues it sits +1.87pp above Kalshi (se 0.59), 18/19 within 5pp.
---

# "Will X say WORD" mention markets — the crowd looks 5pp wrong and is not

**Level or shape? LEVEL** — the claim would have been that we estimate P(utterance) better
than the crowd, from a transcript/history corpus nobody assembles. It never got as far as
needing a model.

## Why I picked it up

It clears every positive screen the firm owns, which is why it was worth a day:

- **Not an efficient object.** The resolution variable is whether a human says a word. There
  is no quoted underlying whose price *is* the forecast (Felix's standing constraint).
- **Sim-tractable with a hidden-but-recoverable state.** P(Powell says "transitory") is a
  Bernoulli with a strongly persistent per-(speaker, phrase) rate. Nobody can *look it up*;
  it has to be assembled from transcripts. That is the exact profile
  `market-selection.md` says to hunt for — hidden from the amateur, recoverable by work.
- **No free specialist.** No DataGolf, no FanGraphs, no Substack PNG. Nobody publishes
  utterance probabilities. This is the first family in six runs where the
  "does a specialist publish this free?" question genuinely returns nothing.
- **Fast resolution** (hours), **recurring** (8 FOMC pressers/yr, hundreds of earnings calls,
  weekly Trump boards), **genuine uncertainty** (per-word base rates spread 0.06-0.91).
- **It passes the tape gate**, which almost nothing we look at does — see below.

And a measurable incumbent, so gate 0 could be *run* rather than described.

## Gate 0 — MEASURED, not described

**Kalshi runs an entire `Mentions` category: 397 series, 3.2% of the 12,231-series
catalogue** (catalogue pulled today; it was 12,187 yesterday). Pulled every market in all 397
series: **17,001 markets, 15,258 settled with a `result`, $310.6M of settled volume, 813
events, median 17 legs/event, all within 2026-05 → 2026-07.** Base rate 43.1% YES.

**Polymarket runs the same family**: 447 mention-shaped events, **$301.5M lifetime**, with
single boards at $53.2M (`what-will-trump-say-during-bilateral-events-with-xi-jinping`,
33 legs), $10.3M (inauguration), $6.75M (State of the Union), $5.5M
(`what-will-powell-say-during-june-press-conference`, 18 legs). 14 boards open today.

So both venues are in it at scale. That alone kills nothing — two retail venues agreeing is
weak evidence (`sharp-line-screen.md`). What matters is whether the line is right.

## The measurement that looked like an edge

Checkpoint construction: for each Kalshi event, `T = min(close_time)` across its legs (the
first leg to resolve). For each leg I took the last hourly candle at or before `T − L` for
L ∈ {1,3,6,12,24,48}h, using `price.close_dollars` (a real trade, so not a phantom midpoint)
and the contemporaneous `yes_bid`/`yes_ask` (present on 100% of candles). Sample: the 6,117
markets in the 51 `KXEARNINGSMENTION*` series plus 17 named political/media speaker series —
the sub-families Polymarket also runs. 6,032 have a clean price at T−1h.

Calibration at the pre-event checkpoint, **last-trade price**:

| price band | n | mean price | realised freq | diff |
|---|---:|---:|---:|---:|
| 0.10–0.25 | 616 | 0.173 | 0.107 | **−6.6pp** |
| 0.25–0.40 | 578 | 0.316 | 0.244 | **−7.2pp** |
| 0.40–0.60 | 555 | 0.498 | 0.405 | **−9.2pp** |
| 0.60–0.75 | 506 | 0.666 | 0.520 | **−14.6pp** |
| 0.75–0.90 | 471 | 0.817 | 0.754 | **−6.3pp** |

Every band from 0.10 to 0.90 is negative — uniform YES-overpricing, not favourite–longshot
(which would change sign across the range). The obvious mechanism is one-sided demand:
"he'll definitely say it" is the fun side, and the NO side is capital locked up for a boring
payoff.

**It is not an intra-speech artifact.** 40.6% of all volume trades in the final hour, so
T−1h risks sitting inside the event. It does not matter — the bias survives at every lead,
event-clustered over 156–268 events:

| lead | legs | events | last-trade edge (p − y) | t |
|---|---:|---:|---:|---:|
| T−3h | 4,115 | 268 | **+6.51pp** | +6.93 |
| T−6h | 3,996 | 252 | **+5.13pp** | +5.27 |
| T−12h | 3,618 | 239 | **+5.36pp** | +5.29 |
| T−24h | 2,382 | 156 | **+4.15pp** | +3.71 |
| T−48h | 1,560 | — | **+6.7pp** | — |

A +5pp, t≈+5 bias, clustered at the event level, two full days before the speech. That is
the point at which this looked like the best thing the firm had found.

## The measurement that killed it

**Re-price the same trade at the price you can actually get.** To harvest "YES is
overpriced" you buy NO at `1 − yes_bid`, so the edge is `yes_bid − y`, not `price − y`.

| lead | events | last trade (p − y) | **executable buy-NO (bid − y)** | **executable buy-YES (y − ask)** |
|---|---:|---:|---:|---:|
| T−3h | 268 | +6.51pp (t +6.93) | **−1.76pp (t −1.79)** | −9.09pp (t −9.49) |
| T−6h | 252 | +5.13pp (t +5.27) | **−2.51pp (t −2.48)** | −7.92pp (t −7.97) |
| T−12h | 239 | +5.36pp (t +5.29) | **−2.89pp (t −2.56)** | −8.65pp (t −8.16) |
| T−24h | 156 | +4.15pp (t +3.71) | **−3.13pp (t −2.55)** | −6.88pp (t −5.79) |

**Both sides lose at the same time.** That is arithmetically only possible if what I measured
was the spread, and the decomposition confirms it to the cent (mid-band, T−6h, n=3,996):

```
mean spread                  8.70c
mean (last trade − mid)     +1.83c    <- mild buy pressure, the real but tiny demand effect
mean (last trade − bid)     +6.18c
last-trade edge (p − y)     +4.56pp
executable edge (bid − y)   −1.63pp
                             ------
difference                   6.18pp   == mean(last trade − bid), identically
```

Only 45.9% of events show a positive executable buy-NO edge — worse than a coin flip.

**Conditioning on tightness does not rescue it** (T−6h, mid-band, event-clustered):

| filter | legs | events | buy-NO | t |
|---|---:|---:|---:|---:|
| all | 3,996 | 252 | −2.51pp | −2.48 |
| spread ≤ 1c | 1,095 | 182 | +3.35pp | +1.60 |
| spread ≤ 2c | 1,480 | 207 | +1.39pp | +0.78 |
| spread ≤ 3c | 1,791 | 219 | +1.26pp | +0.77 |
| **rel-spread gate** `spr ≤ min(5c, ½·mid)` | 2,233 | 224 | **+0.60pp** | **+0.40** |

Under the firm's own `tape-gate.md` relative-spread rule the edge is **+0.60pp ± 1.5**,
indistinguishable from zero. Nothing survives per-series either: of the twelve series with
enough tight-spread legs, none is significantly positive and the only two significant
coefficients are *negative* (`KXPOLITICSMENTION` −17.5pp t=−2.57, `KXRUBIOMENTION` −10.0pp
t=−3.76).

Splitting Bernoulli legs from **counting legs** ("say X 3+ times", the shape that dominates
Polymarket's Warsh board) changes nothing: counting legs at spread ≤3c give +1.66pp,
**t = +0.24**, n=40.

## The trap inside the kill: filtering on lifetime volume is look-ahead

The one filter that appears to rescue everything:

| filter | legs | events | buy-NO | t |
|---|---:|---:|---:|---:|
| **lifetime** `volume_fp ≥ 20k` | 829 | 143 | **+21.15pp** | **+7.30** |
| **honest** pre-checkpoint volume ≥ 20k | 57 | 13 | +6.04pp | +0.59 |
| **honest** pre-checkpoint volume ≥ 5k | 388 | 60 | **−3.06pp** | −0.72 |
| honest pre-vol ≥5k AND spread ≤3c | 316 | 52 | −3.76pp | −0.83 |

`volume_fp` is *lifetime* volume, and **only 14.3% (median) of a mention leg's lifetime
volume trades before T−6h** — the event itself is when the tape happens. So "keep the liquid
legs" silently keeps the legs that were about to trade heavily, which correlates with how
they resolved. A +21pp, t=+7.3 result evaporates to −3.06pp when the same threshold is
applied to volume *known at the checkpoint*. Graduated to
`wiki/reference/lifetime-volume-is-look-ahead.md`.

## Is Polymarket a softer crowd? No — measured

Polymarket's tradeable legs are genuinely tighter than Kalshi's (median 4.8c across all live
legs, but 1–2c on the active ones vs Kalshi's 8.70c mean), so the kill only transfers if the
two crowds agree. Matched by quoted phrase on today's overlapping boards — Kalshi
`KXWARSHMENTION`, `KXEARNINGSMENTIONBA/PG/PYPL/AAPL/META/MSFT` against the equivalent
Polymarket events — 52 phrase pairs, of which **19 have a real book on both venues**
(spread ≤10c both sides, Polymarket volume ≥ $200):

> **Polymarket mid − Kalshi mid = +1.87pp, se 0.59, t = +3.16, n = 19.
> Median |Δ| 2.50pp. 12/19 within 3pp, 18/19 within 5pp, max 6.5pp.**

Same line, ~2pp richer on Polymarket — consistent with the YES-demand story being marginally
stronger there, and far too small to clear a 2c spread plus fee. The remaining 33 pairs were
**phantom midpoints**: the Apple / Meta / Microsoft boards opened today with $34–$178 of
volume and quote 0.02/0.98, which the API reports as a 0.500 mid. Left in, they would have
manufactured ±40pp "cross-venue disagreements" on every leg.

## Fundability

Fees confirmed per-market, not assumed: Polymarket `feeType: mentions_fees`,
`{rate: 0.04, exponent: 1, takerOnly: true, rebateRate: 0.25}` → `0.04·p·(1−p)`, 1.00c/share
at p=0.50. All 397 Kalshi Mentions series are `fee_type: quadratic, fee_multiplier: 1` →
`0.07·p·(1−p)`, 1.75c at p=0.50.

Break-even table on the **best** subset (tightest books, spread ≤1c, n=1,095):

| NO cost band | n | q\* (cost+fee) | q obs | q⁻ 95% lo | verdict |
|---|---:|---:|---:|---:|---|
| 0.05–0.25 | 262 | 0.154 | 0.126 | 0.091 | fails |
| 0.25–0.45 | 238 | 0.362 | 0.424 | 0.363 | clears by 0.1pp |
| 0.45–0.60 | 93 | 0.535 | 0.570 | 0.468 | fails |
| 0.60–0.75 | 184 | 0.690 | 0.658 | 0.586 | fails |
| 0.75–0.95 | 275 | 0.852 | 0.851 | 0.804 | fails |

Under the relative-spread gate instead: **0 of 5 bands clear.** Per-contract on the tightest
books: gross buy-NO +1.65pp, mean Kalshi fee 1.18c, mean Polymarket fee 0.68c → **net +0.46pp
(Kalshi) / +0.97pp (Polymarket)**, i.e. under one cent per contract on a family whose median
leg quotes a 3c spread. The single "clearing" band clears by 0.1pp on 238 legs; that is not
a trade, it is a rounding error.

## What the tape looks like (and the one thing this family does have)

This is the first family the firm has screened that **passes the tape gate outright**. 30
live Polymarket mention legs with spread ≤5c, sampled today:

- **0 of 30 have zero tape.** 3,823 taker trades total, ~127 per leg.
- Top legs: `will-warsh-say-projection...` 418 trades, bid 0.40 / ask 0.41 (1c), $5,431 vol,
  $7,541 liquidity, resolves 2026-07-29. `will-warsh-say-inflation-40-times...` 222 trades,
  0.21/0.22, $5,010. `will-boeing-say-guidance...` 0.38/0.41 (3c), $2,452, resolves 07-28.
  `will-paypal-say-google...` 0.17/0.19, $1,734, resolves 07-28.

Real books, real takers, real spreads, fast resolution — and no edge. Worth recording
precisely because it is the counter-example to the assumption that liquidity is our binding
constraint. Here liquidity was fine and the *price* was right.

## Falsification sketch (pre-registered, and it is what ran)

The idea would have been killed if, at a pre-event checkpoint, the executable edge
(`yes_bid − y` for the NO side, `y − yes_ask` for the YES side) were not reliably positive on
at least one side after fees, event-clustered, on legs passing the relative-spread gate. It
was not: **−2.51pp and −7.92pp respectively**, both sides negative simultaneously, and
+0.60pp (t = +0.40) on the gated subset. Nothing to trial.

## Risks that would have applied had it survived

- **Subjective resolution on both venues.** Kalshi's rules read *"Video of the show will
  primarily be used to resolve the market; if a consensus by Kalshi..."*. Polymarket resolves
  via UMA. Two venues can settle the same utterance differently — plurals, prepared remarks
  vs Q&A, who is speaking, transcription noise. Any cross-venue construction (11 of the 19
  matched pairs nominally *cross*, e.g. Boeing "Tariff" K 0.29/0.30 vs P 0.33/0.34) carries
  correlated settlement risk on both legs at once and should not be read as arbitrage.
- **Resolution-feed session calendar (`stale-feed-gate.md`): not applicable, and worth saying
  so.** The resolving artifact is the video/transcript of the event itself, which is
  generated exactly during the market's live window. There is no feed that can be shut while
  the market moves. This family is structurally clean on the one axis that broke 64 of 95
  rows on the current trial.
- **`can_close_early: True` on Kalshi.** Legs terminate at the utterance, so any intra-event
  strategy is a speed race against bots reading the same livestream — the
  `delayed-execution-test` failure mode, before it even reaches fees.

## Verdict

**discarded-idea.** The family is large, liquid, recurring, fast-resolving, unforecast by any
specialist, and priced by a crowd whose *printed* prices are 4.6–6.5pp too high with
t up to +6.9 out to T−48h. Every one of those points is true and none of them is tradeable:
the printed price is not a price we can get, and at the price we can get, both sides lose.

Do not re-propose "will X say WORD" boards on either venue without a mechanism that does not
route through crossing the spread — a maker-side construction (Polymarket taker-only fees
mean resting orders are free) is the only version of this that is not already dead, and it
is an execution idea, not a research idea.
