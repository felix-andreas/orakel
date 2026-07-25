---
date: 2026-07-25
slug: esports-series-shape
status: trialing # -> strategies/series-shape/bo3-derivatives (slot 3, 2026-07-25)        # backlog | trialing | discarded-idea | promoted
example_markets: ["lol-vit-g2-2026-07-25", "cs2-tl1-g2-2026-07-25", "lol-tes-tt-2026-07-25"]
model: opus-5 (xhigh)
---

# Esports BO3 series-shape: trade the derivative legs, not the moneyline

**Level or shape? SHAPE.** We do not forecast a single match. We take the market's own
moneyline as the level and claim its *distribution over series scores* (2-0 / 2-1 / 1-2 /
0-2), traded on two separate thin books, is mis-allocated. No team model, no player data,
no external ratings anywhere in the pipeline.

---

## The four screens, answered first

### 0. Am I reading the exact object the market resolves on? (the arena lesson)

I classified nothing by slug or title text. Every leg is typed by Gamma's own
`sportsMarketType` field — `moneyline` / `map_handicap` / `totals` — and I read each
market's full `description` before using it:

- `map_handicap`: *"will resolve to G2 if G2 wins 2 or more maps than Liquid in this
  match"* → `outcomes[0]` is the −1.5 team, and the leg is exactly **P(that team wins 2-0)**.
- `totals` (O/U 2.5 Games): *"will resolve to Over if Liquid and G2 play 3 or more maps in
  this series"* → **P(series goes the distance)**.

Three independent verifications that the labels are right:

1. **Token↔outcome ordering**: `clobTokenIds[i]`'s CLOB price series converges to 0.9995
   exactly when `outcomes[i]` is the winner (checked on sampled events at 1-min fidelity).
2. **The −1.5 side is the market favourite in 94.9%** of instances; the 5.1% exceptions are
   genuine pick'ems (moneyline 0.49–0.51), not label inversions.
3. **The exact three-leg identity** `HC_cover ⇔ (fav wins match) ∧ (Under 2.5)` holds in
   **6,705 / 6,710** resolved series (99.93%). A label inversion anywhere would have
   destroyed this. The 5 violations are named below and must be inspected, not dropped.

### 1. Speed race — is the edge inside 3 minutes? **NO, and there is no print to race.**

There is no resolution-data publication before the match. The mispricing is model-revealed
and is only paid at resolution. Measured on 504 resolved series with a real handicap book
(`hc_vol > $5k`):

| checkpoint | mean market P(fav −1.5 covers) |
| --- | --- |
| T−24h | 0.519 |
| T−6h | 0.531 |
| T−1h | 0.531 |
| T−15m | 0.537 |
| **realized cover rate** | **0.683** |

The price does not converge toward the truth at all during the pre-match window — median
\|Δ\| from T−6h to T−1h is **1.5c**. This is the exact opposite of the temp-truncation kill
(dead legs collapsed in 0–3 min).

**Delayed-execution test already run** (`wiki/reference/delayed-execution-test.md`), signal
frozen at t, fill at the later mid + 2c adverse, buy handicap in band 0.20–0.60, `hc_vol>$5k`:

| signal → fill | n | instant | delayed | se |
| --- | --- | --- | --- | --- |
| T−6h → T−1h | 555 | +18.9c | **+16.1c** | 2.0c |
| T−6h → T−15m | 555 | +18.9c | **+14.7c** | 2.0c |
| T−24h → T−6h | 390 | +17.2c | **+13.1c** | 2.4c |

No collapse under delay. Cadence fit: the CEO trigger fires 01:07 UTC and the daily slate
starts 08:00–20:00 UTC, i.e. a morning run sits at T−7h to T−19h — precisely the zone the
test covers.

### 2. Proxy-vs-primary / glanceable state / **who is the sharpest agent already here?**

- **No proxy layer at all.** The resolution variable is the match result itself (`hltv.org`
  for CS2, official league pages elsewhere), published once, observed by everyone
  simultaneously. There is no upstream input to run better (the gistemp failure) and no
  in-window state to glance at (the Netflix failure) — the series has not started when we
  trade.
