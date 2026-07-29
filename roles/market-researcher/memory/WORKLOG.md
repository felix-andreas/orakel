# Market Researcher Worklog

One dated entry per run. Name the exact model id that did the work.

---

## 2026-07-29 (run 8) — a $1.5M board, a real +11pp edge, and $7 at the ask. Post-count ladders killed on leg depth

Model: **claude-opus-5 (effort max)**.

Object: **post-count ladders** — "how many times will @elonmusk post on X this week" and the
Trump Truth Social equivalent, priced as 10–30 leg bucket ladders. Filed
`ideas/2026-07-29-post-count-ladders-discarded.md`, status `discarded-idea`.

Chosen deliberately against `ops/idea-funnel.md`: it is the canonical instance of the one
glanceable-state structure my long-term memory claimed survives — a **partially-realised
cumulative count**, where the visible running number is an anchor and the bias in it is
supposed to be the edge. That hypothesis had been sitting in memory unmeasured since 07-26.
It is now measured. Distinct from the 07-28 mention kill: integer count of a counting process,
not a Bernoulli on utterance content.

**Screens, in the order the playbook demands.**

1. **Kalshi catalogue** (12,298 series today, up from 12,231). The object splits:
   `KXELONTWEETS` is a **dormant shell with 0 markets** — no venue incumbent on Elon;
   `KXTRUTHSOCIAL` is a **live incumbent on Trump** — 102 markets / 92 settled, 100–300k
   contracts and 57–158k OI per week, 1c spreads, identical bucket cuts. And the decisive
   detail: Kalshi **re-cut its cap `>220` → `>240`** as the underlying drifted, while
   Polymarket has cut at `200+` since May. The incumbent had already fixed the exact staleness
   my backtest would later find. Best incumbent result yet — not "their line is unbiased" but
   "their board design already corrects your edge."
2. **Data.** 129 settled boards back to 2024 (Elon weeklies $3–32M each), 1,153 legs of hourly
   `prices-history`. Checkpoint integrity: leg-sums 1.066 → 1.004 monotone from T−168h to
   T−12h, gated to [0.90, 1.15]. Also verified Polymarket's `prices-history` `p` **is the book
   midpoint** (median diff vs live mid = 0.00c) — yesterday's print-at-the-ask trap does not
   apply here.
3. **Calibration.** Pooled edge −0.05 / −0.39 / −0.29 / +1.21pp at T−120/72/48/24h. Calibrated.
4. **The one live band, and its decomposition.** 0.02–0.10 held the same sign at all five
   checkpoints: pooled **+3.28pp, board-clustered t=+2.46**, Wilson lower 0.0684 > mid 0.0531.
   The **mirror test did not fire** (buy-NO on the same legs loses hard), so unlike 07-28 this
   was not the spread and had to be decomposed. Splitting by **leg type** killed it:
   open-ended top bucket **+15.89pp (n=57)** vs interior **+2.31pp (n=747)**. The interior
   residue fails break-even at both measured half-spreads (−0.21pp at 0.55c, −1.20pp at 1.50c).
   The open-HIGH residue is **5 board-wins, all the same leg (Trump `200+`), all May–July** —
   a stale bucket cut, one regime, already fixed by the rival.
5. **The kill that would have sufficed alone.** Walking the live book on legs in the edge band:
   **+1.72c at $100, +6.54c at $500, +14.36c at $2,000** on a 2.65c-mid leg. Median notional
   resting at the best ask: **$7**. Against +11.14pp, q⁻=0.0709 clears q* by +0.07pp at $100
   and fails by −4.95pp at $500 and −13.04pp at $2,000.

**Graduated to the wiki:** `wiki/reference/depth-lives-where-the-edge-is-not.md` — the fourth
way a quote lies, and the one that survives phantom-midpoints, the tape gate and
midpoint-is-not-a-fill all three. The board is genuinely liquid, the tape is real, the mid is
honest, and the leg your edge sits on has $7 behind it. **Depth concentrates at the mode;
mispricing lives in the wings; the two are anti-correlated**, because the property that makes
a leg mispriced is the property that makes it unquoted. Board-level gates are structurally
incapable of seeing this. New standing step: walk the book at your own price band, for your
own size, before any modelling.

Live illustration worth keeping: the Elon `500+` leg shows **$96,266 of lifetime volume and no
quote on either side**; Trump `160-179` quotes 0.050/0.110 (6c on a 5c leg) and `180-199`
quotes 0.200/0.350 (11c) — while the same boards' interior legs quote 1c wide on $74–79k.

Also scanned and rejected before working up: US primary boards (Kalshi runs **175** primary
series — downgraded my best parked candidate), Clacton by-election (Kalshi + Smarkets),
`largest-company-end-of-month` / `grvt-fdv` (live quoted price → Felix's standing
instruction), `highest-grossing-movie-2026` (box office, dead). Noted for the CEO: the
top-volume non-sports tape on 07-29 is overwhelmingly Iran/Hormuz/ceasefire, i.e.
**war-adjacent and blocked pending the domain ruling** — the block is now materially
constraining what I can scan, not just what I can file.

Two consecutive families have now died *behind* the incumbent wall, to execution rather than a
counterparty, and both leave the same single live thread: a **maker-side** construction, which
`CONSTITUTION.md` §5 forbids. That is a pattern worth a decision rather than a third instance.

---

## 2026-07-28 (run 7) — a +5pp bias at t=+5, two days out, that was the spread all along. Mention markets killed on the executable price

Model: **claude-opus-5 (effort max)**.

Worked "will X say WORD during Y" boards. Filed
`ideas/2026-07-28-mention-markets-discarded.md`. **Verdict: discarded-idea.**

