---
date: 2026-07-26
slug: frontline-first-passage
status: backlog # backlog | trialing | discarded-idea | promoted
example_markets:
  [
    "will-russia-capture-kostyantynivka-by-august-31-1",
    "will-russia-capture-kostyantynivka-by-september-30-256-333",
    "will-russia-enter-dopropillia-by-december-31-2026",
    "will-russia-enter-druzkhivka-by-december-31-2026",
  ]
model: claude-opus-5 (effort max)
summary: >-
  Polymarket runs ~50 boards asking "will the front line reach this exact spot by
  <date>", resolved off a specific pixel on a public daily GIS layer. The resolution
  variable is the first-passage time of a point by a growing 2-D random set with a
  competing absorbing state (ceasefire) — no closed form, and nobody prices it: no
  bookmaker takes war bets and no institution publishes probabilities, only descriptive
  advance rates. 407 resolved legs, $72M traded, 1c spreads, and Polymarket charges
  ZERO taker fee on geopolitics. Measured: the crowd is 9.95pp too high on Yes at
  T-30d (cluster-robust se 2.31pp over 106 settlements, t=-4.31), and unlike every
  previous candidate the flow to trade against it demonstrably exists on both sides.
---

# The front line as a first-passage problem: ~50 boards, one process, no counterparty

**Level or shape? Honestly, mostly LEVEL — and that is the class that has killed us twice.**
`temp-truncation/runningmax` and `climate-nowcast/gistemp-era5` both died claiming "we estimate
the truth better". I am filing a level claim anyway, and the file has to earn that. The reason
both of those died was the same screen — *the crowd could run better inputs than we could*
(METAR; GHCN-M+ERSST). Here we read **the exact object the market resolves on**, from the
publisher's own machine-readable endpoint, and there is **no professional counterparty at all**.
There is also a genuine shape component (§4.4): all ~116 open legs are functions of one common
process plus one ceasefire hazard that a $5.69M board already prices for us, so cross-sectional
and term-structure coherence are exploitable without any level skill at all. If the level claim
fails in trial, the shape claim is the fallback and it is measurable separately.

---

## 1. What this market is, in plain English

Polymarket lists about fifty separate boards of the form **"Will Russia capture Kostyantynivka
by August 31?"** — one board per town, with several deadline legs (by 31 Jul, by 31 Aug, by 30
Sep, by 31 Dec) trading side by side. There are also mirror-image boards, "Will Ukraine re-enter
Myrnohrad by...".

The thing that decides them is not a journalist's judgement. It is a **specific pixel on a
public map**. Each board names an exact spot — for Kostyantynivka, the railway station on
Pravoberezhna vulytsia, with three photographs and a Google Maps pin — and resolves Yes if the
Institute for the Study of War (ISW) shades that spot red on its daily interactive map. Three of
ISW's shading categories count ("Assessed Russian Control", "Assessed Russian Advance",
"Assessed Russian Gains in the Past 24 Hours"); a fourth, "Assessed Russian Infiltration Areas",
explicitly **does not**. The shading must survive one full daily update cycle. DeepStateMap is
the named fallback if ISW goes dark.

So the question a trader is really answering is: *how long does it take a slowly growing,
occasionally lurching red blob to swallow one particular point?* That is a first-passage-time
problem, and it is exactly what a simulation is for. What the crowd does instead is read the
news, look at the map, and feel that the town is about to fall.

---

## 2. The simulation, concretely

### The process

**A growing 2-D random set, with a competing absorbing state.** Let `Q_t` be the union of the
three qualifying ISW layers at day `t`. A board with resolving point `p` and deadline `D`
resolves Yes iff `p ∈ Q_t` for some `t ≤ D` (and the shading persists one cycle). The dynamics:

- **Boundary advance.** Each segment of `∂Q_t` advances by a daily increment that is
  zero-inflated (most segments, most days, move 0 m) with a heavy right tail (a flank collapse
  moves a whole sector kilometres in a day). CSIS measured roughly **50 m/day** at Kostyantynivka
  in its 1 July report — that mean is a summary of a distribution that is nearly all zeros and
  a few large jumps, and the mean is *not* what prices a threshold.
