# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

**Where knowledge lives:** durable method → `wiki/`; the object-by-object record and its
counterparties → `ops/idea-funnel.md`. **Do not restate either here.** This file holds only
what is live, unfinished, or not yet generalisable.

## Short-term

- 2026-08-01 (run 11). Killed **deep-tail carry** — buying NO at the ask on standalone binaries
  with a YES mid ≤5c — `ideas/2026-08-01-deep-tail-carry-discarded.md`. **The closest object
  yet: cleared W2 AND W3 outright, and the best point estimate the firm has taken.** Died on
  W1's *direction* and on W4 at the horizon the venue actually offers.
- **The reframe worth keeping (now a wiki page).** For any "collect a premium unless a rare
  thing happens" object, walls 3 and 4 are **one** calculation:
  `π* = 1 − a_eff·(1 + r·d/365)` is the carry hurdle *and* the statistical null; `3/π*` is a
  floor on draws **assuming you never lose once**; and `d_max = 365(1−a_eff)/(a_eff·r)` kills
  some legs on arithmetic alone (0.47c/0.52c legs with **152 days to run vs a 44-day d_max**).
  *The safer the leg looks, the smaller π\*, the BIGGER the sample* — that inversion is the trap.
- **THE BACKTEST, run not proposed:** sampled 356 settled standalone binaries at a T−45d
  checkpoint; **169 (47.5%) were ≤5c and 0 resolved YES**, mean checkpoint 2.46c →
  **+12.97pp over risk-free at executable prices**. And it still fails: Wilson upper **2.22%**
  (169 raw legs) / **3.05%** (122 themes) vs π\* **1.57%** at 45d and **0.44%** at 150d.
  A perfect 0-for-169 on rare events is not enough. One YES → 3.28%/4.50%.
- **I GOT W3 WRONG FIRST AND MUST NOT REPEAT IT.** Counting the *live* cohort (127 legs → ≤70
  themes, 94 on 2026-12-31) gave "4–22 years, dead". Wrong denominator — the open book is a
  snapshot, not a rate. The **settled** census: 99,493 closed events → 36,969 standalone
  binaries → **3,280 non-sport/crypto/weather in 12 months (~273/mo)**, 47.5% in-band at T−45d,
  deflator only **1.39×** (settled panels lack the year-end calendar artifact) →
  **~1,558 band legs / ~1,120 quasi-independent themes per year** vs 243 needed. **Take arrival
  rates off the settlement record, always.**
- **The escape route that closed it:** π\* rises as horizon shortens, so "trade the short end" —
  except ≤45d holds **0.5%** of band volume and is *safer* (1.95c, π\* 0.72%); 46–90d peaks at
  π\* 1.83% with **1.6%** of volume; **≥150d holds 97.8% at π\* ≤0.75%**. Weight π\* buckets by
  **volume**, not by leg count.
- **W1, and the direction is the lesson:** Kalshi **12,368 series** (+13/day). Live twins with
  real size this time — `KXALIENS-27` **26.3M** contracts, `KXTRUMPOUT27-27-DJT` 5.60M/2.40M OI,
  `KXPAHLAVIHEAD` 1.80M, `KXNEWOUTBREAKHANTA` 1.63M, `KXGREENLAND-29-27` 1.47M. **8/8 matched
  pairs ABOVE us, mean +1.52pp, median +1.57pp, sign test p=0.0039.** When the thesis is "this
  venue is too high/low", the incumbent screen tests the **sign**, not the size.
  Definitional-noise floor read off Kalshi disagreeing with *itself*: two Greenland series 0.95pp
  apart. And Kalshi pays interest on collateral while PM USDC does not, so PM's tail should be
  the *dearer* one — the carry adjustment widened the gap against the thesis.
- **W2 PASS, the number to remember:** \$19,352,132 at the NO ask (Hantavirus), \$16.1M Taiwan,
  \$15.3M Xi, \$5.4M Jesus, **VWAP flat to \$10,000** on all five, vs object 13's median **\$7**.
  Median spread 0.60c = 20.0% of the YES leg but **0.62% of the NO leg**.
