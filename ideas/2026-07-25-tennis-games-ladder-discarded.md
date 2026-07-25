---
date: 2026-07-25
slug: tennis-games-ladder
status: discarded-idea
killed_by: sharp-line screen (Pinnacle), confirmed by the phantom-midpoint decomposition
example_markets: ["atp-bublik-halys-2026-07-25", "atp-assche-gaston-2026-07-25", "atp-blockx-darderi-2026-07-25"]
model: opus-5 (xhigh)
---

# DISCARDED: tennis total-games ladder — and two other simulation candidates killed the same day

This was my lead candidate for the 2026-07-25 simulation cycle. It is filed as negative
knowledge because the kill is cheap, decisive, and **independently reproduces both wiki pages
written today from the `series-shape/bo3-derivatives` kill** — in a different sport, on a
different leg type, with a different pipeline. Two further candidates (PGA golf, weekly
earthquake counts *mid-window*) died on other screens. Recording all three so nobody
re-scans them cold.

---

## 1. The candidate

Polymarket lists every ATP/WTA match as **14 derivative legs on one point process** —
`moneyline`, `tennis_set_handicap`, `tennis_set_totals` (O/U 2.5 sets),
`tennis_first_set_winner`, three `tennis_first_set_totals`, three
`tennis_set_games_totals`, and three **`tennis_match_totals`** (Match O/U 21.5 / 22.5 / 23.5
games). Supply is enormous: **10,011 resolved tennis events in 2026 alone** (Jan 1 → Jul 25,
harvested by date-windowed offset paging on `tag_id=864`), **8,675** carrying a
`tennis_match_totals` leg, **5,602** carrying the full ladder, **101,101 resolved legs**,
median event volume **$27,928**, and **≈48 resolved matches per day**.

The thesis was a clean shape claim, and the underlying maths is still correct (§5): I built
an exact DP over the point → game → set → match lattice and showed that the market's two
deepest books cannot see the parameter that drives the totals ladder. Then I measured what
looked like a large edge:

| checkpoint | n | mean market price | realised Over rate | gap (pp) |
| --- | --- | --- | --- | --- |
| T−24h | 613 | 0.4873 | 0.4225 | −6.48 |
| T−6h | 1,575 | 0.4814 | 0.4006 | −8.08 |
| T−1h | 1,676 | 0.4655 | 0.3890 | **−7.64** (se 1.19) |
| T−15m | 1,684 | 0.4724 | 0.3878 | −8.47 |

with the bias apparently concentrated in the fundable band (0.50–0.60 bucket: **−12.06pp**,
n=822). It does not exist.

---

## 2. Kill 1 — the sharp-line screen (`wiki/reference/sharp-line-screen.md`)

Pinnacle's guest API prices tennis **total games** as a *separate matchup* from sets — the
participants carry a `(Games)` suffix (79 of 161 pre-match matchups today). Matching
Polymarket's live board to Pinnacle by participant surnames and de-vigging by normalisation:

**27 matched total-games lines across 13 live matches, 2026-07-25:**

| | n | mean deviation | se | median \|Δ\| | within 3pp |
| --- | --- | --- | --- | --- | --- |
| all matched lines | 27 | **+0.32pp** | 0.12 | 0.36pp | **27/27** |
| Polymarket spread ≤3c | 18 | **+0.07pp** | 0.13 | — | **18/18** |

Examples (Polymarket mid vs de-vigged Pinnacle): Bublik–Halys O/U 23.5 **0.500 vs 0.498**;
Fearnley–Zheng O/U 22.5 **0.500 vs 0.500**; Wild–Dietrich O/U 22.5 **0.500 vs 0.500**;
Van Assche–Gaston O/U 22.5 **0.490 vs 0.486**.

**Polymarket's tennis totals ladder *is* the Pinnacle line, to a third of a point.** What I
had described in the draft as "a market maker's resting quote with no taker price discovery"
($27k of depth inside 5c on a leg with $0 volume) is a sharp bookmaker's line mirrored onto
Polymarket. There is no edge to take, and the screen cost about ten minutes.

---

## 3. Kill 2 — the phantom-midpoint decomposition (`wiki/reference/phantom-midpoints.md`)

Splitting my own 1,683-leg measurement by whether the price ever *moved* pre-match:

| book state | n | mean price | realised | gap |
| --- | --- | --- | --- | --- |
| **all legs (my headline)** | 1,683 | 0.4659 | 0.3898 | **−7.61pp** |
| DEAD (never moved) | 143 | 0.4914 | 0.2168 | **−27.46pp** |
| near-flat (total variation <2c) | 102 | 0.4901 | 0.3235 | −16.66pp |
| LIVE (total variation ≥2c) | 1,438 | 0.4616 | 0.4117 | −5.00pp |
| LIVE (total variation ≥5c) | 1,297 | 0.4582 | 0.4156 | −4.26pp |

And, decisively, **the "edge" inverts with liquidity** — the signature the wiki page names:

| leg volume | n | gap |
| --- | --- | --- |
| $0 | 560 | **−17.26pp** |
| $1–100 | 412 | −7.99pp |
| $100–1k | 510 | −4.36pp |
| **>$1k** | 201 | **+11.78pp** ← sign flips |

The 0.50–0.60 bucket that carried the whole headline is the worst case: **−27.95pp on dead
legs (n=136)** versus −8.21pp on live ones. 8.5% of resolved `tennis_match_totals` legs never
moved before the match. A book quoted 0.05/0.95 reports as a "price" of ~0.50, which is
exactly why the artifact concentrates in the bucket that looks most fundable.