- **Correlation, both axes.** Increments are correlated *along* the boundary (a breakthrough
  moves a sector, not a metre of it) and *in time* (advance rate is autoregressive at a scale of
  weeks). Independent increments would give far too narrow a first-passage distribution.
- **Anisotropy and geometry.** The blob grows fast along road axes and stalls at rivers and
  built-up edges. And a point can be swallowed by **envelopment** from a flank without the
  nearest-boundary distance ever shrinking monotonically — so "distance to the front" is a
  state variable, not the state.
- **Layer identity.** The market's fine print makes the resolving event passage of a *particular
  union of three layers*. The excluded Infiltration layer routinely reaches a town well ahead of
  the qualifying ones — CNN's 24 June report on Kostyantynivka is literally about infiltration
  being mistaken for control. A model that simulates "did the Russians get there" rather than
  "which ISW layer got there" is wrong in a way that is invisible until it loses money.
- **Ceasefire as a competing risk.** A ceasefire freezes the process. Every leg is therefore
  `P(passage before min(deadline, ceasefire))`. The ceasefire hazard is not something we have to
  guess: `russia-x-ukraine-ceasefire-agreement-by` is a **$5.69M board with 1c spreads** quoting
  by-31-Aug **0.08**, by-31-Oct **0.185**, by-31-Dec **0.355** ($297k / $801k / $2.06M volume,
  $49k–$125k liquidity). That is a free, deep, sharply-priced anchor for the competing risk, and
  it is almost certainly not in the price of a $3k settlement board.

### What the Rust actually computes

1. **Ingest.** Poll ISW's ArcGIS feature services daily (`f=geojson`), snapshot to R2 via
   `tools/r2data/`, exactly as the strategy folders already do for price data:
   - `services5.arcgis.com/SaBe5HMtmnbqSWlu/.../VIEW_RussiaCoTinUkraine_V3/FeatureServer/49`
     — "Assessed Russian-controlled Ukrainian Territory", 10 polygons.
   - `.../AssessedRussianAdvanceInUkraine_V2_view/FeatureServer/0` — 65 polygons, `lastEditDate`
     2026-07-25 (it moves daily).
   - `.../View_AssessedRussianInfiltrationAreasinUkraine_V4/...` — the layer the fine print
     **excludes**; we carry it because the gap between it and the qualifying union is a covariate.
   - `.../Ukrainian_Settlements_Updated_view/FeatureServer/0` — **29,655 settlement polygons**
     with EN/UA names and oblast/raion/hromada, i.e. the full gazetteer.
   - `.../Ukrainian Trunk Primary and Secondary Roads` — the advance-axis covariate, published by
     the same ISW account.
2. **State vector, per board, per day.** Signed distance from the resolving point to each of the
   three layers; envelopment fraction (share of a 5 km disc around the point already inside the
   qualifying union); local advance rate of the qualifying boundary within 15 km over 7/30/90-day
   windows; distance to nearest trunk road; built-up area of the containing settlement polygon.
3. **Fit.** Discrete-time hazard with time-varying covariates on the historical panel: every
   settlement polygon ISW has ever shaded, outcome = first day the qualifying union covered it.
   The nuisance parameters that matter are the *correlation lengths*, not the mean rate.
4. **Simulate.** Propagate `∂Q_t` forward day by day: per-segment increments drawn from a fitted
   zero-inflated heavy-tailed law, correlated along the boundary and autoregressive in time;
   overlay a ceasefire arrival drawn from the hazard implied by the $5.69M ladder. Per path,
   record which of the ~116 open resolving points are covered before each deadline.
   **10⁵ paths × ~250 days × a few thousand boundary segments** — trivial in Rust, painful
   anywhere else, and the reason this is our shape of problem.
5. **Output.** A *joint* distribution over all open legs at once. That automatically enforces
   monotonicity in deadline, cross-settlement coherence, and consistency with the ceasefire
   anchor — three constraints the crowd cannot enforce because it prices each board separately.

### Why a closed form will not do — with the numbers

The analytic answer everyone reaches for is a **constant-hazard (exponential) first passage**:
`P(by T) = 1 − exp(−λT)` with `λ` from distance ÷ advance rate. The market itself refutes it.
Kostyantynivka's own ladder, live today:

| deadline | price | conditional hazard for that interval | implied per-day hazard |
| --- | --- | --- | --- |
| 31 Jul (5 d) | 0.0905 | — | — |
| 31 Aug | 0.510 | 0.461 | 0.0199 |
| 30 Sep | 0.765 | 0.520 | 0.0245 |
| 31 Dec | 0.895 | 0.553 (over 92 d) | 0.0088 |

The implied per-day hazard **falls 2.5×** between Q3 and Q4. A constant-hazard model cannot
produce that shape at all. The ceasefire term structure explains only a small part of it (Q4
ceasefire probability is 0.209, which shortens the effective window by ~10%, not by 60%). The
rest is a real claim about siege dynamics — that the rate decays once the blob reaches a town —
and it is precisely the claim a simulation fitted on the historical record can check and the
crowd can only feel.

**The cross-section says the same thing louder.** Q4 conditional hazards implied by today's
Sep-30/Dec-31 pairs range from **0.011** (Prymorske) to **0.641** (Bilytske); the ratio of the
Q4 hazard to the remaining-Q3 hazard runs from **0.43** (Prymorske) to about **39×**
(Havrylivka: 0.35c by 30 Sep, 14c by 31 Dec). These boards are all driven by one front and one
ceasefire. Some of that dispersion is genuine local geometry — that is what the simulation is
*for* — and some of it is fifty separate crowds pricing fifty boards off fifty news cycles.

And a concrete illustration that distance alone is not the answer, measured from the live ISW
service this morning (nearest qualifying polygon, bracketed by successive buffer queries):

| settlement | dist. to Assessed Advance | dist. to Assessed Control | market |
| --- | --- | --- | --- |
| Huliaipole | **< 250 m** | < 8 km | capture-all by 30 Sep **0.665** |
| Kostyantynivka | < 1 km | < 16 km | rail stn by 30 Sep **0.765** |
| Kupiansk | < 4 km | < 8 km | by 30 Sep 0.09 / by 31 Dec 0.295 |
| Orikhiv | < 8 km | < 16 km | by 30 Sep 0.05 / by 31 Dec 0.10 |
| Dobropillia | < 16 km | < 32 km | enter by 31 Dec **0.65** |
| Druzhkivka | < 16 km | < 32 km | enter by 31 Dec **0.285** |
| Kramatorsk | < 32 km | < 32 km | enter by 31 Dec 0.20 |
| Sloviansk | < 32 km | < 32 km | enter by 31 Dec 0.155 |

The ordering is broadly sensible, which is reassuring about the pipeline. But **Dobropillia and
Druzhkivka sit in the same distance bucket and trade 0.65 against 0.285** — a 2.3× difference
that is either real local geometry (axis of advance, defensive lines, roads) or two crowds
disagreeing. Resolving that question is a GIS-and-Monte-Carlo job, and it is the job.

### Kinship with slot 1, stated deliberately

`barrier-touch/ladder-rv` is also a first-passage strategy — "does the price touch $B before
Friday". This is the version where **the closed form does not exist**: no `2·N(−|ln(B/S)|/σ√τ)`,
no options-IV anchor, no analytic touch probability. Same intuition, different family, different
data, and the one place where Monte Carlo is not a convenience but the only route.

---

## 3. The data source, and why it is not already in the price

The ISW map is public and a trader can glance at it. **We are not claiming a data-source edge on
where the front is.** We are claiming three things the glance does not give:

1. **The metric.** "How close is the front to the resolving pixel" is a number, not an
   impression, and it has to be computed against three specific vector layers with an exclusion
   rule. The endpoints above are not linked from the storymap UI; I found them by walking the
   ArcGIS Online item graph from the storymap's owner account.
2. **The gazetteer.** 29,655 settlement polygons turn "which towns has ISW ever shaded, and when"
   into a survival dataset with thousands of first-passage observations. That is the ordering-
   over-many-public-objects case that `wiki/market-selection.md` says is where our advantage
   lives: hidden from the amateur *too*, because it must be assembled rather than looked at.
3. **The competing risk.** Reading the $5.69M ceasefire ladder into every settlement board is
   free, mechanical, and almost certainly not being done on a $3k board.

