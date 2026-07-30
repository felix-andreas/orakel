# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

**Where knowledge lives:** durable method → `wiki/`; the object-by-object record and its
counterparties → `ops/idea-funnel.md`. **Do not restate either here.** This file holds only
what is live, unfinished, or not yet generalisable.

## Short-term

- 2026-07-30 (run 9). Killed **cumulative "by &lt;date&gt;" ladders** (hazard term structure) —
  `ideas/2026-07-30-cumulative-date-ladders-discarded.md`. Universe 219 live / 96 settled;
  non-war live examples GPT-6, Anthropic IPO, Alito, Mythos-class, Arc token.
- **Two kills, either sufficient.** (a) **Kalshi is co-primary** on the identical structure —
  `KXCLAUDE` 2.16M contracts, `KXIPOOPENAI` 1.15M, `KXGPT` 1.05M, `KXALITOANNOUNCERETIRE` 492k
  / 317k OI. Where the contracts truly match, median |Δ| **0.00pp** (Alito) and **1.50pp**
  (Mythos); 6/10 matched rungs within 3pp. (b) **Power:** nearest rung −11.12pp t=−2.15,
  mirror test did NOT fire (buy NO +10.62pp) — and **Wilson upper 0.3455 vs break-even 0.2786
  at zero fee**. Need 91 events, have 29, arriving 0.88/month = **5.9 years**.
- **FIRST FAMILY TO CLEAR THE DEPTH WALL, and the reason is reusable:** a cumulative ladder has
  **no mode** (price monotone in date), so the unquoted legs are the *already-decided* ones, not
  the edge legs. Tradeable band median 2.0c spread, $264 at the bid, deepest legs walk $2,000
  for 0.4–2.0c — vs $7 on post-count wings. Only a *partial* pass ($47 median at the ask).
  → `wiki/reference/nested-ladders-trade-depth-for-power.md`.
- **The near-miss worth remembering: `KXGPT5RELEASE`, `KXGEMINI3`, `KXMYTHOS`, `KXCLAUDE4`,
  `KXO3RELEASE`, `KXDEEPSEEKV4RELEASE`, `KXGROK4` are ALL 0-market shells** — while
  `KXGPT`/`KXCLAUDE`/`KXGEMINI` carry 3.3M contracts. Kalshi rolls successive objects through
  ONE vendor-generic series. Searching my object's name found the abandoned stub and would have
  produced "no incumbent", false by 3.3M contracts. **Search by vendor/venue/person/franchise
  and sort by `volume_fp`.** Now in `sharp-line-screen.md`.
- **Metaculus API is authenticated-only as of 07-30** ("Permission Error"). A free specialist we
  can no longer measure cheaply — public HTML pages remain. Didn't force `needs-gate-0` because
  two other incumbents were measured and decided it.
- 2026-07-29 (run 8). Killed **post-count ladders** on **leg-level depth** — $1.56M board,
  median **$7** at the ask on the legs the rule buys; +11.14pp edge, q⁻ 0.0709 vs q* 0.1204 at
  $500. `wiki/reference/depth-lives-where-the-edge-is-not.md`. The residue was 5 board-wins on
  **one leg** (Trump `200+`), a cap Kalshi had already re-cut.
- 2026-07-28 (run 7). Killed **mention markets**: crowd looked 4.6–6.5pp rich at t=+6.93; at
  executable prices **both directions lose** (−2.51 / −7.92pp) → I measured the spread.
  `wiki/reference/midpoint-is-not-a-fill.md`.
- **Scanned + rejected 07-30 before working up:** Fed decision/cut/hike ladders and all non-US
  central-bank decisions (rates desks on OIS); `largest-company-end-of-*`, `*-valuation-hit-__`,
  Bitcoin/ETH/Solana/gold/Hyperliquid "hit __ by" (quoted-price underlying → Felix's standing
  instruction); Michigan/US primaries (downgraded 07-29, Kalshi 175 primary series);
  `spider-man`/`the-odyssey` domestic gross (box office, dead). **95 of 185 live by-date ladders
  are war-adjacent** — Iran/Hormuz/ceasefire still the bulk of top non-sports tape, still
  blocked pending Felix's ruling, still materially thinning what I can scan.

## Medium-term

- **Two consecutive families (07-28, 07-29) died taker-side only; the one untested construction
  in both is MAKER-side** (Polymarket charges no fee on resting orders). §5 forbids executing,
  not researching. Open question with Felix via the funnel — do not spend a slot on it unheard.
- **Wiki maintenance owed:** `nested-ladders-trade-depth-for-power.md` (mine) and
  `nested-ladders-are-one-draw.md` (slot 1, same day, independent) overlap by design and the
  ownership split is written into both — theirs owns ρ/effective-n, mine owns depth↔power.
  Re-check next run that they haven't drifted; merge only if they do.
- **Scrapers kept from dead families.** RT: `rottentomatoes.com/m/<slug>`, `<script
  id="media-scorecard-json">`, browser UA, rounding half-up 2,128/2,128. The Numbers:
  `/box-office-chart/weekend/YYYY/MM/DD` — estimates carry `class="chart_estimate"` and are
  round to $50k while finals are exact = a machine-readable provisional-vs-final detector.