- **W4 marginal:** the safest, deepest leg on the whole venue (Second Coming, \$64.9M) pays
  **4.42% net annualised vs ~4% risk-free = +0.42pp**; moon landing −2.92pp, 10.0 quake −2.80pp.
  Everything paying more is paying for real event risk — the tail yield curve is a **risk curve**.
- **Scanned + set aside on W3 alone, free, do not re-derive:** `titled-tuesday` chess placement
  ladders (66–75 legs, weekly Swiss, sim-tractable from Elo, state NOT glanceable — but **1
  settled instance ever**, 07-28; revisit ~2027 if it survives); `h100/h200/b200-rental-price`
  monthly GPU ladders (~\$70–98k/board, no plausible incumbent — 3 correlated series, ~2 months
  of history); `monthly-listeners` (Spotify, 3 artists); `movie-delay`; 435 US House race boards
  (Kalshi + free forecasters); Setka Cup table tennis and the obscure-soccer/cricket series
  (Pinnacle). Polymarket `series` slugs are the cheap way to enumerate recurring families.
- Runs 9–10 (07-30 cumulative by-date ladders, 07-31 nested-board dominance): outcomes and
  numbers now live in `ops/idea-funnel.md` rows 14–15 and the wiki. Only the reusable residue
  is kept below.
- **CENSUS TRAPS, all three cost me a wrong number — check every time.** (a) Dormant boards rank
  LAST by `volume24hr`, so the volume-ranked scan found **4 of 85** races; offset paging caps at
  2,000 → **date-windowed paging is mandatory** (1,600-event scan vs **9,347 events / 96,642
  markets** on 08-01). (b) Party boards carry `Will A win…`, single capital letters a
  `Person [A-Z]` filter misses — gate on `volumeNum==0 && liquidityNum==0`, never on the name
  pattern (first pass said 0 party boards; truth 81). (c) **08-01: the open book is a snapshot,
  not a rate — arrival rates come off the SETTLED record** (live cohort said ≤70 themes/cohort,
  settled census said ~1,120/yr).
- Polymarket's `Earn 4%` tag is **not** a venue yield on idle USDC — it is maker liquidity
  rewards. The carry comparator is external (T-bills). Kalshi's, however, *is* real collateral
  interest, and it belongs in any cross-venue carry comparison.
- **The depth wall has a SHAPE, now bounded by two clears:** it needs a *mode*. A cumulative
  ladder has none (unquoted legs are the already-decided ones, \$264 at the bid); a standalone
  binary has no board at all (\$19.4M, zero slippage). `depth-lives-where-the-edge-is-not.md`.
- **`KXGPT5RELEASE`, `KXGEMINI3`, `KXMYTHOS`, `KXCLAUDE4`, `KXO3RELEASE`, `KXGROK4` are ALL
  0-market shells** while `KXGPT`/`KXCLAUDE`/`KXGEMINI` carry 3.3M contracts — Kalshi rolls
  successive objects through ONE vendor-generic series. **Search by vendor/venue/person/
  franchise and sort by `volume_fp`.** Confirmed again 08-01 (`KXALIENS-27`, 26.3M).
- **Metaculus API is authenticated-only as of 07-30** ("Permission Error"). A free specialist we
  can no longer measure cheaply — public HTML pages remain.
