---
date: 2026-07-25
slug: quake-ladder-overdispersion
status: discarded-idea # killed day 1: market already prices the overdispersion (implied Fano 1.362 vs empirical 1.358); the signal was a fresh-board checkpoint artifact        # backlog | trialing | discarded-idea | promoted
example_markets: ["how-many-5pt5-or-above-earthquakes-july-20-july-26-20260718184830018", "how-many-6pt5-or-above-earthquakes-july-20-july-26-20260718184921053", "how-many-6pt5-or-above-earthquakes-july-27-august-2-20260723165219436"]
model: opus-5 (xhigh)
summary: >-
  Weekly USGS earthquake-count bucket ladders are priced as if seismicity were Poisson;
  it is a self-exciting branching process (Fano 5.06 vs 1.00 over 1,385 weeks), so both
  tail buckets are ~1.6x too cheap and the middle up to 1.6x too rich. Shape claim, traded
  once at window-open only. Survives because no bookmaker prices it and nobody publishes
  the distribution.
---

# Weekly seismicity count ladders: sell the middle, buy both tails

**Level or shape? SHAPE, and unusually purely so.** The crowd's *centre* is right — on 22
resolved M6.5+ boards the mean winning-leg price at window-open is **0.364 against a
Herfindahl benchmark of 0.366**, i.e. calibrated on the favourite by the wiki's own test. We
do not claim to forecast next week's earthquake count better than they do. We claim their
**bucket allocation** is wrong, because earthquakes are a self-exciting branching process and
the ladder is priced as if they were Poisson.

**Why this one survived when three richer-looking candidates died today** (see
`ideas/2026-07-25-tennis-games-ladder-discarded.md`): it is the only family I found where
**no bookmaker prices the object and no institution publishes the distribution**. Pinnacle
killed the tennis idea in ten minutes; DataGolf killed golf; FanGraphs kills MLB. Nobody
anywhere prices "how many M6.5+ earthquakes worldwide next week". The wiki's new
`sharp-line-screen.md` says the absence of a professional counterparty is one of the few good
reasons to expect an edge to survive — this is that case, and the price of it is that the
remaining gates must carry more weight.

---

## 1. The simulation

**Process — ETAS (Epidemic-Type Aftershock Sequence), a self-exciting branching point process.**
Conditional intensity

```
λ(t | H_t) = μ  +  Σ_{i: t_i < t}  K · 10^(α(M_i − M_0)) · (t − t_i + c)^(−p)
```

with magnitudes drawn from Gutenberg–Richter (`P(M ≥ m) ∝ 10^(−b·m)`, b ≈ 1.0).

**State.** The full catalogue history at window-open: every event's `(time, magnitude)` for
the preceding ~90 days, plus the branching tree's live offspring intensity.

**Simulation.** Thin the background process, then for each event recursively spawn offspring
(Poisson-many, productivity `K·10^(α(M−M₀))`, times from the Omori kernel, magnitudes from
G–R), until the 7-day window closes. Count events ≥ threshold. **10⁶ simulated weeks per
board** (cheap in Rust — each week is a few thousand branching draws; the expensive part is
the parameter posterior, below).

**Two layers on top, both of which matter more than the ETAS core:**

- **Magnitude-revision noise (§3).** The market resolves on a *threshold count of a
  discretised, revised measurement*. Simulate each event's reported magnitude as
  `M_true + ε`, with ε calibrated from USGS's own version history. This is not a refinement;
  it is worth ±1–2 whole buckets.
- **Parameter posterior.** `(μ, K, α, c, p, b)` fitted by MLE on the declustered catalogue,
  then integrated over — not plugged in. ~2×10³ posterior draws × 10³ weeks each.

**Inputs — free, public, machine-readable, no key.**
USGS FDSN: `earthquake.usgs.gov/fdsnws/event/1/query?format=csv&starttime=…&minmagnitude=…`
(20,000-event cap per query, so chunk by 3-year windows). I pulled **47,534 M5.0+ events,
2000–2026** in nine parallel calls. Version history per event via
`earthquakes.usgs.gov/earthquakes/feed/v1.0/detail/<id>.geojson` → `properties.products.origin[]`,
each carrying `updateTime` and its magnitude (one event I checked went **5.2 → 5.4 → 5.3**).

### Why the closed form is wrong — with the number

The analytic answer everyone reaches for is **Poisson with λ = the historical weekly mean**.
Over **1,385 weeks** (2000-01-03 → 2026-07-20), aligned to the market's own Mon 00:00 ET
window:

| threshold | mean | variance | **Fano (var/mean)** | sd | Poisson sd | width ratio |
| --- | --- | --- | --- | --- | --- | --- |
| **M5.5+** | 9.458 | 47.825 | **5.06** | 6.92 | 3.08 | **2.25×** |
| **M6.5+** | 0.872 | 1.313 | **1.51** | 1.15 | 0.93 | 1.23× |

Poisson would give Fano = 1.00. The M5.5+ count distribution is **2.25× wider than Poisson**,
and the error is not a wash across the ladder — it is a clean U-shape that maps directly onto
the traded buckets:

| bucket | Poisson | empirical (1,385 wks) | ratio |
| --- | --- | --- | --- |
| ≤6 | 0.1682 | 0.2830 | **1.68×** |
| 7 | 0.1049 | 0.1206 | 1.15× |
| 8 | 0.1240 | 0.1119 | 0.90× |
| 9 | 0.1303 | 0.1004 | 0.77× |
| **10** | 0.1232 | 0.0773 | **0.63×** |
| 11 | 0.1059 | 0.0715 | 0.67× |
| 12 | 0.0835 | 0.0621 | 0.74× |
| 13 | 0.0607 | 0.0534 | 0.88× |
| 14 | 0.0410 | 0.0274 | 0.67× |
| **>14** | 0.0582 | 0.0924 | **1.59×** |

**Both tails ~1.6× too cheap under Poisson, the middle up to 1.6× too rich.** Sell the modal
buckets, buy both wings. That is the trade, and it is a shape claim by construction.

There is **no closed form for the count distribution of a branching process with an Omori
kernel** — its probability generating function satisfies an implicit integral equation with
no elementary solution. Monte Carlo is not a convenience here, it is the only route. And the
empirical marginal is *not* a substitute for the simulation, because it cannot condition on
the state (below).

**The clustering that produces the overdispersion**, measured directly:

| trigger | n | mean M5.5+ in next 1d | vs baseline (1.352/d) | 3d | 7d |
| --- | --- | --- | --- | --- | --- |
| after any M6.5+ | 1,208 | 3.627 | **2.68×** | 1.80× | 1.41× |
| after any M7.0+ | 396 | 5.104 | **3.77×** | 2.33× | 1.62× |

---

## 2. The screens

### Sharp-line screen — **passes by construction, and this is the whole reason to file it.**
No bookmaker or exchange prices global weekly earthquake counts. I verified Pinnacle's sport
list from this box (`guest.api.arcadia.pinnacle.com/0.1/sports`): 33 Tennis, 32 Table Tennis,
37 Padel — there is no seismicity market anywhere on it, nor on any betting exchange. Per
`wiki/reference/sharp-line-screen.md` this removes our cheapest check, so gates 0–4 below
carry the load.

### Incumbent — "who is already simulating this, and with what?"
- The USGS publishes **operational aftershock forecasts** — but only per significant
  mainshock, as expected counts over 1d/7d/30d in a local region. **Nobody aggregates them
  into a distribution for the global weekly count**, which is the resolution variable.
- Academic ETAS is a mature literature; there is no free live product.
- Our inputs are the same public catalogue everyone can read, so this is not a data-source
  edge. It is a compute-and-care edge on an object no one has bothered to price. That is
  precisely the brief.

### Phantom-midpoint gate (`wiki/reference/phantom-midpoints.md`) — **passes outright, 0%.**
This is the screen that killed my tennis candidate three hours ago, so I ran it here before
writing anything. Across **314 resolved quake legs** with a usable pre-resolution series:

| book state | count | share | for contrast |
| --- | --- | --- | --- |
| DEAD (price never moved) | **0** | **0.0%** | tennis match-totals 8.5%; esports handicaps 23% |
| near-flat (total variation <2c) | 0 | 0.0% | |
| **LIVE (total variation ≥2c)** | **314** | **100.0%** | median total variation **1.79** per leg |

Every leg on every resolved board has a live, moving, two-sided book. There is no phantom
mass to manufacture an artifact out of.

### Book quality — **passes.** Live board, pulled 2026-07-25:

`how-many-5pt5-or-above-earthquakes-july-20-july-26` — 8 legs, **negRisk**, event volume $19,934:

| leg | bid / ask | spread | liquidity | volume |
| --- | --- | --- | --- | --- |
| ≤8 | 0.11 / 0.15 | 4.0c | $2,560 | $13,250 |
| 9 | 0.23 / 0.24 | **1.0c** | $2,372 | $1,021 |
| 10 | 0.24 / 0.26 | 2.0c | $3,333 | $1,383 |
| 11 | 0.15 / 0.18 | 3.0c | $2,356 | $855 |
| 12 | 0.125 / 0.174 | 4.9c | $4,420 | $1,079 |
| 13 | 0.071 / 0.088 | 1.7c | $2,580 | $840 |
| 14 | 0.03 / 0.05 | 2.0c | $2,417 | $702 |
| >14 | 0.03 / 0.053 | 2.3c | $2,474 | $802 |

**All eight legs clear the ≤5c gate** with $2.3–4.4k of resting liquidity each. The M6.5+
board is thinner ($5,369 total) and its deep-tail legs (`4`, `5`, `>5`) quote 0.1–0.7c — those
are sub-3c legs and are diagnostics, not trades, exactly as in `ladder-rv`.

### Fee viability — **binding, and it selects the trade for us.**
`feeSchedule = {rate: 0.05, exponent: 1, takerOnly: true, rebateRate: 0.25}` (Science/Weather
tag). `fee = shares × 0.05 × p(1−p)` → **1.25c/share at p=0.50, 1.20c at 0.40, 0.68c at 0.10,
0.45c at 0.05.** With a 2–4c spread, round taker cost is **2.5–3.5c** in the modal band but
only **~1.2–1.5c** on the wing legs at 3–10c. Since the model's edge is *largest on the
wings* (1.6× mispricing) and the fee is *smallest* there, fees push us toward exactly the legs
the thesis wants. Note the **rebateRate is 0.25 here versus 0.15 on sports** — resting maker
orders are unusually well paid on this family and should be modelled separately.

### Speed race — **passes only for window-open entries, and the idea is scoped to that.**
Measured, and it is the reason the obvious mid-week trade is excluded: after a qualifying
M6.5+ lands inside a live window, the `0` leg moves **−0.279 within one hour** against a total
move of −0.398 by +24h — **70% of the repricing happens in the first hour** (n=19). That is
the `temp-truncation/runningmax` kill pattern and an agent on a daily cadence cannot compete.

**So the strategy trades once, at window-open (Monday, before any qualifying event exists),
and holds to resolution.** At that instant there is no realised state to race, no public print
pending, and the mispricing is purely model-revealed — the same structure as
`barrier-touch/ladder-rv`. Mid-window entries are forbidden by construction, and re-entry
after a large event is explicitly out of scope.

### Glanceable within-window state — **not applicable at the entry point.**
The running count is glanceable via USGS (that is part of why mid-window is dead), but at
window-open the count is zero for everybody. What is *not* glanceable, ever, is the
distribution — and that is what the ladder resolves on.

---

## 3. Am I reading the exact object the market resolves on?

Every board's description names the source and window explicitly, e.g.:

> *"the total number of earthquakes with a magnitude of 5.5 or higher that occur anywhere on
> Earth between July 20, 2026, 12:00 AM ET, and July 26, 2026, 11:59 PM ET. The resolution
> source … is the United States Geological Survey (USGS) Earthquake Hazards Program, with the
> minimum magnitude set to 5.5 …"*

**Gate 0, run on all 41 parseable resolved boards** (window parsed from the description,
ET→UTC with correct DST, recounted from the USGS FDSN API):

| family | reproduced | mismatched |
| --- | --- | --- |
| **M6.5+ weekly** | **21 / 21 (100%)** | 0 |
| M5.5+ weekly | 15 / 20 (75%) | **5, every one off by exactly 1** |

**The M5.5+ misses are not a bug — they are the single most important fact about this family.**
Magnitudes are reported to 0.1, and **2.01 events per week sit at *exactly* M5.5** (4.60/week
within ±0.10). A routine post-hoc revision of ±0.1 moves the count by 1 — i.e. **one whole
bucket** — and today's catalogue is not the catalogue that resolved the market
(`wiki/reference/first-print-vintages.md`, in a new costume). Weekly sensitivity measured
directly: `n(≥5.45) − n(≥5.55)` ran **1 to 5** across the last 28 weeks. At M6.5 only
0.16 events/week sit within ±0.05 of the threshold, which is exactly why that family
reproduces 21/21.

Two consequences, both written into the design:
- **The M6.5+ ladder is the clean instrument; the M5.5+ ladder is the high-variance one.**
  Trial on M6.5+ first.
- **Revision noise is a modelled term, not an error.** It widens the predictive distribution
  by ~±1 count on M5.5+ — which *reinforces* the thesis (the crowd treats the count as exact)
  but must be calibrated, not assumed. Vintages are reconstructible from ComCat origin
  products (§1).

