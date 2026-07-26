---
date: 2026-07-26
slug: tomatometer-review-arrival
status: backlog # backlog | trialing | discarded-idea | promoted
example_markets:
  [
    "spider-man-brand-new-day-rotten-tomatoes-score-20260630144021976",
    "paw-patrol-the-dino-movie-rotten-tomatoes-score-20260709174855589",
    "the-odyssey-rotten-tomato-score",
    "in-the-grey-rotten-tomatoes-score",
  ]
model: claude-opus-5 (effort max)
summary: >-
  Polymarket runs a weekly ladder on "will this film's Rotten Tomatoes score be at least
  X" that resolves at a fixed clock instant while critic reviews are still arriving. The
  score is round(100*liked/total) over a denominator that roughly triples during the
  market's five-day life, and it drifts DOWN as it does — measured -2.2 points over the
  final 72 hours and -4.1 points from the embargo lift, 11 of 14 films down, 1 up. The
  crowd prices the number currently displayed on the RT page (market implied median sits
  +0.5 above it). We simulate the arrival process instead. Shape-and-level claim on a
  counting process with a free, machine-readable feed and no professional counterparty.
---

# Tomatometer ladders: the score is still being counted when the market resolves

**Level or SHAPE? Both, and they are separable — which is why this is worth a slot.** The
_level_ claim is a measured, directional bias (the displayed score falls between now and
resolution). The _shape_ claim is that the residual uncertainty is not "how good is the
film" but binomial noise on a known number of not-yet-arrived reviews, which is a
computable width. We can trade either alone; the falsification sketch tests them
separately so a failure of one does not hide behind the other.

---

## 1. What the market is, in plain English

Rotten Tomatoes gives every film a "Tomatometer" percentage. It is not an average rating —
it is simply the fraction of professional critics who gave the film a positive review,
rounded to a whole number. If 413 out of 439 critics liked it, the score is 94%.

Polymarket lists, for most wide releases, a board of four to nine separate yes/no bets:
_will this film's Tomatometer be at least 80? at least 85? at least 90? at least 95?_ Each
one settles on whatever number the Rotten Tomatoes page is displaying at a stated instant —
usually 10:00 AM ET on the Monday after opening weekend.

Here is the part almost nobody trading these boards seems to think about. **At that
instant, critics are still filing reviews.** A film opens on Friday with maybe 130 reviews
counted; by the Monday deadline it has 250; three weeks later it has 440. The score is a
running fraction whose denominator is still growing while the market is open — and the
board itself only exists for about five days, so its entire life sits inside the window in
which the answer is being generated.

That matters because the pool of critics who file _early_ is not a random sample of the
pool that files _late_. Early filers are the ones invited to premieres and given advance
screeners, and they are systematically kinder. So the number on the page at the moment you
look is, on average, higher than the number that will be on the page when the market
settles. We measured how much: **about two points over the final 72 hours, about four
points from the moment the review embargo lifts.**

What we would do: read Rotten Tomatoes' own counts (it publishes the raw numerator and
denominator, not just the rounded percentage), simulate the reviews that have not landed
yet, and price every rung of the ladder off the resulting distribution of the final
rounded score — instead of off the number currently displayed.

**The honest catch:** the number currently displayed is public, so this is not a secret;
it is an arithmetic bias in how the public number is read. It will work only while the
crowd keeps anchoring on it, the boards are small ($15k–$100k), and on any given film the
whole thing can be decided by two or three reviews landing in the last hour.

---

## 2. The screens, answered before the thesis

### 2a. Is there a professional counterparty? — No bookmaker. One rival retail venue.

No sportsbook or exchange prices film critic scores. The
[sharp-line screen](../wiki/reference/sharp-line-screen.md) has no line to fetch here,
which the wiki flags as one of the few genuinely good reasons to expect an edge to
survive — and as a reason the other gates must carry more weight.