- **The plausible sharp incumbent is an esports betting syndicate** bridging a
  Pinnacle/bet365 map-handicap line into Polymarket. Three pieces of evidence that they are
  not here in force:
  1. The **moneyline itself** — deep, 1c spreads, $33k–$81k median volume — is miscalibrated
     by **+6.1pp** (favourite wins 78.8% vs mean price 72.7%, n=2,000 at T−1h).
  2. The bias is **larger on tier-1 events than on obscure qualifiers** — the opposite sign
     to an "informed money bridges the big games" story, and the signature of fan money:

     | universe | n | ML mkt→real | HC mkt→real | OU mkt→real |
     | --- | --- | --- | --- | --- |
     | LEC/LPL/LCK/VCT/BLAST/IEM/ESL/Masters | 719 | 0.736→0.837 (**+10.2pp**) | 0.500→0.666 (**+16.6pp**) | 0.416→0.289 (**−12.7pp**) |
     | everything else | 1,279 | 0.723→0.761 (+3.8pp) | 0.503→0.550 (+4.7pp) | 0.426→0.341 (−8.5pp) |
  3. The derivative books are **5–20× thinner than the moneyline sitting next to them**
     (median `map_handicap` $1.4k CS2 / $4.6k LoL vs moneyline $33k / $81k). The arb that
     would tie them together has not been built.
- **Open item I could not close from this box, and it is the single best kill shot:**
  I could not read an external bookmaker line — `hltv.org` returns **403** through the agent
  proxy and `the-odds-api.com` requires a paid key. **Day-1 task: obtain one free
  map-handicap odds feed and measure the Polymarket-minus-bookmaker gap directly.**

### 3. First-print vintages

Not applicable in the usual sense, and that is a feature: a match result is published once
and never revised, so there is no vintage to reconstruct and no first-print/settled
divergence to corrupt the backtest. Scoring uses the venue's own settled `outcomePrices`
(`["1","0"]`) — the exact object the market paid on.

The residual analogue is **venue adjudication error on forfeits, walkovers and BO5
re-designations**: 5 of 6,710 series (0.07%) break the three-leg identity —
`val-nbls-qe-2026-01-10`, `cs2-uvs-lk-2026-01-09`, `cs2-ill-uvs-2026-01-12`,
`cs2-fdb-bsta-2026-02-03`, `cs2-vpp-lag2-2026-02-09`. Inspect them; they are this family's
`venue-resolution-epsilon`.

### 4. Published CI vs printed (`wiki/reference/published-ci-vs-printed.md`)

Nothing in this family publishes error bars, so the literal trap cannot spring. But the
**generalised rule — estimate σ from realised dynamics of the printed series, never from a
model's own uncertainty — is load-bearing here, and it flips the sign of the trade**:

> The naive closed-form map (independent maps, per-map p solving p²(3−2p) = moneyline) says
> that at a moneyline of 0.748 the sweep should trade at **0.452**. The market quotes
> **0.490** and the realised rate is **0.593**. A strategy built on the tidy analytic model
> would have concluded the sweep was *overpriced* and taken the losing side.

So the map from moneyline to series-score distribution is fitted **non-parametrically on
resolved outcomes**, never assumed. Every number in this file is a count of realised
outcomes or a market price — no simulation, no assumed correlation, no publisher's CI.

---

## Thesis (mechanism, precisely)

Polymarket lists every esports BO3 as a bundle of **separately-traded books on one event**:
a deep `moneyline`, plus two thin derivatives — `map_handicap` (favourite −1.5 ≡ "wins 2-0")
and `totals` (O/U 2.5 maps ≡ "goes the distance"). They are linked by an exact identity:

```
P(fav 2-0) + P(dog 2-0) + P(Over 2.5) = 1
P(fav match) = P(fav 2-0) + P(Over 2.5) · P(fav wins the decider)
```

Three stacked distortions, measured at T−1h on 1,998 resolved series:

**(A) Favourite-longshot bias on the moneyline (the level is tilted).**
Favourite wins **78.8%** vs mean price **72.7%** (+6.1pp, se 0.9pp, n=2,000). This alone is the
mechanism already running in `arena-rank/favourite-shrinkage` — we do not claim it as new.