Also verified: the two families cite *different* USGS pages (the M5.5+ boards the search
interface with an explicit `minmagnitude`, the M6.5+ boards the "significant earthquakes"
browse page) — the counts agree either way at 6.5, but the difference must be re-checked if
Polymarket ever changes the boilerplate.

---

## 4. What is actually measured against the market

Checkpoint = board open + 6h (Monday), de-vigged leg prices from CLOB history, scored against
the realised winning bucket.

**Crowd calibration at window-open** (`wiki/reference/recurring-crowd-calibration.md`):

| family | n | mean winner price | Herfindahl benchmark | read |
| --- | --- | --- | --- | --- |
| M6.5+ | 22 | 0.364 | 0.366 | **calibrated on the favourite** |
| M5.5+ | 13 | 0.230 | 0.207 | mildly underconfident |

So there is no level edge, and this idea does not claim one.

**Log-loss vs the market**, using a conditional empirical benchmark built from the 1,385-week
catalogue (a deliberately crude stand-in for ETAS — nearest-neighbour on elapsed count):

| family | checkpoint | n | market LL | model LL | **gain** | se | model wins |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **M6.5+** | **open +6h** | 22 | 1.183 | 1.072 | **+0.110** | 0.046 | **17/22** |
| M6.5+ | +48h | 26 | 1.123 | 1.087 | +0.036 | 0.065 | 15/26 |
| M6.5+ | +96h | 26 | 0.665 | 0.662 | +0.003 | 0.041 | 15/26 |
| M6.5+ | +120h | 26 | 0.533 | 0.480 | +0.053 | 0.021 | 17/26 |
| M5.5+ | open +6h | 13 | 1.745 | 1.712 | +0.033 | 0.152 | 8/13 |
| M5.5+ | +48h → +120h | 14–15 | — | — | −0.07 … +0.00 | ~0.11 | ~7/15 |

**Read this honestly.** The only checkpoint with a real signal is **M6.5+ at window-open:
+0.110 log-loss, t = 2.4, 17 of 22 boards.** That is the same magnitude as the mechanism
currently trialing in slot 2 (`arena-rank/favourite-shrinkage`: +0.111 out-of-sample,
t = 2.63), so it is a known-acceptable effect size for this firm — but n = 22 and the
benchmark is a crude table, not the ETAS simulation. **The idea's whole bet is that a real
ETAS simulation with revision noise materially beats +0.110.** If it does not, this is not
worth a slot, and gate 2 says so numerically.

Mid-window checkpoints tie the market, which is consistent with the speed-race finding: by
then the crowd has the realised count and the remaining edge is being arbitraged in the hour
after each event.

---

## 5. Example markets — real numbers, 2026-07-25

**1. `how-many-5pt5-or-above-earthquakes-july-20-july-26` — 8 legs, negRisk, $19,934, resolves
Jul 26 23:59 ET.** Books in §2. Live prices ≤8 **0.130** · 9 0.235 · 10 0.250 · 11 0.165 ·
12 0.149 · 13 0.080 · 14 0.040 · >14 **0.042** (leg sum 1.091 → ~9% overround).
Realised count from USGS as of 09:0xZ today: **8 events**, one of which is reported at
*exactly* M5.5 (Pacific-Antarctic Ridge, Jul 20 05:38Z) — a single downward revision moves the
whole ladder one bucket. **This board is mid-window and is therefore explicitly NOT a trade
under this strategy**; it is here to show the instrument and the knife-edge.

**2. `how-many-6pt5-or-above-earthquakes-july-20-july-26` — 7 legs, $5,369.**
0: **0.770** · 1: 0.210 · 2: 0.034 · 3: 0.013 · 4: 0.002 · 5: 0.004 · >5: 0.001.
Realised so far: 0. Empirical unconditional for a full week is P(0) ≈ 0.42; four days in with
zero events the conditional is much higher, so the level looks sane — again, the claim is
about the tail buckets, not the favourite.

**3. `how-many-6pt5-or-above-earthquakes-july-27-august-2` — the actual trade vehicle, opened
2026-07-24 15:29Z for a window starting Jul 27 00:00 ET.** 7 legs, $465 volume so far.
0: 0.585 · 1: 0.250 · 2: 0.096 · **3: 0.175** · 4: 0.025 · 5: 0.016 · **>5: 0.125**.
Leg sum **1.272** — and the `3` leg (0.175, bid 0.03/ask 0.32) is quoted *above* the `2` leg
(0.096), a monotonicity violation on a distribution that must be decreasing out here.
Spreads are currently 3–29c because the board is two days old; **this is precisely the state
the book-quality gate must reject until the market makers arrive** (the same freshly-listed
placeholder behaviour `ladder-rv` documented on 2026-07-25). Entry is at window-open Monday,
by which time the comparable board this week was quoting 1–5c.

