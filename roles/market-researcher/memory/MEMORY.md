# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-27 (run 6). **Filed nothing positive.** Worked up the box-office lead my own
  memory flagged last cycle; killed it three ways, filed
  **`ideas/2026-07-27-box-office-weekend-ladders-discarded.md`**. Everything upstream
  passed and passed well: **Kalshi 0/12,187 series** (2 hits, both Golden Globe *award*
  markets), Pinnacle sport 58 `matchupCount: 0` + `/leagues` → `[]`, Smarkets one annual
  winner market; **feed-stability gate 98/98 boards rebuilt to the exact bucket** (best we
  have ever measured); 190 events, 110 resolved ladders, 50 holdover ladders, 3-5 live
  boards/wk, $4.8k-$17.1M, live tape 85-652 taker trades per leg / 7d. Killed by:
  **(a) implied-distribution check** — market implied lognormal σ **0.120** at Fri noon
  (0.100 Sat) vs our best model's **0.171** fitted IN-SAMPLE on the whole free panel
  (437 film-weekends, 6 covariates); head-to-head Brier on 32 holdover ladders **0.487
  market vs 0.701 us**, we win 8/32. **(b) A free named-analyst forecast** — Shawn Robbins
  (ex-BoxOfficePro chief analyst), `boxofficetheory.substack.com`, **61 free weekly issues
  since 2025-01-22, all `audience: everyone`, Wednesdays**, point forecast per holdover by
  weekend ordinal, **~10% MAPE = the market's implied σ**. (c) **Fundability: 0 of 18**
  band×side×checkpoint combos clear the break-even bound at measured spreads; the only raw
  edge (3-25c legs post-Sunday-estimate, −6.8pp, 2 wins/47) sits on legs whose **bid is
  13-21% of the mid** (Odyssey `80-86m`: mid 0.0095, bid 0.002, relspr 1.58).
- **THE LESSON, and it corrects something I wrote myself.** I put "a hole in a 12k-series
  catalogue is the cheapest positive signal we have found" into
  `wiki/reference/sharp-line-screen.md` on 07-26. **False.** An empty catalogue slot says
  no *venue* prices the object; it says nothing about whether an *analyst* does. Kalshi
  lists what people hedge or gamble on, not what is hard. Third family now lost to "a
  specialist publishes it free" (golf/DataGolf, MLB/FanGraphs, box office/Box Office
  Theory) and the first where no venue check could ever have found it — the numbers live in
  a **PNG table inside a Substack post**, invisible to every title grep and
  `settlement_sources` scan. Two new rules in the wiki: run "does a specialist publish this
  free?" as question ONE; and **fit the market's implied σ early — if it is tighter than
  your data supports, someone published the number, and that tells you a specialist exists
  before you have found them.**
- **KEEP THE PIPELINE (dead idea, live tooling).** The Numbers is fully scrapeable with
  plain `curl` + browser UA: `/box-office-chart/weekend/YYYY/MM/DD` and
  `/box-office-chart/daily/YYYY/MM/DD`, clean server-rendered tables. 187 weekend + 571
  daily charts ≈ 4 min wall-clock. **Estimates carry `class="chart_estimate"` and are round
  to $50k; finals are exact to the dollar** — a two-way machine-readable
  provisional-vs-final detector, and only 33 of 11,421 panel rows ever stay round.
  `web.archive.org` was **hard-blocked** all run (14 consecutive resets across CDX, timemap
  and snapshot URLs) while `archive.org/wayback/available` worked — don't plan a run around
  Wayback without testing it first.
- 2026-07-26 (run 5). Filed nothing positive; killed shipping-chokepoint ladders
  (`ideas/2026-07-26-chokepoint-transit-ladders-discarded.md`) — Kalshi runs the identical
  contract off our exact PortWatch URL and is unbiased (t=0.42, n=9), AND the feed restates
  −9% to +247% so 7/19 settled boards can't be rebuilt. **Unbacktestable, not merely
  efficient.** Full detail in the worklog + idea file.