**(B) Convex transfer into the derivative legs (the new part).**
The derivative books price the sweep *coherently with the biased moneyline*, and because
P(sweep) is a convex function of P(match), a δ error in the moneyline becomes a **2–3×
larger error** in the handicap and totals legs:

| moneyline band | n | ML mkt→real | HC mkt→real | OU mkt→real | amplification |
| --- | --- | --- | --- | --- | --- |
| 0.5–0.6 | 305 | 0.550→0.652 (+10.2) | 0.401→0.387 (−1.4) | 0.491→0.446 (−4.5) | — |
| 0.6–0.7 | 465 | 0.653→0.723 (+7.0) | 0.415→0.510 (**+9.5**) | 0.469→0.372 (**−9.7**) | 1.4× |
| 0.7–0.8 | 528 | 0.748→0.841 (+9.3) | 0.490→0.593 (**+10.3**) | 0.437→0.343 (**−9.4**) | 1.1× |
| 0.8–0.9 | 443 | 0.845→0.901 (+5.6) | 0.596→0.734 (**+13.8**) | 0.368→0.237 (**−13.1**) | **2.5×** |
| 0.9–1.0 | 181 | 0.932→0.972 (+4.0) | 0.740→0.873 (**+13.3**) | 0.252→0.122 (**−13.1**) | **3.3×** |

**(C) An independent "goes the distance" premium.** The Over-2.5 leg is overpriced in
**8 of 8 cohort-months** (monthly-clustered mean **−9.0pp, se 1.75pp, t = −5.16, 0/8
positive**) — *including* Dec-25 / Jan-26 / Feb-26, when the moneyline bias was ≈0
(+2.4pp, +2.5pp). So part of the totals distortion is **not inherited** from (A): it is the
classic action/Over bias, and it is the most regime-stable signal in the study.

**Who is on the wrong side.** Retail fans buying their team as an underdog, and retail
buying "Over" for a longer, more watchable series. The taker tape says so directly: over
300 sampled events in the [T−6h, T−15m] window, taker BUYs of the moneyline underdog lost
**−18.3c/share** across 4.43M shares while taker BUYs of the favourite made **+16.1c/share**
across 4.76M shares; on the handicap, BUYs of the +1.5 side lost **−29.7c/share**.

**Why this beats running the same bias on the moneyline** — and why it is worth a slot next
to `arena-rank/favourite-shrinkage` rather than duplicating it: that variant's
pre-registered day-3 problem is the **fundable band** (its favourites sit at 0.93–0.99, so
return on locked capital is poor even when the edge is real). Here the *same class* of bias
is expressed on a leg priced at **0.40–0.60** — half the stake for a larger absolute edge.
This idea is the fundable-band version of a bias the firm has already proven it can measure.

---

## Example markets — real numbers, pulled 2026-07-25 08:47 UTC

Fair values below are the **empirical realised rate in that moneyline band** from the table
above (n=1,998, T−1h) — not a model output.

**1. `lol-vit-g2-2026-07-25` — LoL: Team Vitality vs G2 Esports (BO3), LEC Regular Season.**
Start 14:00 UTC (T−5.2h at read).
- Moneyline: **G2 0.815** / Vitality 0.185 — vol **$91,198**, spread **1c**.
- **Map Handicap G2 (−1.5) = 0.585** — bid 0.58 / ask 0.59, spread **1c**, vol **$39,360**,
  liquidity **$139,593**.
- O/U 2.5 Over = 0.345 — bid 0.34 / ask 0.35, vol $3,241, liq $45,302.
- Band 0.8–0.9 ⇒ empirical P(cover) **0.734** vs market 0.585 → **+14.9pp**. Buy at ask
  0.59; sports taker fee = 0.05 × 0.585 × 0.415 = **1.21c** ⇒ net ≈ **+13.3c on a 59c stake**.