- **Scanned + rejected, don't re-derive:** Fed decision/cut/hike and all non-US central-bank
  decisions (rates desks on OIS); `largest-company-end-of-*`, `*-valuation-hit-__`, and all
  crypto/gold "hit __ by" (quoted-price underlying → Felix's standing instruction);
  Michigan/US primaries (Kalshi 175 primary series); box-office boards.
  **95 of 185 live by-date ladders are war-adjacent** — Iran/Hormuz/ceasefire is still the bulk
  of top non-sports tape, still blocked pending Felix's ruling, still materially thinning what
  I can scan.

## Medium-term

- **THREE families now (07-28, 07-29, 08-01) died taker-side only; the one untested construction
  in all three is MAKER-side** (Polymarket charges no fee on resting orders). §5 forbids
  executing, not researching. Open question with Felix via the funnel — do not spend a slot on
  it unheard. Note for honesty: on 08-01 maker-side would **not** have changed the verdict, since
  the draw count is a property of the world rather than of the order type.
- Wiki overlap `nested-ladders-trade-depth-for-power.md` ↔ `nested-ladders-are-one-draw.md`:
  **checked 08-01, no drift**, ownership split still stated in both. Don't re-check every run.
- **Scrapers/gotchas kept from dead families.** RT: `rottentomatoes.com/m/<slug>`, `<script
  id="media-scorecard-json">`, browser UA. The Numbers: `/box-office-chart/weekend/YYYY/MM/DD`
  — estimates carry `class="chart_estimate"` and round to $50k while finals are exact = a
  machine-readable provisional-vs-final detector. `web.archive.org` hard-blocked 07-27 (14
  resets) while `archive.org/wayback/available` worked — test archive deps before planning.
- Scanned + rejected earlier, don't re-derive: SpaceX launch counts (cadence); measles ladders
  (thin); chess outrights (Kalshi 31); flu weeklies (off-season); music charts (kworb glanceable);
  IPO/acquisition boards (insider); Netflix Top-10 (in-app list); NSIDC sea-ice (Kalshi 8 + SIPN
  free ensemble); VEI-6 volcano; Cat-4 landfall; EIA/AAA gas (Kalshi 35); CDC counts. Weather
  city-dailies: bot-patrolled intraday, only pre-day/forecast angles left. "Hit Price" one-touch
  is trialing as barrier-touch/ladder-rv — don't re-propose.
- **KILLED, do not re-propose** (details in `ops/idea-funnel.md`): PGA/MLB ladders
  (DataGolf/FanGraphs free); tennis derivative ladders; earthquake ladders; esports BO3
  derivatives; RT/Tomatometer ladders; IMF PortWatch chokepoint boards; box-office boards;
  "will X say WORD" mention boards; post/tweet-count ladders; cumulative by-date ladders.
- Scan tooling: `roles/market-researcher/tools/scan/` (Gamma /events → CSV), order `volume24hr`
  for "alive today", ~1 min. Series discovery: `/public-search?q=<text>&limit_per_type=50` —
  **vary the wording and union the slugs**; then `/events?slug=` **and** `&closed=true`.
  **Best family-enumeration trick (08-01): the `series[].slug` field on `/events`** — 680
  distinct series over the open universe, which surfaces dormant recurring families (GPU rental,
  Titled Tuesday, Spotify listeners) that no title grep or volume ranking would show.
- Landscape shape (stable 07-23→08-01): Sports ~8k mkts dominates count; Politics/Elections
  dominate volume; ~86% of open markets <$10k. Kalshi catalogue **12,368 series** (12,355 on
  07-31, 12,329 on 07-30, ~+13–26/day). Coverage map in `wiki/reference/sharp-line-screen.md`;
  read it as "where the cheap kill is available", never "where nobody is looking".

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — `/wiki/market-selection.md`. Deep books are efficient; calibrated
  recurring crowds are efficient at window-open.
- **Classify every idea LEVEL vs SHAPE before filing, and say which.** Level = "we estimate the
  truth better" — died five times (runningmax, gistemp, box office, mentions, and the GPT-6
  cross-venue gap). Shape = "the crowd's own distribution is mis-allocated" — survived twice
  (ladder-rv wings, favourite-shrinkage), died twice (post-counts 07-29, date ladders 07-30).
  A shape claim passes proxy-vs-primary and glanceable-state *by construction*, so its risk is
  entirely **execution cost, regime stability, and now sample size** — check all three first.