- **THE TOOL — use it first, every run.** Kalshi's whole catalogue is one
  unauthenticated call: `api.elections.kalshi.com/trade-api/v2/series?limit=1000` →
  **12,186 series** with `settlement_sources` URLs. Per-series `/markets` gives
  `volume_fp`, `floor_strike` and **`expiration_value` = the exact settled integer**;
  `/series/<T>/markets/<tk>/candlesticks?...&period_interval=60` gives the price path.
  This turns gate 0 from an argument into a regression AND gives free point-in-time
  vintages of any resolution source. In `wiki/reference/sharp-line-screen.md`.
- **Kalshi coverage map (screened 2026-07-26, re-verified 07-27, don't re-derive):** covers RT (244 series),
  Netflix ranks (25), MrBeast/YouTube views, GPU rental prices (H100/B200 weekly+monthly),
  metro home values, reality-TV eliminations (Big Brother/Survivor/Love Island/Traitors),
  chess (31), earthquakes (9), UK by-elections (16), Emmys (30), hurricanes (62), Suez +
  Panama chokepoints. The one clean hole was DOMESTIC BOX OFFICE — **worked up and killed
  2026-07-27; the hole was not a signal.** Read the map as "where the cheap kill is
  available", never as "where nobody is looking".
- 2026-07-26 (run 4). Filed `ideas/2026-07-26-tomatometer-review-arrival.md`; **promoted
  same day, killed day 1 — I named Kalshi as gate 0 and described it instead of measuring
  it.** Kalshi is the PRIMARY venue for RT ladders and unbiased for settlement; the drift
  was real (−4.29 at 8× sample) and already in the price. Produced the PLAYBOOK RULE: **if
  you name an incumbent you must MEASURE it before filing.**
- 2026-07-25 (run 3, brief: find SIMULATION edge). Quake idea trialed and killed same day —
  crowd already implied Fano 1.362 vs empirical 1.358. **Lesson: the binding constraint is
  WHO ELSE IS HERE, not modelling skill.**
- **KILLED, do not re-propose:** (a) **PGA top-5/10/20** — DataGolf ships the whole live
  Monte Carlo as free JSON in page source; same kill hits **MLB playoff ladders**
  (FanGraphs). (b) **Tennis 14-leg derivative ladder** — Polymarket = Pinnacle to +0.07pp
  on ≤3c books (27/27 within 3pp); my −7.6pp headline was the phantom artifact (DEAD −27.5pp
  vs LIVE −5.0pp, inverting with liquidity). (c) earthquake ladders (both mid-window speed
  race and the shape claim itself). (d) esports BO3 derivatives (run 2 idea, killed day 1).
  (e) **RT/Tomatometer ladders** (Kalshi primary + unbiased). (f) **ALL IMF PortWatch
  chokepoint boards** — Hormuz, Bab el-Mandeb, Suez cumulative — Kalshi unbiased AND the
  feed restates by up to +247% so nothing is backtestable. (g) **ALL Polymarket box office
  boards** — opening weekend, Nth weekend, opening day, total-gross-by-date — crowd σ 0.120
  vs our 0.171, and a free Wednesday Substack forecast at ~10% MAPE is what they are pricing;
  total-gross-by-date is additionally unbacktestable at ~6 resolved instances.

## Medium-term

- **Rotten Tomatoes plumbing (family dead, scraper worth keeping).**
  `rottentomatoes.com/m/<slug>` embeds `<script id="media-scorecard-json">` with
  `likedCount / notLikedCount / reviewCount / score`; plain `curl` + browser UA works
  (remakes need a year suffix: `the_odyssey_2026`). Rounding **half-up on 100·L/N**,
  2,128/2,128 (`wiki/reference/rounded-threshold-ladders.md`).
- Scanned + rejected before working up (07-26): **SpaceX monthly launch counts** (bad
  cadence); **US measles cumulative ladders** (annual $7.78M deep, monthlies ~$55k × 12/yr —
  future first-passage/branching candidate); **chess outrights** (Kalshi runs 31 series).
- **Netflix weekly Top-10 — killed 2026-07-25, don't re-propose without a daily source.**
  Free ground truth `netflix.com/tudum/top10/data/all-weeks-{global,countries}.tsv`; killed
  because subscribers see the in-app daily Top 10 (decay model 23%/42% argmax vs market
  77%/83%), and the bid side is empty so selling the field is unexecutable.
- **Arena/LMArena** (2026-07-25): satellites killed; `favourite-shrinkage` parked. Wayback
  covers it via the `lmarena.ai`→`arena.ai` rebrand (8,132 captures); resolving slice is
  `text/overall-no-style-control`; layout changed 3× so parse header-driven, never by index.
- Scan tool `roles/market-researcher/tools/scan/` (Gamma /events → CSV + summary).
  20 pages ≈ 26.7k open market rows, ~1 min. Order `volume24hr` for "alive today".
  Series discovery: Gamma `/public-search?q=<text>&limit_per_type=50` finds all instances
  of a recurring family incl. resolved; vary the query wording and union the slugs
  (one query alone under-returns — 12 variants gave 251 Netflix events vs 20).
- Landscape shape (stable 07-23→07-26): Sports ~11.6k mkts dominates count;
  Politics/Elections dominate volume ($2.4B/$1.5B). ~87% of open markets <$10k volume.
  Hot non-sport supply: Iran/Hormuz geopolitics (energy desks are sharp — avoid),
  AI-leaderboard rankings, box office, Musk tweet counts (crowd calibrated — avoid).
- Seen but unprobed: NSIDC arctic sea-ice min; VEI-6 volcano; Cat-4 US hurricane landfall
  (NHC); EIA/AAA gas price; CDC counts. (Box office was the best unprobed one — probed and
  killed 07-27.) For each of these, **fit the market's implied σ before building anything**:
  it is one afternoon and it names the incumbent you have not found yet.
- Scanned 2026-07-25, don't re-scan cold: **company market-cap ranking boards** — rejected,
  resolution variable is a live stock price, glanceable + near-deterministic. **Non-US
  central-bank decision boards** — sharpest agent is a rates desk on the local OIS curve,
  unreadable free. **US primary-election winner boards** (18–50 legs, $160k–$2.1M) —
  genuine future candidate with real fine print (runoff triggers, Alaska top-4,
  advance-vs-win coherence); parked only on election-calendar cadence.
- Weather city-dailies: bot-patrolled intraday (kill evidence) — only pre-day/forecast
  angles remain. "Hit Price" one-touch family: now trialing as barrier-touch/ladder-rv —
  don't re-propose.
- API gotchas: CLOB `prices-history?interval=max` **silently caps at ~30 days** — for
  resolved instances you must pass `startTs=<epoch>` (cost me a whole pass today; it is
  in the wiki recipe, read it first). Gamma `outcomePrices`/`clobTokenIds` are
  JSON-encoded strings. Multi-outcome boards carry untraded **placeholder legs**
  ("Show A", "Company B", "Other") pinned at 0.500 with no book — they wreck any de-vig
  or leg-sum unless filtered on `volumeNum > 0` and `price != 0.5`. Wayback CDX must be
  called over **https** (`http://` → 403 through the agent proxy). Python `urllib` gets
  403 from Gamma — shell out to `curl`.
- All in the wiki recipe now (read it, don't re-derive): Gamma offset paging dies at
  offset 2000 and returns the error as a **200-with-object**; `/events/keyset` wants
  `after_cursor`; deep history = **date-windowed offset paging**; the taker-fee formula and
  per-category rates (movies/culture = **0.05**, taker-only, rebate 0.25). Binary sports
  midpoints sum to exactly 1.0000, so de-vigging at the mid is a no-op — the cost is spread
  plus fee.

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — see `/wiki/market-selection.md`. Deep books are efficient;
  calibrated recurring crowds are efficient at window-open.
- **Classify every idea LEVEL vs SHAPE before filing, and say which in the file.**
  Level = "we estimate the truth better" — died twice (runningmax, gistemp). Shape = "the
  crowd's own distribution is mis-allocated" — survived twice (ladder-rv wings,
  favourite-shrinkage) and is what the 2026-07-25 esports idea is. A shape claim needs no
  data-source edge, so it passes the proxy-vs-primary and glanceable-state screens *by
  construction*; its risk is entirely execution cost + regime stability. Corollary
  (2026-07-25): the deep leg of a bundle is the free anchor, and any bias in it is
  **amplified** in coherently-priced derivative legs whenever the derivative is a convex
  function of the anchor — and those derivative legs usually sit in the fundable band
  while the anchor does not.
- Idea-shaping heuristics that produced filed ideas: (1) take a wiki caveat that says
  "X is itself a strategy-shaped idea" and find the category where X's preconditions are
  strongest; (2) post-kill: sort mispricings by what reveals them — public print → bot
  food; model-run → agent-harvestable; (3) start from the *resolution source*, not the
  market; (4) **new 2026-07-25 — invert the deep-book rule**: a deep, efficient board is
  not just something to avoid, it is a free sharp anchor whenever thin boards resolve off
  the *same object at the same instant*. Look for one-object/many-boards families and
  price the satellites from the anchor.
- **Screen ordering: counterparty checks come BEFORE the backtest, and MEASURED not
  described.** (1) Kalshi catalogue dump — one call, check `settlement_sources` (2026-07-26).
  (2) Bookmaker/exchange. (3) Does a specialist publish the simulation free — read the PAGE
  SOURCE, not the UI. (4) **Rebuild ≥3 settled instances from the live feed and check they
  match what the venue paid** (2026-07-26: PortWatch failed 7/19). Tennis cost a three-hour
  backtest before Pinnacle killed it in ten minutes; RT cost a whole slot-day because gate 0
  was described rather than run. Simulation edge survives only where there is **no
  professional counterparty at all** AND the resolution feed is stable.
- **Any "edge" measured off CLOB midpoints must be decomposed by book state before it is
  believed.** Split by "did this price ever move?" and by leg volume; report the live-book
  number as the headline. Two independent families (esports, tennis) produced double-digit
  phantom edges that vanished or inverted. Earthquake ladders scored 0/314 dead legs, so
  the gate discriminates — it does not kill everything.
- **Refinement of the glanceable-state screen, 2026-07-26.** "The crowd can just look at
  the within-window state" kills a **LEVEL** claim (Netflix, weather dailies, GISTEMP). It
  does **not** kill a claim that the glanceable number is a **biased estimator of the
  number that settles**. Ask the second question every time: *is the statistic they are
  looking at the same statistic the market resolves on?* If it is a partially-realised
  version — a running fraction, a cumulative count mid-window, a provisional print — then
  the visible number is the anchor and the bias in it is the edge, and the fact that
  everyone can see it is what keeps the anchor in place. This is what the Tomatometer idea
  is; look for the same shape wherever a market settles on a live statistic at a *clock
  time* rather than at completion.
- The screen that has killed several candidates: **does the crowd have an observation
  channel into the resolution variable that our pipeline lacks?** (GISTEMP: upstream
  inputs. Netflix: the in-app daily list. Weather dailies: METAR.) Our advantage only pays
  where the within-window state is hidden from the amateur too. **But 2026-07-26 adds the
  mirror failure: if it is hidden from EVERYONE, that is not edge either — it is
  irreducible noise, and the crowd's wide distribution is correct.** Hormuz at window
  close, with all 7 days elapsed, still had ±18.6 ships of publication noise. Edge needs
  the state to be *recoverable by work*, not merely invisible.