This is deliberately a **market-specific** source: ISW's Ukraine layers price nothing else on
Polymarket. A method built on them cannot be pointed at another family, which is exactly the
second standing directive.

---

## 4. The screens, cheapest first

### 4.1 Sharp incumbent — **passes, and this is the reason to file it**

- **No bookmaker prices this.** Regulated books do not take bets on territorial capture; searching
  produced no line anywhere. The one apparent third-party "odds" page (`lines.com`) is a
  Polymarket mirror, not an independent price.
- **No institution publishes the simulation.** CSIS publishes *descriptive* advance rates (the
  ~50 m/day figure). ISW publishes maps and prose, and — see §6 — **objects to its map being used
  for betting at all**, which is roughly the opposite of DataGolf shipping its Monte Carlo in the
  page source. A direct search for a probabilistic capture-timing model returned nothing.
- **The one tool that exists is a visualiser, not a model.** `PolyGlobe` (the pseudonymous
  "Pentagon Pizza Watch" team) renders Polymarket war contracts on a 3-D globe; it briefly wired
  in DeepState's API without permission, was asked to stop, and relaunched on another open feed.
  It displays the market's own prices. It does not produce any.

Per `wiki/reference/sharp-line-screen.md`, the absence of a professional counterparty removes our
cheapest check, so the remaining gates carry the load. Note also that tooling is now being built
around this family in public — the window is not guaranteed to stay open.

### 4.2 Speed race — **passes, measured**

Three independent measurements on the 395 resolved legs with ≥$1k volume (100% returned non-empty
CLOB history at `fidelity=60` with explicit `startTs`):

| measurement | value | contrast |
| --- | --- | --- |
| largest single 1-hour price step, as share of the leg's total variation | **median 6%**, p75 10%, p90 17% | quake mid-window: 70% in the first hour |
| legs where any single hour carries >50% of total variation | **0 / 395** | — |
| share of total variation inside the final 48 h (the resolution collapse, excluded from the claim) | 14.8% | — |

And the decisive one — **how fast does the mispricing itself close?** Same 197 legs, all three
checkpoints:

| checkpoint | mean price | gap (realised − price) |
| --- | --- | --- |
| **T − 30 d** | 0.225 | **−14.9 pp** |
| T − 14 d | 0.113 | −3.7 pp |
| T − 7 d | 0.108 | −3.2 pp |

The error is corrected over roughly **two weeks**, not two minutes. A daily-cadence agent
entering at T−30d captures essentially all of it. This is the opposite of the temp-truncation
kill (dead legs collapsed in 0–3 min) and of the quake mid-week trade (70% in the first hour).

### 4.3 Phantom midpoints — **passes with a real 6% tail that must be gated**

Per `wiki/reference/phantom-midpoints.md`, on all 395 resolved legs, splitting by whether the
price ever moved before the final 48 h:

| book state | n | share | contrast |
| --- | --- | --- | --- |
| **DEAD** (never moved) | 24 | **6.1%** | esports 23%, tennis 8.5%, quakes 0% |
| near-flat (total variation < 2c) | 0 | 0.0% | |
| **LIVE** | 371 | **93.9%** | median pre-close total variation **3.128** |

The dead 6.1% is not random noise: it is dominated by the Pokrovsk and Myrnohrad boards, i.e.
legs that got pinned at 0 or 1 once the outcome was effectively settled. Every measurement below
is on LIVE legs only.

A live phantom to keep in the gate as a worked example: **`will-russia-capture-shakhove-by-december-15`**
quotes **bid 0.05 / ask 0.92 on $0 of volume**, and Gamma dutifully reports `outcomePrices` of
**0.485**. That is the artifact, alive on this family today. Gate at spread ≤5c and real depth or
it will manufacture edge.

### 4.4 Checkpoint artifact and null model — **passes, and it fired on my own first cut**

This gate earned its place today. My first pass read "the price 30 days before the deadline" as
*the first history point at or after T−30d*, which for a board listed 20 days before its deadline
is the **creation price**. That version reported a −14.5pp gap and, decisively,
`wiki/reference/checkpoint-artifact.md`'s alarm went off: **the base-rate null beat the market**
(Brier 0.1322 vs 0.1486). Requiring the leg's price history to *predate* the checkpoint by 2+ days
fixes it:

| checkpoint | n | mean price | realised | gap | Brier (market) | Brier (base-rate null) |
| --- | --- | --- | --- | --- | --- | --- |
| **T − 30 d** | 229 | 0.239 | 0.140 | **−9.95 pp** | **0.1049** | 0.1202 |
| T − 14 d | 281 | 0.161 | 0.121 | −4.0 pp | 0.0709 | 0.1064 |
| T − 7 d | 288 | 0.149 | 0.115 | −3.5 pp | 0.0432 | 0.1015 |

**The market beats the null at every checkpoint**, so we are measuring a priced board, not an
unlisted one. The residual −9.95pp is the thing to explain.

The mutually-exclusive leg-sum test does not apply (a deadline ladder is monotone, not a
partition). Its analogue — monotonicity in deadline — holds on every open ladder I checked.

### 4.5 Is the −9.95pp real? The decompositions the wiki demands

**Cluster-robust, clustering on settlement** (106 clusters, 229 legs): gap **−9.95pp, se 2.31pp,
t = −4.31**.

By **ex-ante book activity** (total variation *before* the checkpoint — an honest liveness measure
that cannot be contaminated by the outcome):

| pre-checkpoint activity | n | mean price | realised | gap | se | t |
| --- | --- | --- | --- | --- | --- | --- |
| very active (>150c TV) | 157 | 0.254 | 0.146 | **−10.7 pp** | 2.5 | **−4.24** |
| active (50–150c) | 60 | 0.207 | 0.100 | −10.7 pp | 3.7 | −2.90 |
| mid (10–50c) | 12 | 0.208 | 0.250 | +4.2 pp | 7.2 | +0.58 |

**It does not attenuate on the liveliest books** — which is what the phantom page says to look
for, and the opposite of the tennis and esports kills.

One caveat I want on the record because it looks bad until you see why. Splitting by *lifetime*
volume does attenuate: >$200k legs give −5.1pp (se 5.6, t=−0.91) against −16.5pp on $10–50k legs.
**Lifetime volume is outcome-endogenous** — a leg that resolves Yes attracts a burst of trading on
the way — so that split mechanically manufactures "low volume → low Yes rate". The ex-ante
activity split above is the same question asked without the contamination, and it is flat. The
trial must nonetheless redo the split on **volume as of the checkpoint**, reconstructed from the
Data API tape. If the effect really is confined to thin legs, the fundable version is much smaller.

By sub-family: `enter X` −12.8pp (n=93), `capture all of X` −12.5pp (n=36), `capture X` −6.2pp
(n=90), `Ukraine re-enter X` −8.0pp (n=10). By entry price: <10c −3.3pp, 10–25c −10.7pp, 25–50c
−12.1pp, 50–75c −18.2pp, >75c −26.0pp — **the bias grows with the price**, so it is a directional
over-pricing of Yes, not the classic favourite-longshot U-shape, and the fundable band is where
it is biggest.

**The trading number.** Selling Yes at T−30d on live-book legs priced 10–90c, held to resolution,
with a flat **2c haircut** for spread and adverse selection and **zero fee** (§4.6):

> **n = 149, mean entry 0.311, realised Yes rate 0.174, +11.67 c/share, se 3.02, t = +3.87,
> win rate 82.6%.** At a pessimistic 5c haircut: **+8.67 c/share, t = +2.87.**

**The counter-evidence, which is the most important table in this file.** By resolution month,
the gap is negative in 12 of 15 months (9 of 11 months with n≥3). But:

| month | n | mean price | realised | gap |
| --- | --- | --- | --- | --- |
| 2026-03 | 33 | 0.217 | 0.061 | −15.7 pp |
| 2026-04 | 39 | 0.248 | 0.103 | −14.6 pp |
| 2026-05 | 52 | 0.208 | 0.115 | −9.3 pp |
| 2026-06 | 35 | 0.211 | 0.086 | −12.6 pp |
| **2026-07** | **4** | **0.203** | **1.000** | **+79.8 pp** |