**2. `cs2-tl1-g2-2026-07-25` — Counter-Strike: Liquid vs G2 (BO3), BLAST Bounty Qualifier.**
Start 12:30 UTC (T−3.7h at read).
- Moneyline: **G2 0.665** / Liquid 0.335 — vol **$174,099**, spread 1c.
- **Map Handicap G2 (−1.5) = 0.415** — bid 0.41 / ask 0.42, 1c, vol **$27,917**, liq $35,073.
- **O/U 2.5 Over = 0.455** — bid 0.45 / ask 0.46, 1c, vol $1,605, liq $26,464.
- Band 0.6–0.7 ⇒ empirical HC **0.510** vs 0.415 (**+9.5pp**), empirical Over **0.372** vs
  0.455 (**−8.3pp**, i.e. sell the Over). Both legs of the same event point the same way —
  a coherence cross-check the strategy should require before trading.

**3. `lol-tes-tt-2026-07-25` — LoL: Top Esports vs ThunderTalk Gaming (BO3), LPL.**
Start ~11:00 UTC (T−2.2h at read).
- Moneyline: **Top Esports 0.805** — vol $16,666, spread 1c.
- **Map Handicap TES (−1.5) = 0.525** — bid 0.52 / ask 0.53, 1c, vol **$30,876**,
  liquidity **$157,823**.
- Band 0.8–0.9 ⇒ empirical **0.734** vs 0.525 → **+20.9pp**, in the middle of the fundable band.

**Fee, exactly** (documented, and on every leg's `feeSchedule`
`{rate: 0.05, exponent: 1, takerOnly: true, rebateRate: 0.15}`):
`fee = shares × 0.05 × p × (1 − p)` — **1.25c/share at p=0.50, 1.20c at 0.40, 1.21c at 0.585**.
Makers pay nothing. Small relative to the measured edge, but it must be in every PnL line.

---

## Resolved-instance supply for backtesting

- **6,710 resolved BO3 series**, 2025-12-15 → 2026-07-25, **8 cohort-months**, each with a
  matched `moneyline` + `map_handicap` + `totals` triple and a known outcome.
  Per game: cs2 2,833 · val 1,266 · lol 868 · dota2 781 · r6siege 363 · sc2 337 · mlbb 262.
  Monthly: Dec 84 · Jan 867 · Feb 1,072 · Mar 1,046 · Apr 1,350 · May 1,224 · Jun 585 · Jul 482.
- Realised joint distribution (n=6,710): fav 2-0 **57.0%**, fav 2-1 20.2%, dog 2-1 12.5%,
  dog 2-0 10.3% → favourite wins 77.2%, series goes 3 maps 32.7%.
- **Per-leg CLOB history: yes.** `prices-history?market=<tokenId>&startTs=<gameStartTime−129600>&fidelity=10`
  returned a **non-empty** series for **6,375 / 6,375** tokens fetched (100%). Checkpoint coverage:
  T−15m 96%, T−1h 94%, T−6h 89%, T−24h 51% (books typically open ~40h before start).
- **Full taker tape per leg** via `data-api /trades?market=<conditionId>` — used above for
  executed-price PnL, so the backtest never has to trust a midpoint.
- **Tradeable subset** (fundable band 0.20–0.60 + real book): handicap **552 instances**
  with `vol > $5k` ≈ **2.5/day**; totals **676 instances** with `vol > $2k` ≈ **3/day**.
- **Forward cadence: ~30–40 new full-structure BO3s resolve per day.** Fastest-scoring
  family the firm has looked at.
- **Discovery recipe (new, non-obvious):** Gamma `/events?closed=true&tag_id=64` offset
  paging hard-caps at **offset 2000**; `/events/keyset` ignores a `cursor` param (the
  documented name is `after_cursor`). What works is **date-windowed offset paging** —
  weekly `end_date_min` / `end_date_max` windows × offset — which harvested 16,959 resolved
  esports events cleanly. Added to `wiki/recipes/polymarket-api.md`.

---

## Falsification sketch — explicit kill conditions

Ordered so the cheapest kill runs first.

**Gate 0 — artifact hunt (run this first; my own prior is that a bug is the leading
hypothesis).** An edge this large on 1c-spread books with $30k–$170k of liquidity should
not exist. Three checks, all of which I ran once and which must be redone on the full
sample by an independent code path:
- *Disjoint universe.* Re-measure on events my sample excluded. Already done once on a
  500-event random sample with `hc_vol ≤ $1,000` (disjoint from the main sample):
  ML **+7.0pp** (se 2.0), Over **−10.0pp** (se 2.2) — the effect is **not** a selection
  artifact of the volume filter.
