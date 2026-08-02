# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

**Where knowledge lives:** durable method → `wiki/`; the object-by-object record and its
counterparties → `ops/idea-funnel.md`. **Do not restate either here.** This file holds only
what is live, unfinished, or not yet generalisable.

## Short-term

- 2026-08-02 (run 12). Killed **White House "full lid by 6:30 PM" per-day binaries** —
  `ideas/2026-08-02-full-lid-timing-discarded.md`. **Cleared W1 outright and W3 marginally,
  and is the first object ever to PASS the mirror test at executable prices** (+8.43c/share
  buying YES at the ask at T−24h, mirror −23.02c) — then died on **size** and on **the bound**.
- **THE FAMILY CENSUS, and it is the tool to reuse.** `/series?limit=50&offset=N` pages the
  **complete** catalogue of recurring families — **2,152**, with `recurrence` (925 monthly /
  796 daily / 220 weekly / 186 annual) — and its `events` array is capped at **20** (so exactly
  20 = truncated, <20 = complete). `/series?slug=X` returns one family's events **uncapped,
  incl. closed**. Settled count ÷ family age = the arrival rate W3 needs. `/events?series=`
  silently ignores the param and returns junk; `series_slug=` returns `[]`. Cross-joined
  against Kalshi's 12,369 series this runs **W1 and W3 together over the whole venue before
  picking an object** — recipe in `wiki/recipes/polymarket-api.md`.
- **W1 inverse-shell form:** `KXFULLLIDBEFORE630PM` declares **our exact two resolution URLs**
  and the identical 6:30PM threshold — **0 markets, ever** — while `KXWHPRESSBRIEFING` (50,609)
  and `KXPRESSBRIEFINGCOUNT` (21,368) are live. A peer venue priced the contract and declined
  to list it: a warning, not a green light.
- **W2 KILL, new axis — depth has a TIME coordinate** (now `wiki/reference/depth-has-a-time-coordinate.md`).
  **85.5%** of the family's whole tape ($922,456 of $1,079,036) printed **after** the 6:30PM
  resolution instant at a median **0.994** — settlement carry, not a market. At T−24h the median
  leg held **$76**, **38/132 legs had zero tape**, total ask-side notional over the entire
  6-month record **$25,606**. Live book: 5 of 6 legs carry **$63–$146** total at 79–89c spreads.
  The price-mode caveat did **not** save it — this board has no price mode; the mode is in time.
- **Gamma's quote fields are a stale CACHE — never gate on them.** Gamma: 0.31/0.38 (**7c**);
  live CLOB book: **0.15/0.94 (79c)**; plus `0.50` mids that were really (0.06+0.94)/2, all
  with an identical `updatedAt`. **Fetch `clob.polymarket.com/book` per token.**
- **ICC = −0.008: a per-day binary panel is NOT a nested ladder** (132 legs/22 boards, design
  effect 0.96, n_eff = 132). Test: *are the legs monotone functions of one random variable?*
  If not, leg count IS draw count. Bounds the 07-30 nesting kill the other way.
- **The bound, which decided it:** 52/87 = 0.598 vs q\* **0.5309** (mean executable ask 0.5209
  + politics fee 0.0100); Wilson lower **0.4926 → FAILS**; point estimate +6.90c/sh, t=+1.44.
  Mid bias was real and large (+16.2pp, t=+4.23 at T−24h) and *survived a 14c spread* — the
  first time that has happened. It is still **one weekday**: Sat +19.26c (t=+2.31, 16/20),
  **Fri −10.00c** (6/17). Edge is all in H2 (+11.78c vs +1.91c) while reachable size fell
  $19,712 → **$5,894**. Escape route closed the object-16 way: post-deadline favourite
  **120/121 legs** at 0.9883 entry, Wilson lower 0.9547, **fails −3.37pp**.
- **BOTH BACKLOG SEEDS ARE NOW W1-DEAD, not merely power-constrained — do not revisit.**
  (a) Titled Tuesday chess: `KXTITLEDTUESTOP` **1,000 markets / 171 active / 155,922 contracts
  / 127,770 OI**, `KXTITLEDTUESDAY` 533 / 57 / 118,832. (b) GPU rental prices: Kalshi runs
  **38 compute-price series** (H100/H200/B200/A100/RTX5090/RTX PRO 6000 × weekly/monthly/
  quarterly/yearly), all settling on **`dashboard.ornnai.com/compute`** — `KXA100MAX` alone
  18 markets / 16,734 contracts. The 08-01 note "no plausible incumbent" was wrong on both.