The residual on live books (−4 to −5pp) is not zero, but it cannot be reconciled with the
Pinnacle result (+0.07pp on tight books) as edge: the resolved population is dominated by
low-tier ITF/Challenger events where no sharp line is mirrored and no real book exists, while
the >$1k slice — the only one with genuine two-sided books — flips positive. That is noise
plus artifact, not a crowd error.

**Verdict: dead.** Both kills are independent and both point the same way.

---

## 4. Also killed today

**(a) PGA Tour finishing-position boards — INCUMBENT screen.**
Structurally the best simulation family I found: four separate 100+-leg boards on one 4-round
tournament (`2026-3m-open-winner` 121 legs $115k; `-top5`, `-top10`, `-top20` 100 legs each),
resolving on "top N **including ties**" — an order statistic over ~156 correlated integer
score paths with a 36-hole cut, for which no closed form exists and for which the standard
analytic map (Harville/Plackett-Luce from win odds to place odds) is known to be biased.
**24 tournaments × 3 boards already resolved in 2026**, weekly cadence.

Resolution verified hard against ESPN's free golf API (`site.api.espn.com/apis/site/v2/
sports/golf/leaderboard?event=<id>`) for the 2026 PGA Championship: board YES = 12, ESPN
top-20-including-ties = 25, **0 false YES and 0 listed-but-wrongly-NO** — the 13 "missing"
finishers were simply not listed. Note for anyone returning: the boards carry ~100 of a
156-player field and **the listed subset excludes marquee names** (Scheffler, Rahm, Thomas,
Schauffele and the eventual winner Aaron Rai were all absent from the PGA Championship top-N
boards), so the tempting "board sums to k" coherence test is invalid without modelling the
unlisted field. Also beware `2026-pga-championship-winner` (a $4,040 duplicate board) which
resolved to Collin Morikawa on the **Friday**, two days before Aaron Rai actually won — a
mis-resolved auto-generated board sitting next to the real $8.98M one.

**Killed because `datagolf.com/live-model/pga-tour` embeds the complete live Monte Carlo —
`win`, `top5`, `top10`, `top20`, `cut` per player — as free JSON in the page source**
(Scheffler at read: win 0.312, top10 0.671, top20 0.861). A specialist running shot-level
ShotLink data publishes the exact resolution variable for free; our from-scratch simulation
would be a proxy against their primary — the gistemp failure mode in a new costume. It failed
book quality independently too: **10/100 legs at ≤5c on top5 and top10, 1/100 on top20**,
median spread 6–16c.

**The same kill applies to MLB playoff/division ladders** (`mlb-team-to-make-postseason`
30 legs; six division boards $33k–$983k; `mlb-world-series-champion-2026` $36M) — FanGraphs
publishes free daily playoff odds from 20,000 season simulations. Do not re-propose either
without a genuinely new angle.

**(b) Weekly earthquake-count ladders, *mid-window* — SPEED-RACE screen.**
See the companion idea filed today
(`ideas/2026-07-25-quake-ladder-overdispersion-3.md`) — the **window-open** version of this
family survived and is the real deliverable. What died is the tempting mid-week trade: after
a qualifying M6.5+ lands inside the window, the "0" leg moves **−0.279 within one hour**
against a total move of −0.398 by +24h, i.e. **70% of the repricing is done in the first
hour**. That is the `temp-truncation/runningmax` kill pattern exactly, and it is why the
filed idea trades only at window-open and explicitly forbids mid-window entries.

---

## 5. What is worth keeping from the tennis work

The engine and the identification result are correct and reusable if the venue ever lists a
tennis object Pinnacle does not price:

- Exact DP over the point → game → set → match lattice (~60 lines). Sanity gate:
  `pA = pB ⇒ P(win) = 0.5000`. My first version failed it (0.747) because the tiebreak
  point-win rate must be `r = (pA + (1−pB))/2` — averaging server *and* returner — not
  `(pA+pB)/2`.
- **Games in a completed set ∈ {6,7,8,9,10,12,13} — 11 is impossible**, so
  `P(set games > 10.5) ≡ P(set games > 11.5) ≡ P(the set reaches 5-5)`. Any normal
  approximation to total games is wrong at the strikes that trade.
- **The identification result.** Holding the serve *gap* constant at 0.04 and raising the
  serve *level*, the exact DP gives: moneyline 0.697 → 0.655 (**4.2pp**), `P(3 sets)`
  0.464 → 0.478 (**1.4pp**), but `P(total games > 23.5)` 0.492 → **0.701 (20.9pp)**. The
  market's deepest books are near-invariant to the parameter that drives the totals ladder.
  This is a real structural fact — it just does not pay, because Pinnacle has already
  priced it.
- **Fine print that would matter anywhere:** retirement resolves the totals legs **50-50**
  (measured: 112/1,812 = **6.2%** of legs), and super-tiebreak formats collapse a third set
  to **one** game, destroying the lattice — format must be audited per tournament.

---

## 6. The lesson I am promoting to the wiki

The binding constraint on this whole cycle was not modelling skill. It was: **is there a
professional counterparty?** Three of today's four candidates died to one of two forms of the
same question — *someone already publishes or prices this object* (DataGolf for golf,
FanGraphs for MLB, Pinnacle for tennis). The one that survived (weekly seismicity counts)
survived precisely because **no bookmaker prices it and no institution publishes the
distribution**.

Practical ordering for future cycles, cheapest first:
1. **Does a bookmaker price this object?** → Pinnacle guest API / Smarkets, ten minutes.
2. **Does a specialist publish the simulation free?** → check the page source, not just the
   UI; DataGolf ships the whole model as JSON in the HTML.
3. **Then** decompose any measured edge by book state before believing a single number.

Steps 1 and 2 should run **before** the backtest, not after. I ran mine after, and the
backtest was three hours of manufactured edge.