**But there is a second venue, and I found it before proposing this: Kalshi runs the same
family.** Its public series list carries **233 `KXRT*` / `RT*` Rotten Tomatoes series**
(`KXRTWICKEDFORGOOD`, `KXRTPREDATORBADLANDS`, `KXRTJURASSICWORLDREBIRTH`, `KXRTWEAPONS`,
`KXRTONEBATTLEAFTERANOTHER`, …), plus a Metacritic family for video games. Kalshi's market
data API is open and unauthenticated. **This is gate 0 of the falsification sketch** — but
note the asymmetry with the tennis/Pinnacle kill: Kalshi is another retail crowd reading the
same RT page, not a risk-taking book. Agreement between the two venues is therefore *weak*
evidence against the idea (they share the anchor); **Kalshi pricing the drift while
Polymarket does not** would be strong evidence the bias is known — and would also be an
easier cross-venue trade than the one proposed here.

There is no free published simulator. A web search for the object turns up only
metadata-based score predictors (budget/genre/director → rating) from blog and IEEE work —
a completely different and much weaker problem than "given 137 reviews in hand, what will
the rounded fraction be in 72 hours". Nobody publishes the latter.

### 2b. Is the within-window state glanceable? — Yes, and that is the point.

The wiki's newest kill-screen ([market-selection](../wiki/market-selection.md), "glanceable
within-window state") sank the Netflix Top-10 idea: subscribers could just *look* at the
answer. Here the current score is equally glanceable — and the crowd demonstrably does look
at it. **That is the mechanism, not the obstacle.** Our claim is not that we observe
something they cannot; it is that the thing they observe is a biased estimator of the thing
that settles, and correcting it requires the denominator, the arrival curve and the
per-critic history — none of which is on the page they are reading.

This is the same structural shape as `arena-rank/favourite-shrinkage` in slot 2 (a
mis-shaped crowd distribution), but the mechanism is different and market-specific: there,
the crowd mis-allocates across a latent ranking; here, the crowd substitutes a partially-
realised statistic for its own terminal value.

### 2c. Speed race? — No. The signal is a bias, not a print.

This is the screen that killed `runningmax`. It passes cleanly here for a structural
reason: **there is no print to react to.** The drift is not news arriving; it is a
predictable property of an arrival process that plays out over three to five days. The
current score is public the entire time. A latency bot that races to reprice on each new
review is racing to track a number that is *itself* biased — it would trade the same wrong
anchor faster.

Corroborating evidence that no one is patrolling these books tightly: on today's Spider-Man
board the `90+` leg is quoted **0.650 / 0.830 — an 18-cent spread** — sitting between two
legs quoted at 0.8c and 1.0c. A market maker tracking the object would not leave that hole.

### 2d. Phantom midpoints? — The family passes at the family level; individual legs do not.

Across **320 legs** on 60 boards with price history, only **2 legs (0.6%)** never moved
(total variation < 2c); median total variation **1.48**. That is the earthquake-ladder
profile (0/314 dead) and the opposite of esports (23% dead) and tennis (8.5%). The family is
alive.

**Per-leg it is not.** The Spider-Man `90+` leg quotes a Gamma midpoint of **0.740** off a
0.650/0.830 book with **$265 of bid depth and $54 of ask depth** — that is a phantom, and
the implied "61.5% chance the score lands in [90,95)" that a naive read of the ladder
produces is fabricated. Any variant built from this must gate every leg on spread ≤ 3c
_and_ depth before it enters a prediction row.

### 2e. Checkpoint artifact / leg-sum? — Real, and it dictates the checkpoint.

Median board lifetime is **5.1 days** (n=55 boards resolved in 2026; min 3.0, max 46.1). At
T-14d only 11 of 67 boards even have quotes, the ladder is non-monotone on 6 of those 11, and
**the market loses to a uniform-over-buckets null (log-loss 3.575 vs 1.655)**. That is
textbook [checkpoint artifact](../wiki/reference/checkpoint-artifact.md) — the board is
listed but unpriced. **The checkpoint must be T-96h or later**, where 57–64 boards are
priced and monotonicity violations drop to 5/57.

---

## 3. The specific data source, and why it is not in the price

**Rotten Tomatoes' own page, which publishes the raw counts.** Every `/m/<slug>` page
carries an embedded `<script id="media-scorecard-json">` blob:

```json
"criticsScore": {"averageRating":"8.70","certified":true,"likedCount":413,
                 "notLikedCount":26,"ratingCount":439,"reviewCount":439,"score":"94"}
```

plus a **separate Top-Critics subscore** (`"likedCount":71,"notLikedCount":4,"score":"95"`).
Reachable with plain `curl` and a browser UA — verified live today. This is a
market-specific feed in the strictest sense: it prices nothing else on Polymarket.

Three things it gives us that the displayed percentage does not:

1. **The denominator.** `n = 439` tells you how much the score *can still move*. A film at
   94% with n=125 and a film at 94% with n=400 are completely different bets, and the ladder
   prices them identically.
2. **The exact rounding lattice.** The score is `round(100 · liked / total)` to the nearest
   integer — verified on six independent (liked, total, score) triples: 122/125 = 97.6 → 98,
   315/330 = 95.45 → 95, 413/439 = 94.08 → 94, 67/227 = 29.52 → **30**, 94/224 = 41.96 → 42,
   122/134 = 91.04 → 91. So a "95+" leg is really `liked/total ≥ 0.945`; at n = 350 that is
   `liked ≥ 331`, an **integer boundary**. Near a strike the answer is decided by one or two
   reviews, and the correct probability is a lattice computation, not a judgement about the film.
3. **The Top-Critics split**, which is a direct handle on pool composition: the gap between
   the all-critics and top-critics score is the observable proxy for the selection effect
   that drives the drift.

**Backtest history is reconstructable.** The Wayback Machine holds **54–78 captures per film
page**, and around release week the density is **5–7 captures per day** (The Odyssey:
20260714 → 20260725, 5–7/day). Each capture is the gzipped original HTML, so the scorecard
JSON comes straight out of it. Here is The Odyssey's actual path, reconstructed today:

| UTC | liked | notLiked | n | score |
|---|---|---|---|---|
| Jul 14 05:34 | 0 | 0 | 0 | — (embargo) |
| Jul 15 17:06 | 122 | 3 | 125 | **98** |
| Jul 15 23:00 | 177 | 5 | 182 | 97 |
| Jul 16 01:09 | 181 | 7 | 188 | 96 |
| Jul 17 20:39 | 315 | 15 | 330 | 95 |
| Jul 19 01:38 | 345 | 18 | 363 | **95** |
| Jul 26 (today) | 413 | 26 | 439 | 94 |

