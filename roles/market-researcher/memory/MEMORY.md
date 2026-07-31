# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

**Where knowledge lives:** durable method → `wiki/`; the object-by-object record and its
counterparties → `ops/idea-funnel.md`. **Do not restate either here.** This file holds only
what is live, unfinished, or not yet generalisable.

## Short-term

- 2026-07-31 (run 10). Killed **same-venue nested-board dominance** —
  `ideas/2026-07-31-nested-board-dominance-discarded.md`. Chosen *because* it is a dominance
  claim and therefore immune to wall 3's draw-count kill (needs n=1, not 91). It died on the
  other three walls instead.
- **Population (wall 3 as opportunity count, free, before any book):** full 6,788-event open
  universe → **1** rule-implied cross-board nesting and **4** hard Σ=N>1 boards. 85 US races list
  both a primary and a general board; **81 of 85 general boards are 2-leg PARTY boards**.
  Alaska is the only nestable race (top-4 jungle primary ⇒ multi-candidate general), recurring
  ~once per 4-year cycle.
- **Executable:** AK-gov cross-board **0 violations of 19** (best +0.0000, median −0.0300).
  Σ=N baskets: 3 of 4 lose in **both** directions. The 4th (France 2nd round, 36 legs) is the
  **firm's first genuine executable dominance arb**: +23.90c on a \$33.76 basket, guaranteed.
- **It died twice more.** (a) Depth: survives 100 baskets, gone by 250 — binding legs held
  1,244–1,352 shares against a 75,760-share headline leg. **Total extractable \$8.88.**
  (b) **Carry: +0.35% annualised** on capital locked to Apr-2027 vs ~4% risk-free = **−3.7pp/yr**.
- **NEW GATE, arithmetic only, no backtest:** `executable = |Σmid − N| − K·s̄/2`. Reproduced the
  fillable number **to 4 decimals on all 4 boards**. → `wiki/reference/leg-sum-edge-scales-with-leg-count.md`.
- **Kalshi: 12,355 series** (12,329 on 07-30, +26/day). 11 twin series across these races —
  `KXGOVAK`, `KXBRPRESADVANCE`, `KXBRPRES`, `KXBRBALLOT`, `KXAKSENATE`, `SENATEAK`, `KXAKMOV`,
  `KXBRPRES1MOV`, `KXBRDEP`, `KXGOVPARTYAK`, `KXAKSENGOVCOMBO` — **all 0 volume / 0 OI**, with
  `close_time` a full year after the election (auto-listing signature). **No incumbent**, and the
  object died anyway — 3rd time (12, 14, 15). Mirror of 07-30: there the generic ticker held the
  contracts, here the generic ticker is empty too. **Always check both.**
- **TRAP that cost me a wrong census:** party boards carry `Will A win…`/`Will B win…` —
  single capital letters, which the `Person [A-Z]` filter misses. Gate on
  `volumeNum==0 && liquidityNum==0`, never on the name pattern. First pass said "0 party-only
  boards"; truth was 81. Dormant boards also rank LAST by `volume24hr` — the 1,600-event
  volume-ranked scan found **4 of 85** races. Population counts need the full universe, and
  offset paging caps at 2,000 → date-windowed paging is mandatory.
- Withdrew my own assumption mid-run: Polymarket's `Earn 4%` tag is **not** a venue yield on idle
  USDC, it is `Rewards Automation …` (maker liquidity rewards). The carry comparator is external.
- 2026-07-30 (run 9). Killed **cumulative "by &lt;date&gt;" ladders** (hazard term structure) —
  `ideas/2026-07-30-cumulative-date-ladders-discarded.md`. Universe 219 live / 96 settled;
  non-war live examples GPT-6, Anthropic IPO, Alito, Mythos-class, Arc token.
- **Two kills, either sufficient.** (a) Kalshi co-primary, matched rungs agree to median |Δ|
  0.00–1.50pp. (b) **Power:** Wilson upper 0.3455 vs break-even 0.2786; need 91 events, have 29
  at 0.88/month = **5.9 years**. Full numbers in `ops/idea-funnel.md` row 14.