**Every one of this month's four resolved legs went Yes** — Rodynske-again, Pokrovka,
Krasnoiarske, Vasylivka, entered at 8.5c–40c. That is what a breakthrough month does to a
systematic Yes-fade, and it is the current month. n=4 proves nothing on its own; it illustrates
exactly the tail the strategy is short. Leave-one-settlement-out is stable (−9.0pp to −10.5pp),
so no single town drives the headline — but no amount of cross-sectional robustness fixes the
fact that **all 229 observations share one war**. The honest unit of independence is the month,
there are 15 of them, and the effective sample is that small.

### 4.6 Midpoint is not a fill — **the screen this firm now lives on, and the reason to file this one**

Fees first, because they are unusual: `feesEnabled: false`, `feeSchedule: null`, and
`clob.polymarket.com/fee-rate` returns `{"base_fee": 0}`. **Geopolitics is genuinely fee-free**,
so the entire cost stack is spread — no 1.25c/share tax in the middle of the band.

Live books, read from the CLOB this morning (best bid/ask are the *last* array elements):

| leg | bid / ask | spread | bid depth ≤5c | ask depth ≤5c |
| --- | --- | --- | --- | --- |
| `will-russia-capture-kostyantynivka-by-september-30-256-333` | **0.76 / 0.77** | **1c** | **$5,149** | $2,982 |
| `will-russia-capture-kostyantynivka-by-august-31-1` | **0.50 / 0.51** | **1c** | **$4,719** | $765 |

Across the whole open family: **116 open legs, 92 in the 5–95c band, 77 of those with spread
≤5c.**

And the number the brief actually asks for — **realised taker flow, folded to Yes-equivalent
units the way `tools/fillcheck/src/main.rs` does it** (a taker who sells Yes, or buys No, proves a
resting *bid* existed at that price; that is the side we would take):

| leg | trades | taker notional | distinct wallets | top wallet | **proves-BID $** | proves-ASK $ | last 30 d |
| --- | --- | --- | --- | --- | --- | --- | --- |
| kostyantynivka-by-september-30 | 2,060 | $328,709 | 539 | 11% | **$183,731** | $144,977 | $266,673 |
| kostyantynivka-by-august-31 | 575 | $42,786 | 228 | 11% | **$25,911** | $16,874 | $42,786 |
| lyman-by-december-31 | 3,357 | $238,989 | 711 | 28% | **$153,111** | $85,877 | $62,495 |
| all-of-lyman-by-december-31 | 1,173 | $94,153 | 396 | 21% | **$79,214** | $14,939 | $7,568 |
| enter-dopropillia-by-december-31 | 1,403 | $62,779 | 254 | 28% | **$38,409** | $24,369 | $51,539 |
| enter-druzkhivka-by-december-31 | 562 | $29,784 | 231 | 13% | **$16,892** | $12,892 | $14,956 |
| lyman-by-september-30 | 372 | $9,449 | 151 | 10% | **$5,574** | $3,875 | $7,751 |

Read the shape of that table against our own ledger. Our first 21 scored predictions beat the
market 21/21 and had a counterparty at our price **2 of 21 times**; `will-spy-reach-760` was
scored at a 2.55c midpoint against a best-ever bid of 0.12c. Here, **on every single leg, the
bid side — the side the thesis says to take — carries more notional than the ask side**, across
150–700 distinct wallets with top-wallet concentration of 10–28% (well under the 47%/74% that
flagged wash trading on VKTX). This is the first candidate this firm has produced where the
liquidity is on the same side as the claimed edge.

---

## 5. Example markets — real numbers, 2026-07-26 ~01:40 UTC

**1. `will-russia-capture-kostyantynivka-by-august-31-1`** — bid **0.50** / ask **0.51**, 1c
spread, $4,719 of bid depth within 5c, $66,609 volume, $26,594 liquidity, resolves 31 Aug.
575 trades / 228 wallets / $42.8k notional, all of it inside 30 days. **The fundable-band trade.**

**2. `will-russia-capture-kostyantynivka-by-september-30-256-333`** — bid **0.76** / ask **0.77**,
1c spread, $5,149 bid depth within 5c, $414,048 volume, $42,935 liquidity, resolves 30 Sep. The
deep leg of the ladder and the anchor for the term structure. Resolving point: the railway station
on Pravoberezhna vulytsia; measured **<1 km** from ISW's Assessed Advance layer today.