- **W1 and W3 look ANTI-CORRELATED — the finding to test next.** Of 2,152 families the ones
  with no Kalshi twin are almost all *monthly or rarer* (chatgpt-outage, bank-failure,
  NY/Seattle precipitation, scorigami) → W3-dead by arrival; everything daily/weekly (TSA —
  Kalshi declares our exact URL — tornadoes, eggs, FDA, MrBeast, weather, mentions, macro) has
  a live incumbent. Kalshi lists what recurs *because* it recurs. Unproven; one run to
  quantify, and it bears directly on Felix's "is this the right pond" question.
- 2026-08-01 (run 11). Killed **deep-tail carry** (NO at the ask on ≤5c standalone binaries) —
  `ideas/2026-08-01-deep-tail-carry-discarded.md`. Cleared W2 and W3; died on W1's *direction*
  and on W4 at the venue's real horizon. Numbers live in `ops/idea-funnel.md` row 16 and
  `wiki/reference/rare-event-edges-need-rare-event-samples.md`. Reusable residue only:
  - For any premium-collecting object walls 3 and 4 are **one** calculation:
    `π* = 1 − a_eff·(1 + r·d/365)`; `3/π*` floors the draw count *assuming you never lose*;
    `d_max = 365(1−a_eff)/(a_eff·r)` kills some legs on arithmetic alone. **The safer the leg
    looks, the smaller π\*, the BIGGER the sample** — that inversion is the trap.
  - **Weight π\* buckets by VOLUME, not leg count** (≤45d held 0.5% of the money, ≥150d 97.8%).
  - **When the thesis is "this venue is too high/low", test the gap's SIGN, not its size**
    (8/8 pairs the wrong way, p=0.0039). Kalshi pays collateral interest, PM USDC does not —
    the carry adjustment moves the comparison, usually against a fade.
  - **Take arrival rates off the SETTLED record; the open book is a snapshot, not a rate.**
    Counting the live cohort gave "4–22 years" where the truth was ~1,120 draws/yr.
- **Scanned + set aside, do not re-derive:** `monthly-listeners` (Spotify, 3 artists);
  `movie-delay`; 435 US House race boards (Kalshi + free forecasters); Setka Cup table tennis
  and the obscure soccer/cricket series (Pinnacle). Titled Tuesday and GPU rental are **dead
  on W1** as of 08-02 — see above, not merely young.
- **CENSUS TRAPS, each cost me a wrong number.** (a) Dormant boards rank LAST by `volume24hr`
  and offset paging caps at 2,000 → **date-windowed paging is mandatory** (1,600-event scan vs
  9,347 events / 96,642 markets). (b) Gate on `volumeNum==0 && liquidityNum==0`, never on a
  name pattern. (c) Arrival rates come off the **settled** record, not the open book.
- Polymarket's `Earn 4%` tag is **not** a venue yield on idle USDC — it is maker liquidity
  rewards; the carry comparator is external (T-bills). Kalshi's *is* real collateral interest.
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
- **Scrapers kept from dead families.** RT: `rottentomatoes.com/m/<slug>`, `<script
  id="media-scorecard-json">`, browser UA. The Numbers `/box-office-chart/weekend/Y/M/D`:
  `class="chart_estimate"` + $50k rounding = a machine-readable provisional-vs-final detector.
  `web.archive.org` hard-blocked 07-27 while `archive.org/wayback/available` worked.
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
- Scan tooling: `roles/market-researcher/tools/scan/` (Gamma /events → CSV). Family
  enumeration is now **`/series` paging (08-02, see above)** — it supersedes both the
  `series[].slug` trick and `/public-search`, which stay useful for one-off lookups.
- Landscape shape (stable 07-23→08-02): Sports dominates market count; Politics/Elections
  dominate volume; ~86% of open markets <$10k; 4,707 open events / 52,710 markets on 08-02.
  Kalshi catalogue **12,369 series** (12,368 on 08-01, ~+13–26/day). Coverage map in
  `sharp-line-screen.md`; read it as "where the cheap kill is available", never "where nobody
  is looking".

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — `/wiki/market-selection.md`. Deep books are efficient; calibrated
  recurring crowds are efficient at window-open.
