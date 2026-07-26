# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-26 (run 5, Felix: "don't pick markets that are already efficient"). **Filed
  nothing positive.** Worked up shipping-chokepoint transit ladders (IMF PortWatch), killed
  it three ways, filed
  **`ideas/2026-07-26-chokepoint-transit-ladders-discarded.md`**. It passed EVERY screen we
  own except the two I ran: live 1-2c book, 174 wallets, $28k/7d taker flow both sides,
  leg-sum 1.019, zero taker fee, 19 resolved instances, ~$36M open siblings.
  **(a) Kalshi runs the identical contract** (`KXHORMUZWEEKLY`, `settlement_sources` =
  our exact PortWatch URL), 156k–446k contracts/wk, 1c spreads, and is **unbiased** at
  window close: mean err +2.63, se 6.19, t=0.42, n=9. **(b) No cross-venue spread**: PM vs
  Kalshi on the realised winner +4.6pp, se 3.8, t=1.2 — PM if anything the *better* venue.
  **(c) The feed is not a number**: PortWatch restated settled weeks −9% to **+247%**;
  today's API reproduces the wrong winning bucket on **7/19** boards; the two venues
  resolved the SAME week to contradictory values 2 days apart (Kalshi 15 / PM 40–59 / feed
  52 today). No vintage archive exists (ArcGIS query endpoints aren't in Wayback), so the
  family is **unbacktestable**, not merely efficient.
- **THE TOOL FROM THIS RUN — use it first, every run.** Kalshi's whole catalogue is one
  unauthenticated call: `api.elections.kalshi.com/trade-api/v2/series?limit=1000` →
  **12,186 series** with `settlement_sources` URLs. Per-series `/markets` gives
  `volume_fp`, `floor_strike` and **`expiration_value` = the exact settled integer**;
  `/series/<T>/markets/<tk>/candlesticks?...&period_interval=60` gives the price path.
  This turns gate 0 from an argument into a regression AND gives free point-in-time
  vintages of any resolution source. In `wiki/reference/sharp-line-screen.md`.
- **Kalshi coverage map (screened 2026-07-26, don't re-derive):** covers RT (244 series),
  Netflix ranks (25), MrBeast/YouTube views, GPU rental prices (H100/B200 weekly+monthly),
  metro home values, reality-TV eliminations (Big Brother/Survivor/Love Island/Traitors),
  chess (31), earthquakes (9), UK by-elections (16), Emmys (30), hurricanes (62), Suez +
  Panama chokepoints. **The one clean hole is DOMESTIC BOX OFFICE** — 2 hits, both Golden
  Globe *award* markets; no opening-weekend/weekend-gross/total-gross series anywhere.
- **NEXT CYCLE'S LEAD (unverified — do NOT file as backlog before gate 0).** Polymarket box
  office: `avatar-fire-and-ash-opening-weekend-box-office` $17.1M, moana-2 $4.4M, joker-2
  $2.6M, wicked-for-good $2.4M, dozens at $300k–$1.4M, plus 2nd/3rd-weekend and
  total-domestic-gross boards. Live now: `the-odyssey-2nd-weekend-box-office-20260720175402816`
  ($261k, $79k/24h, 86–92m leg 0.83/0.84) and `spider-man-brand-new-day-opening-weekend-box-office-20260618144048496`
  ($204k). Resolves on **The Numbers final daily figures, explicitly "not studio estimates"**,
  open until BOM+TN both confirm → the Sunday number the crowd sees ≠ the number that
  settles (Tomatometer shape). Fees `culture_fees` 0.05 taker-only. **Gate 0 = BoxOfficePro
  long-range forecast**: live site 403s us, but it **is** in Wayback under
  `boxofficepro.com/long-range-box-office-forecast*` (verified). Pull N archived forecasts,
  match to films, compare to actual AND to the pre-release Polymarket price. Half a day.
- 2026-07-26 (run 4). Filed `ideas/2026-07-26-tomatometer-review-arrival.md` (RT threshold
  ladders; drift −4.14 embargo→settle, −2.23 at T−72h, market implied median +0.50 ABOVE
  the displayed score). **Promoted same day, killed day 1 — I named Kalshi as gate 0 and
  described it instead of measuring it.** Kalshi is the PRIMARY venue for RT ladders
  ($58k–$7.19M vs PM's $25k median, 1c vs 18c spread) and unbiased for settlement. The
  drift was real (replicated −4.29 at 8× sample) and already in the price. This produced
  the PLAYBOOK RULE: **if you name an incumbent you must MEASURE it before filing.**
  Surviving RT screens: checkpoint must be **T−96h or later** (at T−14d/T−7d the market
  loses to a uniform null, LL 3.575 vs 1.655); phantom gate 2/320 legs dead.
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
  (e) **RT/Tomatometer ladders** (Kalshi primary + unbiased). (f) **ALL IMF PortWatch
  chokepoint boards** — Hormuz, Bab el-Mandeb, Suez cumulative — Kalshi unbiased AND the
  feed restates by up to +247% so nothing is backtestable.

## Medium-term

- **Rotten Tomatoes family — dead as a target, but keep the plumbing.**
  `rottentomatoes.com/m/<slug>` embeds `<script id="media-scorecard-json">` with
  `likedCount / notLikedCount / reviewCount / score` + a Top-Critics subscore; plain `curl`
  + browser UA works (remakes need a year suffix: `the_odyssey_2026`). Rounding is
  **half-up on 100·L/N**, 2,128/2,128 (`wiki/reference/rounded-threshold-ladders.md`).
  Wayback holds 54–78 captures/film; `id_` captures are **gzip — decompress before
  regexing**. Kalshi runs 233 `KXRT*` series and is the primary venue.
- Scanned + rejected before working up (07-26): **SpaceX monthly launch counts** (family
  stopped after Feb, bad cadence); **US measles cumulative ladders** (annual $7.78M deep,
  monthlies ~$55k × 12/yr — future first-passage/branching candidate); **chess outrights**
  (simulable but Kalshi runs 31 chess series, and only a few events/year).
- **Netflix weekly Top-10 — killed 2026-07-25, don't re-propose without a daily source.**
  Free ground truth `netflix.com/tudum/top10/data/all-weeks-{global,countries}.tsv`; killed
  because subscribers see the in-app daily Top 10 (decay model 23%/42% argmax vs market
  77%/83%), and the bid side is empty ($0–96 top-of-book) so selling the field is unexecutable.
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