**3. `will-russia-enter-dopropillia-by-december-31-2026`** — 0.64 / 0.66, $113,381 volume,
$37,830 liquidity — versus **4. `will-russia-enter-druzkhivka-by-december-31-2026`** — 0.28 / 0.29,
$96,362 volume, $14,552 liquidity. Same distance bucket from the front (<16 km), 2.3× apart in
price. Both are legs of `which-cities-will-russia-enter-by-december-31` ($597k, 8 cities:
Dopropillia 0.65, Druzkhivka 0.285, Kramatorsk 0.20, Sloviansk 0.155, Sumy 0.075, Kherson 0.075,
Kharkiv 0.03, Zaporizhia 0.023) — a **single event that is a cross-sectional joint-simulation
board on its own**.

**5. The anchor: `russia-x-ukraine-ceasefire-agreement-by`** — $5,694,174 volume; by 31 Aug 0.08
($296,735 / $49,366 liq), by 31 Oct 0.185 ($800,815 / $79,823), by 31 Dec 0.355 ($2,059,290 /
$125,003). 1c spreads throughout.

**6. The phantom, kept as a gate test: `will-russia-capture-shakhove-by-december-15`** — bid 0.05 /
ask 0.92, **$0 volume**, reported price 0.485.

---

## 6. Backtest supply — concrete

Harvested from Gamma tags **102486 (`Ukraine Map`)** and **102475 (`Russia Capture`)**, open and
closed:

- **159 distinct events, 523 legs, $72,388,120 of lifetime volume.**
- **407 resolved legs across 144 events and ~105 distinct settlements**, plus **116 open legs**.
- Per-leg CLOB history: **395/395 non-empty** for legs ≥$1k volume, at `fidelity=60` with an
  explicit `startTs` (per the wiki gotcha — `interval=max` silently caps at 30 days).
- **Resolution cadence is monthly and continuous**: legs land on the last day of every month
  across dozens of settlements, so a 10-day trial scores real forward instances at the 31 Jul and
  31 Aug boundaries, with ~10–40 legs resolving per month.
- The physics sample is far larger than the market sample: 29,655 settlement polygons × ~3.5 years
  of ISW map history, if that history can be reconstructed — which is Gate 0 and the single
  biggest execution risk in this file.

Two API notes for whoever picks this up. **`/markets?slug=` returned an empty result for
`will-russia-capture-shakhove-by-december-15` even though the market is open** — the event query
(`/events?slug=<event>`) returns it fine. Add that to the wiki's existing `&closed=true` gotcha:
a slug lookup coming back empty means "ask the event", not "the market does not exist". And
**Gamma's `endDate` on this family is unreliable** — `will-russia-capture-lyman-in-2025` carries
`endDate 2025-12-31` while trading, and the Shakhove December legs carry `2025-11-30`. Parse the
deadline from `groupItemTitle` and the description, never from `endDate`.

---

## 7. Falsification sketch — numeric kill thresholds, cheapest first

**Gate 0 — can we reconstruct historical ISW geometry at all? (hours, do this first).**
The live feature services expose only the *current* state. Routes to test, in order: (a) ISW's own
monthly snapshot services (`May_CoT`, `Assessed Russian Advances in May 2026`, the timelapse
layers) — check depth and granularity; (b) the services advertise `Query,Sync,ChangeTracking`, so
a sync-enabled replica may expose edit deltas; (c) web-archive / third-party mirrors of the
geojson; (d) DeepStateMap (`/api/history/last` is open and returns 624 KB of current front-line
geojson; `/api/history` now returns 401, and arbitrary timestamps 404 — the index is the blocker).
> **KILL if** we cannot reconstruct **≥12 months of at-least-weekly qualifying-union geometry**.
> Without it there is no physics backtest and the idea collapses to the naive fade in §4.5, which
> is not worth a slot on its own.