- **Screen ordering, MEASURED not described:** (1) Kalshi catalogue — check
  `settlement_sources`, compare **bucket cuts** AND **rule text**, and search by *vendor* not
  object. (2) Does a specialist publish the simulation free — read the PAGE SOURCE, check
  newsletters/forums, and try Manifold as a cheap third quote. (3) Fit the implied σ.
  (4) **Count independent DRAWS, compute required n, divide by arrival rate** — before any
  modelling; this is the cheapest kill I have added and it needs no data. **For a DOMINANCE
  object substitute the opportunity count: n=1 suffices for truth, so count live instances and
  the arrival rate of new ones instead.** **For a PREMIUM-COLLECTING object substitute the
  break-even event rate `π* = 1 − a_eff·(1+r·d/365)` and look up `3/π*`.** (4b) On any Σ=N board compute **K·s̄/2** before
  anything else — arithmetic, one book fetch. (5) **Walk the live book at the price band and
  size the rule would buy**, report VWAP not top-of-book; on a BASKET the **thinnest leg sizes
  all K legs**. (6) Price both sides at executable prices. (7) **Divide by the capital lock-up
  and compare to the risk-free rate** — a guaranteed profit is not an edge until it clears the
  hurdle (France: +0.35%/yr over 9 months). (8) Rebuild ≥3 settled instances from the feed.
- **Report the bound, never the point estimate.** 07-30's −11.12pp at t=−2.15 with the mirror
  test passing still died because Wilson upper 0.3455 > break-even 0.2786. A monotone gradient
  in the pre-registered direction is not a pass. `wiki/reference/break-even-win-rate.md`.
- **Decompose a surviving band by leg TYPE, and leave-one-family-out.** Wing buckets aren't
  comparable to interior buckets at the same price (07-29: wings carried 100% of a +3.28pp edge,
  747 interior legs none). 07-30: dropping rocket launches took t from −2.15 to −1.45.
- **The walls, in the order they now fire:** (1) someone already prices it — 10 of 16 objects;
  (2) taker-side reachability (spread 07-28, leg depth 07-29, basket-thinnest-leg 07-31) —
  **has a pass state and a shape: it needs a MODE**, so it does not bite a modeless ladder
  (07-30) or a standalone binary (08-01, \$19.4M and zero slippage); (3) **statistical power /
  draw count (07-30)**, or for a dominance object its analogue the **opportunity count**
  (07-31) — **first cleared outright 08-01**; (4) **carry — the hurdle rate on locked capital
  (07-31)**. **08-01: on any premium-collecting object, walls 3 and 4 are the SAME
  calculation** — state the hurdle as a break-even *event rate* π\*, weight π\* by **volume**
  across horizon buckets, and `3/π*` is a floor on draws that nothing can get under.
  Absence of a counterparty removes our cheapest *check*, it does not create edge — 4 objects
  "none found", 3 dead. Full analysis in `ops/idea-funnel.md`.
- **KILLED 08-01, do not re-propose:** fading the ≤5c standalone-binary tail (novelty/lottery
  legs, "nothing ever happens" premium) in any taker-side form. The band is reachable in size
  and still unprovable — and Kalshi prices it *higher* than we do.
- **KILLED 07-31, do not re-propose:** primary⊂general dominance pairs, Σ=N basket coherence,
  cross-board leg-sum arbitrage — the venue is coherent to within its own spread, and negRisk
  mechanically enforces Σ=1 so only N>1 boards can drift.
- **Does the crowd have an observation channel our pipeline lacks?** (GISTEMP inputs; Netflix
  in-app list; METAR.) Mirror failure: **if the state is hidden from EVERYONE that is not edge
  either** — irreducible noise, and a wide crowd distribution is correct (Hormuz ±18.6 ships).
  Edge needs the state *recoverable by work*.
- **A stale bucket cut is a real mispricing and a worthless one** — +15.89pp, one regime, fixed
  the moment the venue re-cuts, leg unfillable. Check whether a rival has re-cut: that dates it.
- API gotchas live in `wiki/recipes/polymarket-api.md` — read it, don't re-derive. The two that
  cost me time: `prices-history` returns **empty, not truncated**, for windows >~14d (chunk at
  ≤14d; this looked exactly like "unbacktestable" and would have been a wrong kill), and per-leg
  deadlines come from `endDate`, never the question text. Fees: `mentions_fees` 0.04,
  `culture_fees` 0.05, movies 0.05, geopolitics 0; CLOB reports 1000bps maker/taker base on
  AI/politics boards — **unresolved, don't quote it as 10%**; Kalshi's own is `0.07·p·(1−p)`.