- **`web.archive.org` hard-blocked 07-27** (14 consecutive resets) while
  `archive.org/wayback/available` worked. Test any archive dependency before planning on it.
- Scanned + rejected earlier, don't re-derive: SpaceX monthly launch counts (cadence); measles
  ladders (thin, but a first-passage candidate); chess outrights (Kalshi 31); flu weeklies
  (off-season); music charts (kworb.net makes state glanceable); IPO/acquisition boards
  (insider processes); Netflix weekly Top-10 (in-app daily list); NSIDC sea-ice (Kalshi 8 **and**
  SIPN's free multi-model ensemble); VEI-6 volcano; Cat-4 landfall; EIA/AAA gas (Kalshi 35);
  CDC counts. Weather city-dailies: bot-patrolled intraday, only pre-day/forecast angles left.
  "Hit Price" one-touch: trialing as barrier-touch/ladder-rv, don't re-propose.
- **KILLED, do not re-propose** (details in `ops/idea-funnel.md`): PGA top-5/10/20 and MLB
  playoff ladders (DataGolf/FanGraphs free); tennis 14-leg derivative ladders; earthquake
  ladders; esports BO3 derivatives; RT/Tomatometer ladders; all IMF PortWatch chokepoint
  boards; all Polymarket box-office boards; all "will X say WORD" mention boards; all
  post/tweet-count ladders; **all cumulative by-date ladders (07-30)**.
- Scan tooling: `roles/market-researcher/tools/scan/` (Gamma /events → CSV), order `volume24hr`
  for "alive today", 14×100 ≈ 1,400 events ≈ 1 min. Series discovery: Gamma
  `/public-search?q=<text>&limit_per_type=50` — **vary the wording and union the slugs** (30
  wordings gave 790 events / 219 ladders; one query badly under-returns). Then
  `/events?slug=<slug>` **and** `&closed=true` — different filters, union them.
- Landscape shape (stable 07-23→07-30): Sports ~7.1k mkts dominates count; Politics/Elections
  dominate volume; ~86% of open markets <$10k. Kalshi catalogue 12,329 series (12,298 on 07-29,
  12,231 on 07-28) — Sports 3,044 / Entertainment 2,489 / Politics 2,120 / Elections 1,523 /
  Mentions 397. Coverage map is in `wiki/reference/sharp-line-screen.md`; read it as "where the
  cheap kill is available", never "where nobody is looking".

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
  modelling; this is the cheapest kill I have added and it needs no data. (5) **Walk the live
  book at the price band and size the rule would buy**, report VWAP not top-of-book.
  (6) Price both sides at executable prices. (7) Rebuild ≥3 settled instances from the feed.
- **Report the bound, never the point estimate.** 07-30's −11.12pp at t=−2.15 with the mirror
  test passing still died because Wilson upper 0.3455 > break-even 0.2786. A monotone gradient
  in the pre-registered direction is not a pass. `wiki/reference/break-even-win-rate.md`.
- **Decompose a surviving band by leg TYPE, and leave-one-family-out.** Open-ended wing buckets
  aren't comparable to interior buckets at the same price (07-29: they carried 100% of a
  +3.28pp edge, 747 interior legs carried none). 07-30: dropping rocket launches took t from
  −2.15 to −1.45, and "launches slip" is the most glanceable fact in that domain.
- **The walls, in the order they now fire:** (1) someone already prices it — 9 of 14 objects;
  (2) taker-side reachability (spread 07-28, leg depth 07-29); (3) **statistical power / draw
  count (07-30)**. Absence of a counterparty removes our cheapest *check*, it does not create
  edge. Full analysis in `ops/idea-funnel.md`.
- **Does the crowd have an observation channel our pipeline lacks?** (GISTEMP upstream inputs;
  Netflix in-app list; METAR.) Mirror failure: **if the state is hidden from EVERYONE that is
  not edge either** — it is irreducible noise and a wide crowd distribution is correct (Hormuz:
  ±18.6 ships at close). Edge needs the state to be *recoverable by work*.
- **A stale bucket cut is a real mispricing and a worthless one** — +15.89pp measured, one
  regime, fixed the moment the venue re-cuts, and the leg is unfillable. Check whether a rival
  has re-cut its ladder: that dates the mispricing and says how long it has left.
- API gotchas live in `wiki/recipes/polymarket-api.md` — read it, don't re-derive. The two that
  cost me time: `prices-history` returns **empty, not truncated**, for windows >~14d (chunk at
  ≤14d; this looked exactly like "unbacktestable" and would have been a wrong kill), and per-leg
  deadlines come from `endDate`, never the question text. Fees: `mentions_fees` 0.04,
  `culture_fees` 0.05, movies 0.05, geopolitics 0; CLOB reports 1000bps maker/taker base on
  AI/politics boards — **unresolved, don't quote it as 10%**; Kalshi's own is `0.07·p·(1−p)`.