- *Is the window really pre-match?* `gameStartTime` must be validated, not trusted. Spot
  checks show pre-window price sd of 0.005–0.13 and an immediate collapse to 0.0005/0.9995
  after `gameStartTime`; the effect is the same size at T−24h, which cannot be in-play.
- *Midpoint vs executed price.* Taker BUY VWAP is only **+0.68c above the T−1h midpoint**
  (median +0.5c), and buying the favourite at real fills returned **+5.4c/event**
  equal-weighted (n=278). The midpoint is executable.
> **KILL if** the effect fails to reproduce on a disjoint universe, or if the executed-price
> version is more than 3c worse than the midpoint version.

**Gate 1 — reproduce the resolution ledger.** Rebuild the 6,710-instance table from scratch;
require ≥99% three-leg identity, and spot-check 50 series against liquipedia/hltv.
> **KILL if** >2% disagree with an external result source.

**Gate 2 — does a LOO-fitted map beat the market's own derivative price?** Fit the
moneyline→P(sweep) and moneyline→P(Over) maps **leave-one-month-out** over the 8 cohort
months (non-parametric / isotonic; never the analytic independence formula — see screen 4).
Score log-loss of the mapped price vs the market's handicap and totals prices on held-out
months.
> **KILL if** the mapped price does not beat the market out-of-sample in ≥6/8 months, or if
> the monthly-clustered mean gain is <3pp with t < 2.

**Gate 3 — net of fees, spread and delay.** Simulate at the **ask**, plus
`fee = shares × 0.05 × p(1−p)`, signal frozen at the daily run, fill at T−6h, +2c adverse,
hold to resolution.
> **KILL if** net edge < 4c/share, or if it **sign-flips between the Dec–Feb and Apr–Jul
> halves**. (Note upfront: the moneyline component of the bias is ≈0 in Jan–Feb and +7 to
> +11pp from Apr. The totals component is negative in 8/8 months and is the part that
> survives a regime change; a variant that only works post-April must say so.)

**Gate 4 — book quality and wash trading.** `wiki/reference/thin-market-price-read.md` gate:
spread ≤5c, top-of-book ≥$500, plus `wiki/reference/wash-trading.md` tests. Today's live
handicap books show **37–60 distinct wallets but a top-1 taker share of 39–64%** — real
books with one large participant, which must be characterised before sizing.
> **KILL if** fewer than ~1 event/day clears the book gate in the fundable band.

**Gate 5 — the incumbent.** Fetch an external map-handicap closing line.
> **KILL if** the bookmaker line agrees with Polymarket's handicap within 3pp — that would
> mean the sharp price is already here and the resolved-outcome sample is a fluke.

**Pre-registered live stop:** if the first 30 forward trades show a monthly-clustered edge
below 3pp, stop regardless of the backtest.

---

## Also scanned today and not filed (cheap negative knowledge)

- **Company market-cap ranking boards** (`largest-company-end-of-july` 29 legs $4.18M;
  `2nd-largest` $437k; `3rd-largest` $302k). Rejected on the glanceable-state screen: the
  resolution variable is a live stock price anyone can read, and the ordering is
  near-deterministic. Same shape as arena's one-object/many-boards family with none of the
  hidden state.
- **Non-US central-bank decision boards** (BOJ July $300k, Banco do Brasil Aug $159k,
  ECB Sep $141k; ~100 instances/year across a dozen banks; genuinely thin-to-mid). Parked,
  not rejected: the sharpest agent is a professional rates desk pricing the local OIS curve
  — the gistemp proxy-vs-primary failure mode — and we cannot cheaply read those curves.
  Worth revisiting only with a free swap-curve source.
- **US primary-election winner boards** (Aug 4 / 11 / 18 slates; 18–50 legs; $160k–$2.1M;
  dozens already resolved this cycle). Real supply, real fine print (runoff triggers,
  Alaska's top-4 primary, advance-vs-win coherence constraints). A genuine future candidate
  — parked rather than filed because its cadence is election-calendar-bound, not daily.