**Why this one and not the usual suspects.** The open-market scan (20 pages, 25,072 rows)
plus a resolved-family sweep turned up the usual dead ends — geopolitics (avoid/blocked),
crypto and market-cap boards (efficient objects, Felix's rule), macro (sharp desks), weather
dailies, AI rankings, box office (killed 07-27). Mention markets were the first family in
seven runs to clear **every** positive screen at once: not a quoted underlying; a
hidden-but-recoverable Bernoulli rate; **no free specialist anywhere** (no DataGolf, no
FanGraphs, no Substack PNG — nobody publishes utterance probabilities); hours to resolution;
recurring; base rates spread 0.06–0.91.

**Gate 0, MEASURED.** Kalshi catalogue re-pulled: **12,231 series** today (12,187 on 07-27).
It runs a whole **`Mentions` category — 397 series, 3.2% of the catalogue**. Pulled all of
them: **17,001 markets, 15,258 settled with a `result`, $310.6M settled volume, 813 events,
median 17 legs/event**, all 2026-05→07. Polymarket runs the same family: **447 events,
$301.5M lifetime**, single boards at $53.2M (Trump×Xi bilateral, 33 legs), $10.3M
(inauguration), $5.5M (Powell June presser). 14 boards open today.

**The apparent edge.** Checkpoint = last hourly candle at/before `min(close_time)` over the
event, at leads 1/3/6/12/24/48h, on 6,117 markets in the 51 `KXEARNINGSMENTION*` series + 17
named speaker series. Realised YES frequency sits **below** the traded price in every band
from 0.10 to 0.90 — 0.60–0.75 by −14.6pp. Event-clustered: **+6.51pp t=+6.93 (T−3h),
+5.13/+5.27 (T−6h), +5.36/+5.29 (T−12h), +4.15/+3.71 (T−24h)**. 40.6% of volume trades in
the final hour, so I checked it was not intra-speech decay — it survives to T−48h.

**The kill.** Re-priced at what we can get (buy NO at the bid, buy YES at the ask):
**−2.51pp (t=−2.48) and −7.92pp (t=−7.97)**. *Both sides lose simultaneously*, which is only
arithmetically possible if the thing measured was the spread — and it closes to the cent:
mean spread 8.70c, mean(last trade − mid) +1.83c, **mean(last trade − bid) 6.18c**, and
last-trade-edge minus executable-edge = **6.18pp, identical**. Only 45.9% of events show a
positive executable edge. Under the `tape-gate` relative-spread rule the edge is
**+0.60pp, t=+0.40**. Counting legs ("say X 3+ times") likewise: +1.66pp, t=+0.24, n=40.

**The trap inside the kill, and the day's best find.** `volume_fp ≥ 20k` gives
**+21.15pp at t=+7.30**. It is **look-ahead**: only **14.3%** (median) of a mention leg's
lifetime volume has traded by T−6h. The honest version of the same filter (volume known at
the checkpoint, rebuilt from the candle path) gives **−3.06pp, t=−0.72**. New wiki page.

**Polymarket is not a softer crowd — measured.** Matched by quoted phrase against Kalshi
`KXWARSHMENTION` + `KXEARNINGSMENTIONBA/PG/PYPL/AAPL/META/MSFT`: 52 pairs, raw median |Δ|
10.5pp — but 33 of them were **phantom midpoints** (Apple/Meta/MSFT boards opened today at
$34–$178 volume quoting 0.02/0.98 → a 0.500 mid). Real books both sides (19 pairs):
**+1.87pp, se 0.59, t=+3.16, median |Δ| 2.50pp, 18/19 within 5pp.** Same line, 2pp richer.

**Fundability.** Fees read per market, not assumed: Polymarket `mentions_fees`
`{rate 0.04, exponent 1, takerOnly, rebate 0.25}`; all 397 Kalshi Mentions series
`quadratic ×1` = 0.07·p·(1−p). On the tightest books: gross +1.65pp, **net +0.46pp (Kalshi) /
+0.97pp (Polymarket)**. Break-even table: 1 of 5 bands clears by 0.1pp on the ≤1c subset,
**0 of 5 under the relative-spread gate**.

**The thing worth remembering that is not a kill.** This family **passes the tape gate
outright** — 0 of 30 live tight-spread Polymarket legs have zero tape, ~127 taker trades per
leg, 1–2c spreads, $4.7k–$7.6k of listed liquidity, resolution in 1–2 days. It is also clean
on `stale-feed-gate`: the resolving artifact is the video of the event, generated during the
market's own window, so there is no feed that can be shut while the market moves. Real books,
real takers, fast resolution, no incumbent — **and the price was simply right.** Liquidity is
not always our binding constraint.

Wiki: new `reference/lifetime-volume-is-look-ahead.md`; `midpoint-is-not-a-fill.md` gains the
"both sides lose ⇒ you measured the spread" diagnostic and a rule 0; `sharp-line-screen.md`
gains "filter both sides to real books before comparing venues"; index updated.

---

## 2026-07-27 (run 6) — the empty Kalshi slot was empty for a reason. No positive idea filed; box office killed on the implied-distribution check and a free Substack

Model: **claude-opus-5 (effort max)**.

Worked up the lead my own memory flagged last cycle as "NEXT CYCLE'S LEAD": Polymarket's
domestic box office weekend ladders. Filed
`ideas/2026-07-27-box-office-weekend-ladders-discarded.md`.

**Gate 0, measured before anything else (playbook rule from 07-26).**

- **Kalshi**: dumped all **12,187** series; grepped titles, tickers and every declared
  `settlement_sources` URL. **2 hits, both Golden Globe *award* markets** (`KXGGBOXOFFICE`,
  `KXGGBOFILM`, settling on goldenglobes.com). Zero gross ladders.
- **Pinnacle** guest API: sport 58 "Entertainment" `matchupCount: 0`; `/sports/58/leagues`
  → `[]`. **Smarkets** v3, all 7 states: one annual *winner* market
  ("Highest Grossing Movie 2026") and one placeholder with `{"markets": []}`.
- **`first-print-vintages` rebuild gate**: pulled 187 weekend charts (2023-01 → 2026-07,
  11,421 film-weekend rows) and rebuilt every resolved board from today's live pages —
  **98/98 = 100%** reproduce the exact bucket Polymarket paid. Best feed we have measured
  (PortWatch was 12/19). Only **33 of 11,421** rows anywhere still carry an estimate value,
  5 of them the currently-open weekend.

**Family sizing.** 190 events via unioned `/public-search` variants; 110 resolved three-day
ladders, **50 holdover (Nth-weekend) ladders** of which 46 resolved with volume; 3-5 live
boards a week; $4.8k-$17.1M; `culture_fees` 0.05 taker-only. **13.3%** of resolved boards
landed within 1% of a bucket edge (23.5% within 2%).

**The mechanism I came for, confirmed live and then priced correctly by the crowd.** The
Numbers shows studio *estimates* Sunday midday and *finals* Monday afternoon, and the
boards stay open across that gap by their own resolution text. At 03:20 ET today The
Odyssey's second weekend read $25,800,000 + $34,550,000 + $26,650,000 = **exactly
$87,000,000** (all round to $50k = estimates; the previous Thursday reads $17,625,485,
exact = final; the cell also carries `class="chart_estimate"`). The `86-92m` leg was
**0.975** with its lower edge 1.15% away. Pooled, that price is right: post-estimate
leaders win **94.4%** at a mean price of 0.901, and the 0.90-1.01 band went **71/71**.

**Kill 1 — implied-distribution check** (the `quake-etas` kill, different family). Pulled
the CLOB path for all **524 legs of 110 resolved boards** at five checkpoints. The leading
bucket is calibrated at every one: Fri 12:00 ET 0.605 → 59.0%, Sat 0.738 → 74.1%, Sun 20:00
0.901 → 94.4%, Mon 22:00 0.983 → 97.2% (Brier 0.474 / 0.317 / 0.089 / 0.033). Fitting a
lognormal to each de-vigged ladder: **market implied σ = 0.120 at Friday noon** (n=36),
0.100 Saturday. Against that I built the best forecast the free data supports — **571 daily
charts** (2025-01-01 → 2026-07-26, 652 films) + the weekend panel → 1,363 holdover
film-weekends, 437 at the $2M+ board scale; regressed log(weekend) on prior weekend,
current-week Mon-Thu, theatre ratio, weekend ordinal and seasonal dummies: **σ 0.218 raw /
0.171 robust, in-sample.** The idea's central hope — that the current week's Mon-Thu
dailies are near-sufficient — is false: Mon-Thu alone (0.311) is *worse* than the prior
weekend (0.250), and adding it buys 0.002. Median interior bucket is **10.4%** wide.
Head-to-head on 32 resolved holdover ladders at Friday noon: **market Brier 0.487, us
0.701, we win 8/32.**

**Kill 2 — the counterparty is not a venue, it is a man with a Substack.** **Box Office
Theory** (Shawn Robbins, ex-BoxOfficePro chief analyst) publishes a point forecast for
**every holdover, by weekend ordinal**. Verified against the Substack archive API myself:
**61 "Box Office Weekend Forecast" posts, 2025-01-22 → 2026-07-24, 61 of 61
`audience: "everyone"`, published Wednesdays** (46/61). Measured MAPE vs The Numbers: 9.1%
(n=6, 07-15 issue vs finals) and 14.9% (n=5, 07-22 issue). **~10% MAPE is σ ≈ 0.12** — the
market's implied σ to two decimals. The price *is* his forecast plus its error
distribution. The numbers live in a **PNG table inside a free Substack post**, which is why
no venue check, title grep or `settlement_sources` scan could ever have found it.

**Kill 3 — fundability.** A raw point-estimate edge does survive post-estimate: legs priced
3-25c at Sunday 20:00 go **2 wins in 47** (4.3% realised vs 11.1% priced, −6.8pp). It is
unreachable. Measured live relative spreads: Odyssey `80-86m` mid 0.0095 / bid 0.002 / ask
0.017 = **1.58**; `92-98m` 1.16; Spider-Man `220-240m` 0.83. The tape is genuinely alive
(85-652 taker trades per leg / 7d, $1.7k-$38.9k flow on every leg) — this is
`tape-gate`'s *other* warning, best edge and worst book correlate. Full `q*/q/q⁻` table at
measured spreads plus the 0.05 fee, per band, per side, three checkpoints: **0 of 18
combinations clear.** Closest is Sat 12:00 / 0.75-0.90, which went **19 for 19** and still
misses (q⁻ 0.832 vs q\* 0.839) — the `arena-rank` 16/16 lesson in a new family.

**Wiki maintenance (I corrected my own entry from yesterday).** On 07-26 I promoted "a hole
in a 12k-series catalogue is the cheapest positive signal we have found" into
`wiki/reference/sharp-line-screen.md`. Today falsified it. Amended that page with a new
section, *"The screen has a blind spot: the counterparty is not always a market"*, carrying
four rules — run "does a specialist publish this free?" as question **one** (third family
now lost to it after golf/DataGolf and MLB/FanGraphs); an empty catalogue slot answers only
"does a *venue* price this?"; look for the forecast in newsletters, forums and podcasts,
not just web pages; and **fit the market's implied σ early — if it is tighter than your data
supports, someone published the number, and that tells you a specialist exists before you
have found them.** Index entry updated to match.

**Environment note.** `web.archive.org` was hard-blocked the entire run — 14 consecutive
connection resets across CDX, timemap and snapshot URLs — while `archive.org/wayback/available`
worked fine. The planned BoxOfficePro vintage pull was impossible; it did not matter, but
do not plan a run around Wayback without testing it first. `boxofficepro.com` also 403s
behind Cloudflare to curl, WebFetch and r.jina.ai alike.

---

## 2026-07-26 (run 5) — Felix: "don't pick markets that are already efficient". No positive idea filed; a $36M family killed on measured gates

Model: **claude-opus-5 (effort max)**.

- **Landscape scan** (20 pages, 25,902 open market rows). Screened for the shapes Felix
  asked for — counting processes, scheduling/capacity, elimination structures, cumulative
  thresholds — and pulled the strongest candidate: Polymarket's **shipping-chokepoint
  transit ladders** resolving on **IMF PortWatch**.
- **Built the gate-0 tool the playbook now demands.** Kalshi's entire catalogue is one
  unauthenticated call: `/trade-api/v2/series?limit=1000` → **12,186 series** carrying
  `settlement_sources`. Per-series `/markets` yields `volume_fp`, `floor_strike` and
  **`expiration_value`** (the exact settled integer); `/candlesticks` yields the price path.
  Promoted to `wiki/reference/sharp-line-screen.md` + index.
- **Gate 0, measured (not described).** Kalshi's `KXHORMUZWEEKLY` declares *our exact
  PortWatch resolution URL*, trades **156k–446k contracts/week** at **1c spreads**, and its
  window-close implied median is **unbiased** against 9 realised settlements: mean error
  **+2.63, se 6.19, t = 0.42**. Same kill as tomatometer, found before any modelling.
- **Cross-venue fallback, also measured.** Translated Kalshi's ≥X step CDF into Polymarket's
  bucket geometry at matched timestamps across the 9 overlapping weeks: Polymarket priced
  the realised winner **+4.6pp** higher (median +1.2pp, se ≈3.8, **t ≈ 1.2**), better in
  6 weeks / worse in 2 / tied 1. Polymarket is if anything the *sharper* venue — no
  harvestable spread in either direction.
- **The venue-independent kill: the resolution feed is not a fixed number.** Pulled the full
  PortWatch ArcGIS layer (2,757 days × 28 chokepoints, 2019→, no key). Comparing settled
  values to the same API today: revisions of **−9% to +247%**. Rebuilding all 19 resolved
  Polymarket boards from the live feed reproduces the **wrong winning bucket on 7 (37%)**.
  Sharpest fact: for the week of May 11–17 **Kalshi settled 15 on May 19, Polymarket
  resolved the 40–59 bucket on May 21, and the feed reads 52 today** — two venues,
  contradictory answers, one week. No vintage archive exists (ArcGIS query endpoints are not
  in Wayback), so the family is **unbacktestable**, not merely efficient. Promoted to
  `wiki/reference/first-print-vintages.md` as a mandatory pre-modelling gate.
- **Also measured, and worth keeping:** the board passes every liquidity screen we own —
  1–2c spreads, 174/125 distinct wallets, ~$28k of 7-day taker flow on **both sides** of the
  leg we would trade, leg-sum 1.019, zero taker fee. Our liquidity gates are working; they
  are not the binding constraint. Screened Bab el-Mandeb (no Kalshi counterpart, cv 7%,
  lag-1 R² 0.075) and rejected it: inherits the vintage kill with **zero** resolved
  instances to measure it on, on a $12k board.
- **Filed `ideas/2026-07-26-chokepoint-transit-ladders-discarded.md`** (`discarded-idea`)
  with all three kills, the live book/tape table, and a cheapest-first revival checklist.
- **Forward pointer, honestly labelled unverified.** Screening all 12,186 Kalshi series
  against our structural families found Kalshi covering essentially everything (RT 244,
  Netflix 25, MrBeast views, GPU prices, metro home values, reality-TV eliminations, chess
  31, quakes 9, by-elections 16, Emmys 30, hurricanes 62). **The one clean hole is domestic
  box office** — 2 hits, both Golden Globe *award* markets. Polymarket's box-office family is
  deep ($17.1M Avatar opener; dozens at $300k–$1.4M; live boards today at $261k/$204k) and
  resolves on The Numbers **final** figures, explicitly *"not studio estimates"* — the
  Tomatometer shape. Recorded as a **lead, not an idea**: gate 0 is BoxOfficePro's long-range
  forecast, which 403s us live but **is archived in Wayback** (verified). Half a day of work,
  and it must be run before anyone spends a slot.
- Memory pruned 201 → 182 lines (run-4 RT detail compressed now that the variant is retired).

---

## 2026-07-26 (run 4) — Felix brief: market-specific data + simulable process. One idea filed

Model: **claude-opus-5 (effort max)**. Brief restated two standing directives: (1) not
market-agnostic — edge must come from a *market-specific* data source; (2) niche boards whose
resolution is the output of a simulable process (queues, brackets, cumulative counts,
path-dependent thresholds). Plus the four kills as pre-screens, now including the new
`midpoint-is-not-a-fill` page.

**Filed: `ideas/2026-07-26-tomatometer-review-arrival.md`** — Polymarket's weekly Rotten
Tomatoes threshold ladders. **The resolution variable is a counting process that is still
running when the market settles**, and the market's whole life sits inside the window in
which the answer is generated (median board lifetime **5.1 days**, n=55).

What I measured, in order:

- **Family census.** 83 RT events found via `public-search` (5 query variants, unioned);
  **67 resolved boards** in the modern threshold format, 60 with usable price history since
  2025-11, **2–4 resolving per Monday**, 4–9 legs each, `negRisk=false`. **Zero coherence
  violations across all 67** resolution patterns. Two open today.
- **The resolution source is fully machine-readable and publishes the raw counts.** RT's
  `<script id="media-scorecard-json">` gives `likedCount / notLikedCount / reviewCount /
  score` plus a separate **Top-Critics** subscore. Plain `curl` + browser UA works. Rounding
  confirmed as nearest-integer on `100·L/N` from six independent triples (67/227 = 29.52 → 30),
  so every strike is an **integer lattice boundary**, not a judgement about the film.
- **The drift, which is the whole idea.** Reconstructed score paths from **Wayback** (54–78
  captures/film, 5–7/day in release week; captures are gzipped originals, must `gzip.decompress`
  before parsing). Embargo→resolution: **mean −4.14, median −2.0, 11 down / 2 flat / 1 up**
  (n=14). Conditional on the denominator: **n<80 → −5.09 (11 films); n≥80 → −0.67 (3)**.
  At the tradeable checkpoint T−72h→T: **mean −2.23, median −2.0, 8 down / 4 flat / 1 up**
  (n=13, median 96 reviews added). The Odyssey went **98 at embargo (n=125) → 95 at settlement
  → 94 today (n=439)** on a board whose top strikes were 95/96/97/98/99.
- **The market does not price it**: interpolated implied median minus displayed RT score at
  the embargo instant = **+0.74 mean, +0.50 median** (n=9). The crowd centres on the number on
  the page.
- **Ladder width is also wrong.** At T−72h (n=57): market LL **0.981** vs uniform-null 1.733;
  modal bucket wins **0.684 ± 0.062** against its own Herfindahl **0.535**, priced 0.641; PIT
  in the outer 20% only **5.3%** vs 20% expected. Distribution too diffuse and centred too high.
- **Checkpoint artifact found and avoided.** At T−14d/T−7d the market **loses to a uniform
  null** (LL 3.575 vs 1.655) with 6/11 monotonicity violations — the boards are listed but
  unpriced. Checkpoint must be **T−96h or later**. Exactly the `checkpoint-artifact` pattern;
  I caught it because I ran the null first this time.
- **Phantom gate passed at family level, failed per-leg.** Only **2/320 legs (0.6%)** never
  moved, median total variation 1.48 — the earthquake-ladder profile. But the live Spider-Man
  `90+` leg quotes a **0.740 Gamma midpoint off a 0.650/0.830 book with $265/$54 depth**. The
  naive ladder read gives that bucket 61.5% of the mass; it is fabricated.
- **Liquidity, measured on the window we would trade.** Last-72h taker notional and how much of
  it sits in 8–92c, ten largest 2026 boards: in-the-grey **$52.0k / $48.8k in band** (93
  wallets), michael $20.6k / $7.6k (297), how-to-make-a-killing $11.3k / $9.8k (33),
  good-luck-have-fun $17.8k / $4.8k (69) — but scream-7 **$33.6k / $37** and mario
  **$12.0k / $19**, because those settled far from every strike and collapsed to 0/1 early.
  **The band is non-empty exactly when the score lands near a strike, which is exactly when our
  edge is largest.** 50/60 boards have ≥1 leg in 10–90c at T−72h, 35/60 have ≥2.
- **Counterparty screen.** No bookmaker prices critic scores. **But Kalshi runs 233 `KXRT*`
  RT series** — a second retail venue, open unauthenticated API. Made it **gate 0**, with the
  explicit note that agreement is *weak* evidence (both crowds read the same page) whereas
  Kalshi pricing the drift while Polymarket does not would be decisive.
- **Speed screen answered structurally**: there is no print to race. The drift is a
  multi-day bias in a continuously-visible number, not news arriving.

Rejected before working up: SpaceX monthly launch-count ladders (family discontinued after
February — only annual boards remain, bad cadence); US measles cumulative-count ladders
(monthly cadence only, and the $7.78M annual board is deep); chess tournament outrights (a few
events/year). Kept the RT candidate because it was the only one with weekly supply, a free
machine-readable feed, a simulable generating process, and no professional counterparty.

Wiki: no new page this run. The RT finding is one market family's mechanism, not
cross-strategy knowledge — it belongs in the idea file until a trial confirms it. Index left
unchanged (14 pages).

---

## 2026-07-25 (run 3) — Felix brief: hunt SIMULATION edge. One idea filed, three candidates killed

Model: **opus-5 (xhigh)**. Brief: find niche markets whose resolution variable is a
path-dependent/combinatorial function we can simulate faster or more faithfully than anyone
pricing it — closed form absent or wrong, cheap public inputs, legs of one simulation traded
as separate books.

- Onboarded on the four screens plus the two wiki pages that landed mid-run from the
  `series-shape/bo3-derivatives` kill (`phantom-midpoints.md`, `sharp-line-screen.md`). Those
  two pages changed the outcome of this cycle — see below.
- Scanned 600 open events by `volume24hr` and censused multi-leg structures. Four candidates
  worked up properly; **three died, and the survivor is the one nobody else prices.**

**Candidate 1 — PGA Tour top-5/10/20 boards. KILLED on the incumbent screen.** Structurally
ideal: four separate 100+-leg boards on one 4-round tournament, resolving on "top N
*including ties*" — an order statistic over 156 correlated integer score paths with a cut, no
closed form, and the standard Harville/Plackett-Luce map is known-biased. 24 tournaments × 3
boards already resolved in 2026. Verified resolution against ESPN's free golf API for the 2026
PGA Championship: **0 false YES, 0 wrongly-NO**; the 13 "missing" top-20 finishers were simply
unlisted (boards carry ~100 of a 156 field and omit marquee names, so the "board sums to k"
test is invalid). Killed because **`datagolf.com/live-model/pga-tour` ships the complete live
Monte Carlo — win/top5/top10/top20/cut per player — as free JSON in the page source**. Same
kill applies to MLB playoff ladders (FanGraphs). Book quality failed independently (10/100
legs ≤5c).

**Candidate 2 — weekly USGS seismicity count ladders. SURVIVED, scoped to window-open.**
Filed as `ideas/2026-07-25-quake-ladder-overdispersion-3.md`. Built the physics from 47,534
M5.0+ USGS events (2000–2026): weekly M5.5+ count has mean 9.458, variance 47.825 →
**Fano 5.06**, i.e. **2.25× wider than Poisson**, with a clean U-shaped bucket error (both
tails ~1.6× too cheap, middle up to 1.6× too rich) mapping directly onto the traded lattice.
Clustering measured: after a M6.5+ the next-day M5.5+ rate is 2.68× baseline (n=1,208); after
a M7.0+, 3.77× (n=396). **Gate 0 reproduced 21/21 M6.5+ boards exactly and 15/20 M5.5+ boards,
every miss off by exactly one** — because 2.01 events/week sit at *exactly* M5.5 and
magnitudes are revised ±0.1 post-hoc, which is one whole bucket (vintages are reconstructible
from ComCat origin products; one event I checked went 5.2 → 5.4 → 5.3). Crowd is **calibrated
on the favourite** at open (0.364 vs Herfindahl 0.366), so this is a pure shape claim. A crude
conditional model beats the market by **+0.110 log-loss at open (se 0.046, 17/22 boards)** and
by ~0 mid-week. **Mid-week is dead and I measured it**: after a qualifying quake lands, the `0`
leg moves −0.279 within one hour of a −0.398 total move (70% in the first hour — the
runningmax pattern), so the idea trades once at window-open and holds.

**Candidate 3 — tennis 14-leg derivative ladder. KILLED, and it is the instructive one.**
Filed as `ideas/2026-07-25-tennis-games-ladder-discarded.md`. Supply was the best I have seen
(10,011 resolved 2026 tennis events, 5,602 full ladders, 101,101 legs, ~48/day). I wrote an
exact point→game→set→match DP and showed a genuine structural result: holding the serve *gap*
fixed and raising the serve *level*, the moneyline moves 4.2pp and P(3 sets) 1.4pp while
**P(total games > 23.5) moves 20.9pp** — the market's deepest books are near-blind to the
parameter that drives the totals ladder. I then measured a −7.6pp Over bias (n=1,676, t=6.4).
**Both of the day's new wiki screens killed it:**
  - *Sharp line*: Pinnacle lists tennis total games as a separate `(Games)` matchup. Across 27
    matched lines on 13 live matches, Polymarket's mean deviation was **+0.32pp (se 0.12),
    27/27 within 3pp; +0.07pp on ≤3c books**. The ladder *is* the Pinnacle line. What looked
    like an untouched book ($27k depth inside 5c, $0 volume) was a mirrored sharp line.
  - *Phantom midpoints*: my headline decomposed to **−27.46pp on dead legs vs −5.00pp on live
    ones**, and **inverted with liquidity** ($0 volume −17.26pp → >$1k volume **+11.78pp**).
    8.5% of legs never moved pre-match; the artifact concentrated in the 0.50–0.60 bucket
    exactly because an empty book reports as ~0.50.
  I ran the phantom gate on the earthquake family as a control: **0 / 314 dead legs, 100% live,
  median total variation 1.79** — so the gate discriminates rather than killing everything,
  which is what makes the filed idea's measurements trustworthy.

- **Order-of-operations mistake I made and am recording**: I ran a three-hour tennis backtest
  and *then* killed it in ten minutes with Pinnacle. The counterparty checks are the cheapest
  kill available and must run first. Promoted to `wiki/market-selection.md` as a new SELECT
  AGAINST bullet with the ordering, plus replication notes appended to
  `wiki/reference/sharp-line-screen.md` (tennis, +0.07pp) and
  `wiki/reference/phantom-midpoints.md` (tennis decomposition + the earthquake control).
- Memory pruned to 147 lines: run-2 esports detail compressed to its post-kill residue, three
  named do-not-re-propose entries added, two new long-term screening rules.
- Honest open item on the filed idea: n=22 boards at t=2.4 is one bad month from noise, and if
  the crowd is right on the tails too, the remaining edge sits in sub-3c legs that fees make
  unfundable. Gate 3 is written to decide exactly that.

---

## 2026-07-25 (run 2) — extra cycle after the arena kill; esports series-shape idea filed

Model: **opus-5 (xhigh)**. Felix requested a second cycle on the day
`arena-rank/satellites` was falsified and rebadged as `arena-rank/favourite-shrinkage`.

- Onboarded on the four screens that now gate idea quality, incl. the new
  `wiki/reference/published-ci-vs-printed.md`, and on my own post-mortem
  (`strategies/arena-rank/satellites/results/backtest-2026-07-25.md`). Two corrections
  internalised: verify the *exact resolving object* and state how; classify the claim
  **level vs shape** before filing. Both are now standing memory entries.
- Fresh scan: 2,000 open events by `volume24hr`. Filtered to multi-leg, fast-resolving,
  non-weather/non-crypto-ladder/non-arena families. Rejected on the spot: company
  market-cap ranking boards (glanceable resolution variable). Parked with reasons:
  non-US central-bank decision boards (rates-desk incumbent), US primary-election winner
  boards (good, but election-calendar cadence). Pursued: **esports BO3 bundles**.
- Built the resolved universe from scratch: **16,959 resolved esports events**
  (Dec-2025 → Jul-2026) via date-windowed offset paging — plain offset paging hard-caps
  at 2,000 and `/events/keyset` ignores `cursor` (the param is `after_cursor`). Yielded
  **6,710 BO3 triples** (`moneyline` + `map_handicap` + `totals`) with known outcomes;
  the three legs are arithmetically consistent in **6,705/6,710** (99.93%).
- Fetched per-leg CLOB history (6,375/6,375 tokens non-empty) and the taker tape for 300
  events. Headline measurements at T−1h: moneyline favourite 0.727→**0.788**; handicap
  0.531→**0.683**; Over-2.5 overpriced in **8/8 cohort-months** (−9.0pp, se 1.75,
  t=−5.16) including the months where the moneyline bias was ≈0. Convex transfer: a
  +5.6pp moneyline error becomes a **+13.8pp** handicap error in the 0.80–0.90 band.
- Ran the kill-screens *before* filing, not after: speed screen (handicap mid moves
  0.519→0.537 across the whole 24h pre-match window vs a 0.683 realised rate — no print
  to race); delayed execution (T−6h signal, T−15m fill, +2c adverse = **+14.7c**, se 2.0);
  midpoint-vs-tape (taker BUY VWAP only +0.68c above mid, so the mid is executable);
  selection-artifact guard (reproduces on a **disjoint** low-derivative-volume sample:
  ML +7.0pp, Over −10.0pp); incumbent test by tournament tier (bias is *larger* on
  tier-1 — fan money, not information).
- Quantified the cost stack properly: Polymarket's **taker fee = shares × rate × p(1−p)**,
  sports rate 0.05 ⇒ ~1.2c/share in the fundable band, makers free. Graduated to
  `wiki/recipes/polymarket-api.md` along with the pagination facts and `sportsMarketType`
  typing — both are cross-strategy, not idea-specific.
- Filed `ideas/2026-07-25-esports-series-shape-2.md` (backlog): explicit **shape** claim,
  all four screens answered upfront with evidence, three live example markets with
  today's book numbers, six gates with numeric kill thresholds, and a pre-registered live
  stop. Honest open item recorded as gate 5: I could **not** read an external bookmaker
  line from this box (hltv.org 403 through the proxy, the-odds-api needs a paid key), and
  that is the cheapest way to kill the whole thesis.
- Standing caution written into the idea: an edge this large on 1c-spread books should
  not exist, so gate 0 is an artifact hunt, not a confirmation.

---

## 2026-07-25 — Netflix killed in research; arena-rank-satellites filed

Model: **opus-5 (max)** — claude-opus-5. First run under the 2026-07-24 routing change
(Opus 5 everywhere, max effort for this role; fable retired).

- Onboarded on both kills + the three screens. Fresh scan (20 pages, 26,668 open market
  rows). Backlog was empty, slot 2 free.
- **Candidate 1, Netflix weekly Top-10 — measured hard, killed before filing.** Structure
  looked close to ideal: 8 boards/week, 243 resolved instances, precise fine print
  (global boards are **"English only"**; US boards resolve on country rank, global on
  views), and — verified — *every* official Netflix publication lands before the market
  opens, with nothing further until the resolving print, so Mon–Tue trades a frozen,
  unpublished outcome. Netflix hands out complete free ground truth
  (`all-weeks-{global,countries}.tsv`, 264 weeks × 94 countries). Measured on resolved
  instances: crowd modal leg underpriced at all 7 checkpoints (Wed 0.651→won 0.750;
  Mon-frozen 0.920→won 0.971, n=102), field legs in the 3–50c zone priced 0.08–0.12 vs
  0.02–0.07 realized, taker flow ~$160k/wk arriving 71–88% in the frozen window,
  "Other" wins 4.5%. Two independent kill findings: (a) the executable side is missing —
  bids are $0–96 top-of-book while tail *asks* are deep, so the sell-the-field direction
  cannot be filled; (b) decisive — a decay model fitted on Netflix's own 264 weeks scores
  **23% (shows) / 42% (films) argmax vs the market's 77% / 83%** at Thursday. The crowd
  has an observation channel we don't (in-app daily Top 10; FlixPatrol 403 from this box).
  Same shape as the gistemp kill, caught for one day of research instead of a slot.
- **Candidate 2, filed: `ideas/2026-07-25-arena-rank-satellites.md`.** Monthly
  arena.ai/LMArena family — 7+ boards resolving off ONE Text-Arena Rank column read at
  ONE instant, liquidity spanning 250× ($30,299 WebDev → $7,587,062 #1-overall), spreads
  spanning 37×. Thesis inverts the deep-book rule: the efficient $7.6M board is a free
  sharp anchor on the same latent ranking that seven thin satellites price separately.
  Two mechanical mispricings: company boards are order statistics over *portfolios*
  (Anthropic holds ranks 1,2,3,4,6 today), and the Rank column is an estimate whose
  publisher stamps every row with ±CI, vote count, Preliminary flag and an explicit Rank
  Spread (rank 1 → "1–5", rank 10 → "4–27"). Screen 2 passes structurally: the upstream
  is a private vote stream, so *nobody* can hold a better feed — the published table is
  the primary for every participant, and a cache-busted fetch confirmed we hold the same
  Jul-21 vintage. Live incoherence evidence: WebDev-Aug legs sum 1.222 vs Math-Aug 0.934;
  Moonshot priced 0.001 / 0.017 / 0.182 / 0.662 across four boards on one ranking; the
  $655k Chinese board has Alibaba 0.786 vs Moonshot 0.182 while the table has Moonshot
  11 points ahead. 6 kill gates incl. gate-0 resolution reproduction, refresh-lifetime
  speed gate, paired log-loss, a portfolio-effect gate, t+24h delayed exec, capacity.
  Whole July cohort checks 2026-07-31 → a trial scores in six days, then monthly.
- Screen 3 has real teeth here and is written into the file: the leaderboard is
  *recomputed retroactively* at each refresh, so today's table resolved nothing; Wayback
  has 500 captures 2025-05-28→2026-01-28 and **zero after**, so Feb–Jul 2026 instances
  are unscoreable and we must archive the live table ourselves from day 1.
- Wiki maintenance: new SELECT AGAINST bullet in `wiki/market-selection.md` —
  **"glanceable within-window state"**, the sibling of proxy-vs-primary, earned by the
  Netflix numbers, with the corollary on where to look instead (state hidden from the
  amateur too). Memory pruned and restructured; API gotchas recorded (prices-history
  `interval=max` 30-day cap, placeholder legs pinned at 0.500, Wayback CDX needs https).

---

## 2026-07-24 — Felix directive: market-specific data sources; gistemp idea filed

Model: fable (high). Governing input: inbox
`2026-07-23-felix-market-specific-strategies.md` (now status: done).

- Fresh scan (20 pages, 26,614 open market rows, frozen:
  `data/scans/2026-07-24-events-vol24.csv.r2.json`). Grepped for primary-source
  topics: climate/GISTEMP, hurricanes/NHC, CDC counts, CPI/BLS, EIA gas, Netflix/
  Spotify/Billboard weeklies, earthquakes/USGS, box office. Strongest data-advantage
  candidate by far: the GISTEMP monthly bucket family (poly's literal heritage).
- Verified both primary sources read-only from this box and froze snapshots to R2
  (`data/sources/2026-07-24-{gistemp-glb,era5-daily-2t-global}.csv.r2.json`): GISTEMP
  LOTI monthly CSV (Jun 2026 = 1.18) and ECMWF Climate Pulse ERA5 daily global 2t
  (through Jul 21, updated Jul 23).
- Built the actual nowcast live: ERA5 July MTD(21d) +0.632, persistence-projected
  full month 0.629±0.032; June-anchored + year-effect transfer → GISTEMP nowcast
  center ~1.246 σ 0.056 (30-month day-21 hindcast: MAE 0.052, bias +0.025, only 6/30
  within ±0.025). Market prices modal 1.20–1.24 bucket 72/83 vs model 0.31–0.37;
  adjacent buckets 4.9c/20c asks vs model 0.15–0.24/0.25–0.32; $108k ranking sibling
  bids 0.94 on "1st hottest" vs model 0.68–0.82. Confirmed the family trades past
  month-end until the print (Jul-2025 closedTime Aug 8) → 8–19 day model-revealed
  edge horizon, no print to race; speed screen passes by construction.
- Filed `ideas/2026-07-24-gistemp-monthly-nowcast.md` (backlog): 5 kill conditions
  incl. market-already-sharp test, modal-calibration test, t+24h delayed-exec sim,
  GISTEMP first-print vintage check, capacity floor. Replied to Felix (status: done).
- Wiki maintenance: added Gamma `/public-search` series-discovery recipe to
  `wiki/recipes/polymarket-api.md`. Memory pruned (run-2 detail compressed; climate
  family facts + "start from the resolution source" heuristic added).

---

## 2026-07-23 (run 2) — extra cycle after same-day kill; idea 2 filed

Model: fable (high). Felix requested a second cycle after runningmax was killed day 1.

- Onboarded on the kill: `wiki/reference/delayed-execution-test.md` (new),
  market-selection's new SELECT AGAINST (speed-race mispricings), runningmax memory +
  `results/backtest-2026-07-23.md`. Takeaway operationalized: speed screen now goes
  *in the idea file, upfront*.
- Re-mined the frozen 08:11Z scan (pulled from R2, sha-verified) for the three unfiled
  candidates. Dropped (a) generic negRisk dead-leg sweeping (bot-harvested premium in
  weather; brackets show hours-long windows but dust-sized books). Left (c) esports
  unfiled. Pursued (b): "Hit Price" one-touch ladders — found the family is 3-tier
  (daily/weekly/monthly) across ~25 assets incl. equities+commodities, with weekly
  boards squarely thin-to-mid ($5–80k), not just the deep BTC annuals memory recalled.
- Fresh probes (Gamma + CLOB books + Pyth Hermes, 09:15–09:20Z): live monotonicity
  violations on SPY/NVDA weekly LOW ladders persisting ≥65 min unchanged (vs 0–3 min
  weather collapse); WTI July board implies 53% ATM → 88% wing touch-vol smile;
  discovered the extension-leg trap — strikes added mid-window carry private
  `startDate` windows ("after market creation"), e.g. monthly WTI $80-LOW created
  Jul 20 16:30Z = re-touch claim at 0.25/0.26 while the weekly $80-LOW sits at 1.0.
  Tail top-of-book is dust ($3–20); depth is real in the 3–50c zone of monthlies.
- Idea filed: `ideas/2026-07-23-hit-price-ladder-rv.md` (status: backlog) — IV-anchored
  one-touch relative value + fine-print windows; speed screen passed upfront (edge is
  model-revealed, harvested over 1–9d holds; post-touch convergence explicitly
  excluded); kill gates include measured violation-lifetime screen and t+24h
  delayed-execution sim. Resolved supply verified: WTI May $40.2M/30 legs, BTC June
  $25.2M, SPY+NVDA week-of-Jul-13 boards closed.

---

## 2026-07-23 — first scan, backlog seeded

Model: fable (high). First run since founding; backlog was empty.

- Built `tools/scan/` (stable-rust crate, not cargo -Zscript — nightly absent from this
  box): pages Gamma `/events`, flattens to CSV, prints horizon/volume/tag summary.
- Scanned top 2000 open events by 24h volume -> 26,539 open market rows. Freeze:
  `roles/market-researcher/data/scans/2026-07-23-events-vol24.csv.r2.json` (11MB -> R2).
- Landscape: Sports 9.8k mkts / Politics $2.4B vol; 87% of open markets <$10k volume;
  ~11k markets resolve <=7d. Recurring series = crypto (deep, avoid) + 49-city daily
  temperature bucket families (~$3.2M open, resolved instances back to April).
- Probed temp families live: Seoul post-peak fully converged (0.9995); Hong Kong 16:40
  local still bid 0.016 on a near-dead 34°C leg vs 5h of 33°C METAR prints; London
  pre-peak with genuine spread (26°C at 0.50). Resolution stations differ per city (HKO
  vs Wunderground airport pages) — structural fine print confirmed in descriptions.
- Idea filed: `ideas/2026-07-23-temp-daily-max-truncation-lag.md` (status: backlog) —
  intraday truncation repricing in daily temp families; falsification = window-open
  Herfindahl test + dead-leg collapse-time backtest + truncated-model log-loss vs
  de-vigged mids, on months of resolved city-days.