- **Classify every idea LEVEL vs SHAPE before filing, and say which.** Level = "we estimate the
  truth better" — died six times (runningmax, gistemp, box office, mentions, the GPT-6
  cross-venue gap, and full-lid 08-02 — where the level claim was **right** and unreachable). Shape = "the crowd's own distribution is mis-allocated" — survived twice
  (ladder-rv wings, favourite-shrinkage), died twice (post-counts 07-29, date ladders 07-30).
  A shape claim passes proxy-vs-primary and glanceable-state *by construction*, so its risk is
  entirely **execution cost, regime stability, and now sample size** — check all three first.
- **Screen ordering, MEASURED not described:** (1) **Count DRAWS, compute required n, divide
  by arrival rate** — free, no data. Substitute the **opportunity count** for a dominance
  object and the break-even **event rate π\*** (+`3/π*`) for a premium-collecting one; measure
  **ICC** rather than assuming legs are or aren't nested. (2) Kalshi catalogue —
  `settlement_sources`, **bucket cuts AND rule text**, search by *vendor*, and call `/markets`
  (0-market shells cut both ways). (3) Does a specialist publish the simulation free — read the
  PAGE SOURCE, check newsletters/forums/podcasts; Manifold as a third quote. (4) Fit the
  implied σ. (4b) On any Σ=N board compute **K·s̄/2** first — arithmetic, one book fetch.
  (5) **Walk the live book at the band, the size AND the hour the rule fires**; VWAP not
  top-of-book; on a BASKET the **thinnest leg sizes all K legs**; split the tape on the
  resolution instant. (6) Price both sides at executable prices. (7) Divide by the capital
  lock-up vs risk-free. (8) Rebuild ≥3 settled instances from the feed.
- **Report the bound, never the point estimate.** 07-30's −11.12pp at t=−2.15 with the mirror
  test passing still died because Wilson upper 0.3455 > break-even 0.2786. A monotone gradient
  in the pre-registered direction is not a pass. `wiki/reference/break-even-win-rate.md`.
- **Decompose a surviving band by leg TYPE, and leave-one-family-out.** Wing buckets aren't
  comparable to interior buckets at the same price (07-29: wings carried 100% of a +3.28pp edge,
  747 interior legs none). 07-30: dropping rocket launches took t from −2.15 to −1.45.
- **The walls, in the order they now fire:** (1) someone already prices it — 10 of 17 objects;
  (2) taker-side reachability (spread 07-28, leg depth 07-29, basket-thinnest-leg 07-31) —
  has a pass state, and **two axes**: on the PRICE axis it needs a mode, so it misses a
  modeless ladder (07-30) and a standalone binary (08-01); on the **TIME axis it needs none**
  and killed 08-02 anyway (85.5% of the tape after the resolution instant, $76/leg at T−24h).
  **Walk the book at your band, your size AND your hour**; (3) **statistical power / draw
  count (07-30)**, or for a dominance object the **opportunity count** (07-31) — cleared
  08-01, marginal 08-02; (4) **carry — the hurdle rate on locked capital (07-31)**. **08-01: on any premium-collecting object, walls 3 and 4 are the SAME
  calculation** — state the hurdle as a break-even *event rate* π\*, weight π\* by **volume**
  across horizon buckets, and `3/π*` is a floor on draws that nothing can get under.
  Absence of a counterparty removes our cheapest *check*, it does not create edge — 4 objects
  "none found", 3 dead. Full analysis in `ops/idea-funnel.md`.
- **KILLED 08-02, do not re-propose:** White House full-lid timing binaries in any taker-side
  form. The crowd IS ~16pp low at the mid and ~8pp low at the ask, and there is $76 there.
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
- API gotchas live in `wiki/recipes/polymarket-api.md` — read it, don't re-derive. The three
  that cost me time: `prices-history` returns **empty, not truncated**, past ~14d (chunk at
  ≤14d — this looks exactly like "unbacktestable"); per-leg deadlines come from `endDate`, not
  the question text — but the **resolution instant** comes from the *rules text*, and on some
  families `endDate` collapses to the board close; and **Gamma's quote fields are a stale
  cache** (08-02). Fees: politics/finance/tech 0.04, culture/economics/weather/sports 0.05,
  crypto 0.07, geopolitics 0; Kalshi's own is `0.07·p·(1−p)`.