---

## 6. Backtest supply — concrete

- **44 resolved boards, 2026-01-05 → 2026-07-20**: ~21 M6.5+ weekly, ~15 M5.5+ weekly, plus
  semi-annual/annual M7.0+ boards. Weekly cadence, two boards per week → **a trial scores its
  first forward instance in ≤7 days** and has a paired backtest immediately.
- Volumes $8k–$280k per board (M6.5+ family peaked at $279k in Feb, currently $5–35k).
- **Per-leg CLOB history: 343 / 343 tokens returned non-empty** at `fidelity=60` with explicit
  `startTs`. Full pre-resolution series for every leg of every board.
- **Ground truth for the model is 1,385 weeks (26.5 years) of complete USGS catalogue** —
  M5.5+ global completeness is essentially perfect post-1990. This is the unusual asset here:
  the *market* sample is 44 instances but the *physics* sample is 1,385 weeks, so the
  simulation can be fitted and validated far beyond what the market history alone supports.
- Adjacent capacity if the model works: `how-many-7pt0-or-above-earthquakes-in-2026` ($1.36M),
  `how-many-large-volcano-eruption-vei-4-in-2026` ($1.15M), `how-many-tornadoes-in-the-us-in-2026`
  ($79.5k) — same overdispersed-count structure, no bookmaker, but annual resolution, so they
  are a scaling venue, never a trial vehicle.

---

## 7. Falsification sketch — numeric kill thresholds

**Gate 0 — resolution reproduction and vintages.** Recount all 44 boards from USGS.
Reconstruct resolution-time vintages from ComCat origin products for the five known M5.5+
mismatches and confirm each is explained by a magnitude revision across the threshold.
> **KILL if** M6.5+ reproduction is below 20/21, or if the M5.5+ mismatches are *not*
> explained by revision (i.e. our window/timezone logic is wrong instead).

**Gate 1 — does ETAS beat the empirical marginal on the physics alone?** Out-of-sample on the
1,385-week catalogue (fit pre-2015, score 2015–2026), predictive log-loss of the ETAS
simulation vs the empirical marginal vs Poisson, on the actual traded bucket lattices.
> **KILL if** ETAS does not beat the empirical marginal by ≥0.05 log-loss on the physics
> sample. If it cannot beat a lookup table on 26 years of its own data, it will not beat a
> market.

**Gate 2 — does it beat the market at window-open?** Leave-one-month-out, M6.5+ and M5.5+
boards separately, de-vigged prices at open+6h.
> **KILL if** the gain over the market is **< +0.110 log-loss** (the crude benchmark already
> achieves that — the simulation must justify itself), or if it wins in fewer than 15/22
> boards, or t < 2.

**Gate 3 — net of fee, spread and delay.** Trade only legs where the book passes (spread ≤5c,
top-of-book ≥$100, price ≥3c), at the ask/bid, `fee = shares × 0.05 × p(1−p)`, signal frozen
at the daily run, **fill at t+24h with +2c adverse**, hold to resolution.
> **KILL if** net edge < 3c/share, or if it sign-flips between the Jan–Apr and May–Jul halves.

**Gate 4 — the artifact hunt** (the discipline that killed today's other candidate). Decompose
every claimed edge by book state and by leg volume; the headline number must be the
**live-book** number. Book state already measured at 0% dead (§2), so this gate should pass —
which is exactly why it must be re-run on the full sample by an independent code path.
> **KILL if** the edge concentrates in legs below $500 volume, or if it inverts with liquidity.

**Gate 5 — capacity.** Depth is ~$2.4–4.4k of resting liquidity per leg and taker volume is
$700–13,000 per leg per week.
> **KILL if** fewer than 4 legs/week clear the book gate with ≥$200 of takeable size at a
> price the model disagrees with by more than the cost stack.

**Pre-registered live stop:** if the first 12 forward boards (six weeks) show a
monthly-clustered edge below 3pp, stop regardless of the backtest.

**Standing caution.** Two independent things in this file are weaker than they look and the
trial must treat them as the leading hypotheses: (a) n = 22 boards at t = 2.4 is one good
month away from being noise; (b) the crowd is *calibrated on the favourite*, so if they are
also roughly right on the tails, the entire remaining edge is the sub-3c wing legs where fees
and the 3c floor make it unfundable. Gate 3 is the one that decides this idea.