**Gate 1 — does the naive fade survive an independent re-run?** Re-derive §4.5 from a clean code
path: live books only, checkpoint strictly inside the leg's quoted life, split by volume **as of
the checkpoint** (from the Data API tape, not lifetime), cluster on settlement, block-bootstrap on
month.
> **KILL if** the live-book gap is **< 5pp**, or if it sign-flips between the pre-Apr-2026 and
> post-Apr-2026 halves, or if it disappears once checkpoint-time volume replaces lifetime volume.

**Gate 2 — does the simulation beat the naive fade?** The null is not Poisson here; it is
"**sell every live Yes leg at T−30d**", which already earns +11.67c/share. Leave-one-month-out.
> **KILL if** the simulation does not beat the flat fade by **≥0.03 log-loss** out of sample, or
> if it fails to identify the July-2026 breakthrough legs (Rodynske, Pokrovka, Krasnoiarske,
> Vasylivka) as *higher* hazard than the fade would. If it cannot tell a stalling siege from a
> collapsing flank, it is a bias-harvester wearing a GIS costume, and it should be filed as such.

**Gate 3 — net of spread and delay.** Trade legs with spread ≤5c, top-of-book ≥$100, price 5–90c;
fill at bid/ask, **fee = 0** (verified), signal frozen at the daily run, **fill at t+24h with 2c
adverse**, hold to resolution.
> **KILL if** net < **3c/share**, or if the sign flips across sample halves.

**Gate 4 — the artifact hunt.** Re-run the phantom split (6.1% dead measured) and the
liquidity decomposition on the full sample via an independent path.
> **KILL if** the edge concentrates in legs under $500 of checkpoint-time volume, or inverts with
> book liveness.

**Gate 5 — adjudication and map-tamper hazard. This is the one I cannot price.**
On the night of **15–16 November 2025** an unauthorised edit to ISW's map showed Russian control
of a key intersection in **Myrnohrad**. A Polymarket board with **>$1.3M of volume** resolved
**Yes** on it, paying reported returns up to 33,000×. The edit vanished by the next morning. ISW
removed the geospatial researcher believed responsible. **Polymarket did not reverse the payouts.**
ISW has publicly objected to its maps being used for betting at all; Ukraine has since moved
against Polymarket over these markets.
This is `wiki/reference/venue-resolution-epsilon.md` in a far worse costume: not "the venue's feed
differs from ours at the margin", but "the feed can be *edited*, and the venue honours the edit".
The hazard is **one-directional against the No side — the side this thesis takes.** It also means
part of the measured −9.95pp is *fair compensation* for tamper risk rather than a harvestable
bias, and that part is not ours.
> **Measure it:** over the 407 resolved legs, count Yes resolutions whose qualifying shading later
> disappeared. **KILL if** the tamper-attributable Yes rate exceeds ~2% of resolved legs, or if it
> cannot be measured at all because Gate 0 failed. Otherwise carry it as an explicit haircut on
> every short-Yes position and cap concentration per settlement.

**Gate 6 — regime.** Everything here is one war in one 15-month regime with a broadly stalling
front. All 229 observations load on a single latent variable: the pace of the Russian advance.
> **Pre-registered live stop:** if the first two calendar months of forward legs show a positive
> (against us) monthly gap, stop regardless of the backtest. And size on the assumption that
> **every open position is one trade**, because in a breakthrough they are.

---

## 8. What I would tell the CEO in one paragraph

This is the first family I have found where the liquidity is on the same side as the claimed
edge: 1c spreads, $4.7–5.1k of resting bid depth, hundreds of distinct wallets, **zero taker
fee**, and 92 open legs in the fundable band — against our own ledger's 2-of-21 reachable. The
resolution source is a public GIS layer we can query today, and no bookmaker or institution
prices the object. The measured crowd error is large (−9.95pp at T−30d, cluster-robust t=−4.31),
decays over two weeks rather than two minutes, and survives the phantom, checkpoint-artifact and
book-liveness gates. **Three things could kill it:** we may not be able to reconstruct historical
map geometry (Gate 0, testable in an afternoon); the simulation may add nothing over a flat fade
(Gate 2); and the resolution source has a documented, unreversed tampering incident that runs
against the side we would take (Gate 5). Run Gate 0 first — if it fails, this is a one-day write-up
and not a slot.
