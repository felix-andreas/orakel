# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-26 (run 4, Felix brief: market-SPECIFIC data + simulable process). Filed
  **`ideas/2026-07-26-tomatometer-review-arrival.md`** — Polymarket's weekly **Rotten
  Tomatoes threshold ladders**. Core: the resolution variable is a **counting process still
  running at settlement**. Score = `round(100·liked/total)` at a fixed clock instant; the
  denominator roughly triples during the board's life (median lifetime **5.1d**) and the
  score **drifts DOWN**: embargo→settle **mean −4.14, median −2.0, 11 down/2 flat/1 up**
  (n=14); at T−72h→T **mean −2.23, median −2.0, 8/4/1** (n=13, ~96 reviews added).
  Conditional on denominator: **n<80 → −5.09, n≥80 → −0.67**. Market implied median sits
  **+0.50 above the displayed score** — it prices no drift. Ladder also too diffuse: at
  T−72h (n=57) modal bucket wins **0.684±0.062** vs own Herfindahl **0.535**, PIT outer-20%
  **5.3%** vs 20%. Supply 67 resolved boards, **2–4/Monday**, zero coherence violations.
- **RT run-4 screens, all measured:** phantom gate **2/320 legs dead (0.6%)**, median total
  variation 1.48 (earthquake-ladder profile) — but individual legs still fake (live
  Spider-Man `90+`: Gamma mid **0.740** off a **0.650/0.830** book, $265/$54 depth).
  Checkpoint artifact **found**: at T−14d/T−7d the market **loses to a uniform null**
  (LL 3.575 vs 1.655), 6/11 monotonicity violations → checkpoint must be **T−96h or later**.
  Liquidity in the last 72h is **state-dependent and aligned with the edge**: in-the-grey
  $52.0k of which **$48.8k in 8–92c** (93 wallets); but scream-7 $33.6k / **$37 in band**
  and mario $12.0k / **$19** because they settled far from every strike and collapsed early.
  50/60 boards have ≥1 leg in 10–90c at T−72h. Fees `culture_fees` 0.05 taker-only.
- 2026-07-25 (run 3, brief: find SIMULATION edge). Filed
  `ideas/2026-07-25-quake-ladder-overdispersion-3.md` + discarded-idea file
  `ideas/2026-07-25-tennis-games-ladder-discarded.md` carrying three kills. **Lesson: the
  binding constraint is WHO ELSE IS HERE, not modelling skill.** Quake idea trialed and
  killed same day — crowd already implied Fano 1.362 vs empirical 1.358, and the signal was
  a fresh-board checkpoint artifact.
- **KILLED, do not re-propose:** (a) **PGA top-5/10/20** — DataGolf ships the whole live
  Monte Carlo as free JSON in page source; same kill hits **MLB playoff ladders**
  (FanGraphs). (b) **Tennis 14-leg derivative ladder** — Polymarket = Pinnacle to +0.07pp
  on ≤3c books (27/27 within 3pp); my −7.6pp headline was the phantom artifact (DEAD −27.5pp
  vs LIVE −5.0pp, inverting with liquidity). (c) earthquake ladders (both mid-window speed
  race and the shape claim itself). (d) esports BO3 derivatives (run 2 idea, killed day 1).

## Medium-term

- **Rotten Tomatoes family (2026-07-26), reusable facts.** Resolution source is fully
  machine-readable: `rottentomatoes.com/m/<slug>` embeds
  `<script id="media-scorecard-json">` with `likedCount / notLikedCount / reviewCount /
  score` **plus a separate Top-Critics subscore**. Plain `curl` + browser UA works (RT slugs
  need a year suffix for remakes: `the_odyssey_2026`, `michael_2026`). Rounding is
  **nearest-integer on 100·L/N**, verified on 6 triples (67/227=29.52→**30**) → every strike
  is an integer lattice boundary. **Wayback holds 54–78 captures/film, 5–7/day in release
  week**, and `id_` captures are **gzip-compressed — decompress before regexing** (cost me a
  pass). **Kalshi runs 233 `KXRT*` RT series** + a Metacritic game-score family; its
  `/trade-api/v2/series?limit=1000` and `/markets` are open and unauthenticated — that is our
  cheapest cross-venue check for any culture market, and it is a *retail* crowd, not a sharp
  book, so agreement is weak evidence.
- Scanned 2026-07-26 and rejected before working up: **SpaceX monthly launch-count ladders**
  (`how-many-spacex-launches-in-january/february` exist but the family stopped after Feb —
  only annual boards remain, bad cadence); **US measles cumulative-count ladders**
  (`measles-cases-in-us-in-2026` $7.78M annual is deep, monthlies only ~$55k and 12/yr —
  genuine future candidate for a first-passage/branching sim if cadence ever matters less);
  **chess tournament outrights** (Swiss pairings + tiebreaks are genuinely simulable and
  nobody prices them, but only a few events/year).
- **Netflix weekly Top-10 family — measured and killed in research 2026-07-25, do not
  re-propose without a daily source.** 8 boards/wk, 243 resolved, ~$160k/wk real taker flow.
  Free ground truth: `netflix.com/tudum/top10/data/all-weeks-{global,countries}.tsv` (264
  wks, 94 countries). **Killed because subscribers see the in-app daily Top 10**: decay
  model 23%/42% argmax (shows/films) vs market 77%/83% at Thursday; FlixPatrol 403s here.
  Also measured: crowd modal leg underpriced at every checkpoint (n=102) but the bid side is
  empty ($0–96 top-of-book) so the sell-the-field side is unexecutable.
- **Esports landscape** (2026-07-25, post-kill): ~30-40 BO3s/day, 6,710 resolved triples,
  typed by `sportsMarketType`. Keep the harvest recipe, discard the thesis (phantom).
- **Arena/LMArena** (2026-07-25): satellites idea killed; `favourite-shrinkage` runs in
  slot 2. Surviving facts: Wayback covers the family life once you follow the
  `lmarena.ai` → `arena.ai` rebrand (8,132 captures); the resolving slice is
  `text/overall-no-style-control`, NOT `/leaderboard/text`; layout changed 3× so parse
  header-driven, never by index.
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
  (NHC); EIA/AAA gas price; CDC counts. **Box office is the best unprobed one** — weekly,
  no financial incumbent, and it is the *same shape* as the Tomatometer idea (settles on a
  running total at a clock time), but Thursday-previews→weekend multiplier is
  hobbyist-modelled, so run the calibration test first.
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
- **Screen ordering, learned the hard way 2026-07-25 (run 3): counterparty checks come
  BEFORE the backtest.** (1) Does a bookmaker/exchange price this object? (2) Does a
  specialist publish the simulation free — check the PAGE SOURCE, not the UI. I ran a
  three-hour tennis backtest and then killed it in ten minutes with Pinnacle. Corollary
  (positive): simulation edge survives only where there is **no professional counterparty
  at all**. That is now the first filter I apply to any candidate.
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
- The screen that has now killed three candidates in a row is one question: **does the
  crowd have an observation channel into the resolution variable that our pipeline
  lacks?** (GISTEMP: upstream inputs. Netflix: the in-app daily list. Weather dailies:
  METAR.) Our comparative advantage only pays where the within-window state is hidden
  from the amateur too — counts that must be assembled, estimates with error bars,
  orderings over many objects. Test it *before* building anything.