- **First family to clear the depth wall, and the reason is reusable:** a cumulative ladder has
  **no mode**, so the unquoted legs are the *already-decided* ones. Detail in
  `wiki/reference/nested-ladders-trade-depth-for-power.md`.
- **The near-miss worth remembering: `KXGPT5RELEASE`, `KXGEMINI3`, `KXMYTHOS`, `KXCLAUDE4`,
  `KXO3RELEASE`, `KXDEEPSEEKV4RELEASE`, `KXGROK4` are ALL 0-market shells** — while
  `KXGPT`/`KXCLAUDE`/`KXGEMINI` carry 3.3M contracts. Kalshi rolls successive objects through
  ONE vendor-generic series. Searching my object's name found the abandoned stub and would have
  produced "no incumbent", false by 3.3M contracts. **Search by vendor/venue/person/franchise
  and sort by `volume_fp`.** Now in `sharp-line-screen.md`.
- **Metaculus API is authenticated-only as of 07-30** ("Permission Error"). A free specialist we
  can no longer measure cheaply — public HTML pages remain. Didn't force `needs-gate-0` because
  two other incumbents were measured and decided it.
- 07-29 post-count ladders (leg depth, median **$7** at the ask) and 07-28 mention markets (both
  directions lose ⇒ I measured the spread). Numbers in `ops/idea-funnel.md`; method in
  `wiki/reference/depth-lives-where-the-edge-is-not.md` and `midpoint-is-not-a-fill.md`.
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
  modelling; this is the cheapest kill I have added and it needs no data. **For a DOMINANCE
  object substitute the opportunity count: n=1 suffices for truth, so count live instances and
  the arrival rate of new ones instead.** (4b) On any Σ=N board compute **K·s̄/2** before
  anything else — arithmetic, one book fetch. (5) **Walk the live book at the price band and
  size the rule would buy**, report VWAP not top-of-book; on a BASKET the **thinnest leg sizes
  all K legs**. (6) Price both sides at executable prices. (7) **Divide by the capital lock-up
  and compare to the risk-free rate** — a guaranteed profit is not an edge until it clears the
  hurdle (France: +0.35%/yr over 9 months). (8) Rebuild ≥3 settled instances from the feed.
- **Report the bound, never the point estimate.** 07-30's −11.12pp at t=−2.15 with the mirror
  test passing still died because Wilson upper 0.3455 > break-even 0.2786. A monotone gradient
  in the pre-registered direction is not a pass. `wiki/reference/break-even-win-rate.md`.
- **Decompose a surviving band by leg TYPE, and leave-one-family-out.** Open-ended wing buckets
  aren't comparable to interior buckets at the same price (07-29: they carried 100% of a
  +3.28pp edge, 747 interior legs carried none). 07-30: dropping rocket launches took t from
  −2.15 to −1.45, and "launches slip" is the most glanceable fact in that domain.
- **The walls, in the order they now fire:** (1) someone already prices it — 9 of 15 objects;
  (2) taker-side reachability (spread 07-28, leg depth 07-29, basket-thinnest-leg 07-31);
  (3) **statistical power / draw count (07-30)**, or for a dominance object its analogue, the
  **opportunity count (07-31: 1 nesting and 4 boards in 6,788 events)**; (4) **carry — the
  hurdle rate on locked capital (07-31)**, which only bites once an object survives far enough
  to be *true*, and had therefore never been asked. Absence of a counterparty removes our
  cheapest *check*, it does not create edge — now 4 objects with "none found" and 3 dead.
  Full analysis in `ops/idea-funnel.md`.
- **KILLED 07-31, do not re-propose:** primary⊂general dominance pairs, Σ=N basket coherence,
  cross-board leg-sum arbitrage. The venue is coherent to within its own spread on every board
  that carries the constraint, and negRisk mechanically enforces Σ=1 so only N>1 boards can
  even drift.
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