The board `the-odyssey-rotten-tomato-score` carried strikes 60/70/80/90/95/**96/97/98/99**
and resolved with 95 YES and 96 NO — **final score 95**. A trader reading the page when the
embargo lifted saw **98** and had four rungs above the eventual answer to be wrong about.

**Why it is not in the price.** Because the crowd reads the rendered percentage, not the
counts — and the drift only exists relative to a quantity (the terminal denominator) that
the page never shows.

---

## 4. Measured evidence

### 4a. The drift is real and directional (n = 14 films with reconstructed paths)

From first observed score (embargo lift) to last observation before resolution:

| film | first n | score | last n | score | Δ |
|---|---|---|---|---|---|
| evil_dead_burn | 32 | 94 | 151 | 71 | **−23** |
| mortal_kombat_ii | 60 | 75 | 155 | 65 | **−10** |
| scream_7 | 40 | 43 | 135 | 34 | **−9** |
| in_the_grey | 27 | 48 | 45 | 44 | −4 |
| the_odyssey_2026 | 125 | 98 | 363 | 95 | −3 |
| predator_badlands | 67 | 88 | 224 | 85 | −3 |
| wicked_for_good | 68 | 72 | 279 | 69 | −3 |
| star_wars_mando_grogu | 72 | 64 | 235 | 62 | −2 |
| the_super_mario_galaxy_movie | 66 | 44 | 187 | 42 | −2 |
| 28_years_later_bone_temple | 97 | 94 | 236 | 93 | −1 |
| marty_supreme | 129 | 95 | 249 | 94 | −1 |
| zootopia_2 | 44 | 93 | 173 | 92 | −1 |
| the_devil_wears_prada_2 | 112 | 78 | 264 | 78 | 0 |
| the_running_man_2025 | 61 | 64 | 246 | 65 | **+1** |

**Mean −4.14, median −2.0. Eleven down, two flat, one up** (sign test on 11 vs 1, p ≈ 0.006).

And it is **conditional on the denominator**, exactly as the selection mechanism predicts:

| first-observation n | films | mean drift |
|---|---|---|
| n < 80 | 11 | **−5.09** |
| n ≥ 80 | 3 | **−0.67** |

That conditioning is the whole reason a simulator is needed rather than a constant fudge:
the correction is a function of state, and `n` is observable for free.

### 4b. At the checkpoint we would actually trade, the drift is smaller but still there

From the last capture at or before **T−72h** to the last capture before resolution
(n = 13 films; median **96 reviews added** in that window):

**Mean −2.23, median −2.0 · 8 down, 4 flat, 1 up.**

Two points is not small against these ladders. Strike spacings in the family run 1 to 10
points — `how-to-make-a-killing` was **56/57/58/59/60** and settled at 57; `marty_supreme`
was 93/94/95/97/98; `the-odyssey` was …/95/96/97/98/99; `28-years-later` was
89/91/93/95/97. On a 1-point lattice, a systematic 2-point shift moves the entire ladder.

### 4c. The market does not price it

At the first Wayback observation of each film (embargo lift), I interpolated the market's
implied **median** score from the ladder and compared it to the score displayed at that
instant (n = 9 boards with quotes at that time):

> **market implied median − displayed RT score: mean +0.74, median +0.50**

The market centres on the number on the page — very slightly _above_ it — while the realised
change from that instant is **−4.1**. That gap is the trade.

### 4d. The ladder's width is also wrong — and in the direction that pays

Calibration of the market's own implied bucket distribution against realised buckets, by
checkpoint (all boards resolving since 2025-11):

| checkpoint | n | market log-loss | uniform-null LL | modal bucket wins | Herfindahl | mode priced at |
|---|---|---|---|---|---|---|
| T−14d | 11 | 3.575 | 1.655 | 0.364 | 0.377 | 0.483 |
| T−7d | 15 | 2.544 | 1.625 | 0.333 | 0.395 | 0.502 |
| **T−96h** | 43 | 1.387 | 1.687 | 0.605 | 0.494 | 0.612 |
| **T−72h** | **57** | **0.981** | 1.733 | **0.684 ± 0.062** | **0.535** | 0.641 |
| T−48h | 62 | 0.770 | 1.744 | 0.661 | 0.633 | 0.732 |
| T−24h | 64 | 0.527 | 1.763 | 0.766 | 0.686 | 0.759 |
| T−6h | 64 | 0.363 | 1.763 | 0.875 | 0.752 | 0.816 |

Read the T−14d and T−7d rows as the **checkpoint artifact** (§2e), not as edge — the null
wins because the book is unpriced. From T−96h on the market is genuinely priced and beats
the null, and there the modal bucket **wins more often than the crowd's own distribution
says it should** (0.684 vs Herfindahl 0.535 at T−72h; PIT lands in the outer 20% only
5.3% of the time against 20% expected under calibration). The crowd's distribution is
**too diffuse and centred too high**. Our simulation replaces both: it shifts the centre
down by the drift and tightens the width to the binomial width implied by the reviews still
to come.

---

## 5. The simulation, and why simulation beats a closed form

**Object.** Simulate `S(T) = round(100 · L(T) / N(T))` at the resolution instant `T`, given
the state `(L(t), N(t))` observed at checkpoint `t`, then read every ladder leg off the
sampled distribution at once.

**Generative model, per draw:**

1. **Latent film quality** `q ~ posterior(L(t), N(t), priors)` — the long-run fresh-rate of
   the *full* critic population, which is not `L(t)/N(t)`.
2. **Selection offset** `δ(n_t)` — the gap between the early pool's fresh-rate and `q`,
   fitted as a function of the observed denominator (the −5.09 vs −0.67 split above is the
   crude one-parameter version of this) and of the observed all-critics-minus-top-critics gap.
3. **Arrival process** `N(T) − N(t)` — a doubly-stochastic counting process fitted per
   release scale, with strong day-of-week and hour-of-day structure (embargo Tue/Wed burst,
   Friday opening bump, weekend tail). Median 96 additional reviews over 72h in our sample,
   but the dispersion is the thing that matters.
4. **Per-critic verdicts** — each arriving critic is Bernoulli with a probability tilted by
   that critic's own lifetime fresh-rate (available from their RT profile) and by `q`. Not
   independent: they share `q`, which is what makes the terminal distribution wider than a
   plain binomial and narrower than the crowd's.
5. **Round and threshold** on the integer lattice.

**Why a closed form does not exist here.** The composition is a random-denominator ratio
(`N(T)` is stochastic), through a non-linear integer rounding, of correlated Bernoullis with
heterogeneous, time-varying marginals, evaluated at a wall-clock instant rather than at
completion — and then read out on 4–9 *simultaneous* thresholds whose joint distribution is
what the portfolio P&L depends on. Every one of those five features alone kills the
Beta-Binomial closed form; together they leave Monte Carlo as the only honest route. And
because the legs are correlated through one latent, pricing the ladder coherently requires a
joint sample, not five marginal calculations.

**Where the Rust actually goes.** Not the per-board simulation — 10⁷ paths on one board is
milliseconds. It goes into (a) fitting the hierarchical critic-effects model over the
per-critic review panel (tens of thousands of reviews × thousands of critics, refit weekly),
and (b) the historical replay: 60+ boards × ~15 Wayback captures × the full checkpoint grid
× the joint simulator, which is the loop the falsification sketch below runs repeatedly.
I would rather state that honestly than claim a compute bound that is not there.

---

## 6. Liquidity evidence — where the money actually is

This is the gate our first headline failed
([midpoint-is-not-a-fill](../wiki/reference/midpoint-is-not-a-fill.md)), so it is measured
first, from the Data API tape, on the window we would trade.

**Taker flow in the final 72 hours, ten largest boards resolved since 2026-02:**

| board | headline vol | total taker $ | last 72h $ | of which in 8–92c | fills 72h | wallets 72h | top wallet |
|---|---|---|---|---|---|---|---|
| in-the-grey | 102,006 | 54,539 | **52,035** | **48,846** | 215 | 93 | 56.2% |
| scream-7 | 160,721 | 64,190 | 33,612 | **37** | 542 | 209 | 33.1% |
| michael | 88,013 | 40,493 | 20,638 | **7,635** | 647 | 297 | 22.5% |
| good-luck-have-fun-dont-die | 42,877 | 32,511 | 17,751 | **4,798** | 121 | 69 | 35.1% |
| the-super-mario-galaxy-movie | 52,902 | 26,483 | 11,985 | **19** | 250 | 107 | 16.6% |
| how-to-make-a-killing | 645,862 | 50,325 | 11,307 | **9,842** | 890 | 33 | 35.2% |
| wuthering-heights | 55,107 | 42,398 | 9,714 | 2,267 | 206 | 78 | 37.8% |
| melania | 37,627 | 27,014 | 6,183 | 3,195 | 254 | 126 | 18.2% |
| the-odyssey | 41,202 | 23,112 | 4,258 | 1,876 | 157 | 62 | 14.2% |
| goat | 49,370 | 34,492 | 3,007 | 578 | 99 | 55 | 42.6% |

**The finding, stated honestly: the fundable band is real but state-dependent, and it is
non-empty exactly when our edge is largest.** When the film's final score lands near a
strike, the last-72h band flow is $3k–$49k across 33–297 distinct wallets — that is a
tradeable book. When it lands far from every strike (scream-7 settled at 30 against strikes
50/60/70/80/90; mario at 42 against 45/50/55/60) the board collapses to 0/1 early and the
8–92c band holds **$19–$37** — nothing. Since the whole edge is about resolving the ladder
near a boundary, edge and liquidity coincide rather than trade off. That is a much better
property than the wing legs that produced our 2/21 fillable batch.

**Fundable-leg count across the family:** at T−72h, **50 of 60** boards have at least one leg
quoted in 10–90c, **35 of 60** have two or more; mean **2.02 legs** per board.

**Fees:** these markets carry `feeType = culture_fees`, `feeSchedule = {rate 0.05, exponent 1,
takerOnly true, rebateRate 0.25}`. Taker cost is `shares × 0.05 × p × (1−p)` — **0.57c/share
at p = 0.87 or 0.13**, 1.25c at p = 0.50. Against a 2-point systematic score shift on a 1–5
point strike lattice that is affordable, but it must be in every P&L line, and it is charged
on entry **and** exit while resolution is free — so hold to settlement.

**Wallet concentration** runs 14–56%. `in-the-grey`'s 56.2% needs the
[wash-trading tests](../wiki/reference/wash-trading.md) before that $48.8k is believed; the
289-wallet boards (michael, scream-7) do not.

---

## 7. Live example markets (numbers pulled 2026-07-26 01:34 UTC)

### `spider-man-brand-new-day-rotten-tomatoes-score-20260630144021976` — the primary example

"Spider-Man: Brand New Day" Rotten Tomatoes Score. Board volume **$15,537**, liquidity
**$14,634**, opened 2026-07-01, **resolves on the displayed All-Critics Tomatometer at 10:00
AM ET on 2026-08-03**. Film in theatres **Jul 31**. `negRisk = false` — four independent
binaries. RT page today: **`reviewCount: 0`** — the embargo has not lifted.

| leg | Gamma mid | best bid | best ask | spread | depth ≤5c (bid / ask) | levels | leg vol | tape |
|---|---|---|---|---|---|---|---|---|
| **80+** | 0.928 | 0.924 | 0.932 | **0.8c** | $3,552 / $3,881 | 37 / 23 | $2,886 | 64 fills, 40 wallets, top 24.9% |
| **85+** | 0.875 | 0.870 | 0.880 | **1.0c** | $4,308 / $4,850 | 23 / 10 | $2,981 | 36 fills, 28 wallets, top 24.8% |
| 90+ | _0.740_ | 0.650 | 0.830 | **18c** | $265 / $54 | 13 / 9 | $896 | 32 fills, 21 wallets — **phantom, do not use** |
| **95+** | 0.125 | 0.120 | 0.130 | **1.0c** | $392 / $448 | 11 / 26 | $8,774 | 95 fills, 64 wallets, top 20.1% |

Board tape to date: **227 fills, ~$6,600 taker notional**, last fill 2026-07-25 23:49Z.

Reading only the three tight legs, the market implies **P(85 ≤ score < 95) = 0.875 − 0.125 =
0.750** and **P(score < 80) = 0.072**. Franchise anchor is obvious: No Way Home 93, Homecoming
92, Far From Home 90.

**The trade this idea generates, concretely:** do nothing until the embargo lifts (expected
Jul 28–29). Then read `(likedCount, notLikedCount)`, simulate `S(Aug 3, 10:00 ET)`, and take
the tight legs against it. If the embargo pool comes in at 93% on n ≈ 120, the −5.09 small-n
drift plus binomial width puts real mass below 90 and very little at 95+ — and `95+` at a
**0.130 ask with $448 of depth inside 5c** is precisely the kind of leg that is both wrong
and reachable. If it comes in at 96% on n ≈ 200, the model may well agree with the crowd and
we pass. **The idea does not require us to trade every board.**

### `paw-patrol-the-dino-movie-rotten-tomatoes-score-20260709174855589` — the counter-example

$862 volume, resolves 2026-08-17. Legs quoted 60+ **0.620/0.910 (29c)**, 70+
**0.200/0.890 (69c)**, 80+ 0.380/0.620 (24c), 90+ 0.100/0.130 (3c). Included deliberately:
**this is what an unfundable board in this family looks like**, and a variant must refuse to
predict on it rather than book a phantom 0.545 midpoint on the 70+ leg. It will become
tradeable, if at all, only in the last 72 hours.

### Family supply

**67 resolved boards** since 2025-02 (60 with usable price history since 2025-11), running at
**2–4 boards per resolution Monday** — 2026-02-16 had four, 2026-05-04 had four, 2026-06-01 had
four. Every board's resolution pattern is monotone: **zero coherence violations across all 67**,
so the ladder semantics are unambiguous. Median lifetime 5.1 days. A 10-day trial should see
**4–8 boards resolve, ~15–35 legs**.

---

## 8. Falsification sketch — cheapest first

**Gate 0 — Kalshi cross-venue (30 minutes, no modelling).** For every film where both venues
listed an RT board (Predator Badlands, Wicked for Good, Weapons, One Battle After Another,
Jurassic World Rebirth, …), pull Kalshi settlement history and compare implied medians on
tight books at T−72h. **Kills the idea if Kalshi's ladder already sits ~2 points below the
displayed score while Polymarket's sits at it** — that would mean the drift is known and
Polymarket is simply the slower of two crowds, which is a different (and much thinner) trade
than the one proposed. Mere agreement between the two venues does **not** kill it; both are
retail crowds reading the same page.

**Gate 1 — widen the drift measurement (2 hours).** Reconstruct Wayback score paths for all
~60 boards, not 14. Require: (a) median T−72h→T drift ≤ −1.0 point, (b) sign test still
significant, (c) the n-conditioning survives (small-n boards drift more). **Kill if the
median drift at T−72h is under 1 point or the sign flips on the larger sample.** My n=14 is
the single weakest number in this file and it is the first thing to attack.

**Gate 2 — leg-sum and null-model gate (1 hour, run before any model).** At the chosen
checkpoint confirm ladder monotonicity within 2c and confirm a **zero-drift null** — "final
score = score at checkpoint, plus binomial noise on the expected arrivals, no selection
offset" — roughly *ties* the market and does not beat it. **If the zero-drift null beats the
market, stop**: the checkpoint is unpriced, per
[checkpoint-artifact](../wiki/reference/checkpoint-artifact.md), and this whole file is
measuring a placeholder book.

**Gate 3 — does the drift model beat the market? (half a day).** Paired log-loss and Brier
against the market's own ladder on all resolved boards, checkpoint T−72h, legs gated at
spread ≤ 3c and depth ≥ $200. Report the fillable count beside every aggregate. **Kill if the
paired improvement is under 1.5 standard errors, or if it does not survive a split between
2025-11→2026-02 and 2026-03→2026-07.**

**Gate 4 — is it executable? (half a day).** Run `tools/fillcheck` over the winning legs:
best price a counterparty was actually observed at within one hour of the checkpoint, minus
the `0.05 · p · (1−p)` taker fee, minus a 1-tick slippage assumption. **Kill if median
`exec_edge` is under 1.5c/share, or if fewer than half the model's chosen legs have any
counterparty at all.**

**Gate 5 — speed and delay (2 hours).** Re-run gate 3 with fills at **t+6h and t+24h**,
model inputs frozen at `t`. The thesis says the drift is a multi-day bias, so the edge should
barely move. **Kill if it collapses** — that would mean we are racing the review feed after
all, and the [delayed-execution test](../wiki/reference/delayed-execution-test.md) applies.

**Gate 6 — resolution epsilon.** Confirm on resolved boards how often the settled score
differed from the score at T−1h. A review landing at 09:58 ET can flip a leg;
[venue-resolution-epsilon](../wiki/reference/venue-resolution-epsilon.md) says never sell
inside that epsilon. Quantify it, then set a hard no-trade band of ±1 review around each
strike.

---

## 9. What would make me abandon this, beyond the gates

- **The drift is a composition effect that could be regime-dependent.** RT changed its
  critic-eligibility rules more than once. If the effect is much weaker in 2026 than 2025,
  the sample split in gate 3 will show it.
- **Wayback density thins for small films.** Six of the sixteen films I attempted returned
  too few captures for a clean path (`michael_2026` returned none). Backtest coverage will be
  biased toward big releases; live trading does not have this problem, because
  `workers/snapshot` can poll the RT page hourly — but that means the historical evidence and
  the live strategy run on different data quality, and gate 3 must be honest about it.
- **Board supply is 2–4 per week, not per day.** This is slower scoring than the earthquake
  or barrier families. It is fast enough for a 10-day trial and no faster.
- **RT removes and re-scores reviews.** The denominator is not monotone. Verify on the paths
  before assuming it is.
