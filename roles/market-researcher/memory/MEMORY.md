# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-28 (run 7). **Filed nothing positive.** Killed "will X say WORD" **mention
  markets** — `ideas/2026-07-28-mention-markets-discarded.md`. This family cleared every
  positive screen we own: not a quoted object, hidden-but-recoverable Bernoulli rate,
  **no free specialist anywhere** (first family in 7 runs where that question returns
  nothing), hours to resolution, recurring, base rates 0.06–0.91, and it **passes the tape
  gate outright** (0/30 live legs with zero tape, ~127 taker trades/leg, 1–2c spreads) and
  the stale-feed gate (the resolving artifact is the event's own video). Kalshi runs a
  **`Mentions` category: 397 series, 17,001 markets, 15,258 settled, $310.6M in 3 months**;
  Polymarket 447 events / $301.5M lifetime ($53.2M on Trump×Xi alone).
- **THE KILL, and it is a method not a fact.** At a pre-event checkpoint the crowd looks
  **4.6–6.5pp too high in every band 0.10–0.90**, event-clustered t up to **+6.93**,
  surviving to **T−48h** so it is not intra-speech decay. Re-priced at what we can get —
  buy NO at the bid, buy YES at the ask — it is **−2.51pp and −7.92pp**. *Both sides lose
  at once*, which is only possible if I measured the spread, and it closes to the cent:
  mean(last trade − bid) **6.18c**, last-trade-edge minus executable-edge **6.18pp**.
  Under the relative-spread gate: **+0.60pp, t=+0.40**. **A LAST-TRADED PRICE IS NOT A FILL
  EITHER** — prints cluster at the ask on a one-sided book. Price both sides, always, first.
- **THE TRAP, and the day's best find: `volume_fp ≥ 20k` gives +21.15pp at t=+7.30 and is
  LOOK-AHEAD.** Only **14.3%** (median) of a mention leg's lifetime volume trades before
  T−6h; 40.6% of all volume is in the final hour. Rebuilt honestly from the candle path the
  same filter gives **−3.06pp, t=−0.72**. Every filter must be computable at the checkpoint.
  New page `wiki/reference/lifetime-volume-is-look-ahead.md`. Audit question for every
  feature: *was this number's value at my checkpoint the same as the value I read now?*
  Suspects already in our pipelines: `volumeNum`, `liquidityNum`, `open_interest`, and any
  "did this book ever move?" statistic computed over a market's whole life.
- **Cross-venue screens must filter BOTH sides to real books.** Kalshi vs Polymarket on
  matched phrases: raw 52 pairs, median |Δ| 10.5pp, individual legs ±40pp — 33 were phantom
  midpoints (boards opened that day at $34–$178 quoting 0.02/0.98 → 0.500 mid). Real books
  both sides (19 pairs): **+1.87pp, se 0.59, median |Δ| 2.50pp, 18/19 within 5pp.** Same
  line, ~2pp richer on Polymarket. An unpriced leg does not vote.
- 2026-07-27 (run 6). Killed **box office weekend ladders**
  (`ideas/2026-07-27-box-office-weekend-ladders-discarded.md`). Everything upstream passed
  well — Kalshi 0/12,187, Pinnacle/Smarkets empty, **98/98 boards rebuilt to the exact
  bucket**. Killed by: market implied lognormal **σ 0.120 vs our best in-sample 0.171**
  (Brier 0.487 market vs 0.701 us, we win 8/32); a **free named-analyst forecast** (Shawn
  Robbins, `boxofficetheory.substack.com`, 61 free weekly issues, Wednesdays, ~10% MAPE =
  the implied σ); and 0 of 18 band×side×checkpoint combos clearing break-even.
- **A hole in the Kalshi catalogue is NOT a positive signal** (I wrote the opposite on 07-26
  and it was false). An empty slot says no *venue* prices the object; an *analyst* still
  may. Three families lost to "a specialist publishes it free" — golf/DataGolf, MLB/FanGraphs,
  box office/a PNG in a Substack. Run "does a specialist publish this free?" as question ONE,
  and **fit the market's implied σ early**: if it is tighter than your data supports, someone
  published the number.
- 2026-07-26 (run 5). Killed shipping-chokepoint ladders — Kalshi runs the identical contract
  off our exact PortWatch URL and is unbiased (t=0.42, n=9), AND the feed restates −9% to
  +247% so 7/19 settled boards can't be rebuilt. **Unbacktestable, not merely efficient.**
- **THE TOOL — use it first, every run.** `api.elections.kalshi.com/trade-api/v2/series?limit=1000`
  → **12,231 series** (07-28; 12,187 on 07-27), unauthenticated, with `settlement_sources`.
  Per-series `/markets` gives `volume_fp`, `floor_strike`, `custom_strike` (the actual
  strike text), `result`, and **`expiration_value` = the exact settled integer**;
  `/series/<T>/markets/<tk>/candlesticks?...&period_interval=60` gives the hourly path
  **including `yes_bid`/`yes_ask` on 100% of candles** — that is what makes the executable
  test possible on someone else's venue. In `wiki/reference/sharp-line-screen.md`.
- **Kalshi coverage map (07-26, re-verified 07-27/28, don't re-derive):** RT (244), Netflix
  ranks (25), MrBeast/YouTube views, GPU rental prices, metro home values, reality-TV
  eliminations, chess (31), earthquakes (9), UK by-elections (16), Emmys (30), hurricanes
  (62), Suez + Panama chokepoints, **Mentions (397)**, sea-ice min/max (8), measles (6),
  SpaceX/launch counts (57), gas prices (35), IPOs (32), city temperatures (84). The one
  clean hole was DOMESTIC BOX OFFICE — **worked up and killed**. Read the map as "where the
  cheap kill is available", never "where nobody is looking".
- 2026-07-26 (run 4). `tomatometer/arrival-drift` promoted same day, **killed day 1 — I named
  Kalshi as gate 0 and described it instead of measuring it.** Produced the PLAYBOOK RULE:
  **if you name an incumbent you must MEASURE it before filing.**
- **KILLED, do not re-propose:** (a) PGA top-5/10/20 (DataGolf free JSON in page source);
  same kill hits MLB playoff ladders (FanGraphs). (b) Tennis 14-leg derivative ladders
  (Polymarket = Pinnacle to +0.07pp). (c) earthquake ladders. (d) esports BO3 derivatives.
  (e) RT/Tomatometer ladders. (f) ALL IMF PortWatch chokepoint boards. (g) ALL Polymarket
  box office boards. (h) **ALL "will X say WORD" mention boards, both venues** — the only
  version not already dead is a **maker-side** construction (Polymarket fees are taker-only,
  so resting orders are free), and that is an execution idea, not a research idea.

## Medium-term

- **Scrapers kept from dead families.** RT: `rottentomatoes.com/m/<slug>` embeds
  `<script id="media-scorecard-json">` (`likedCount/notLikedCount/reviewCount/score`), curl +
  browser UA, rounding half-up 2,128/2,128. The Numbers: `/box-office-chart/weekend/YYYY/MM/DD`
  and `/daily/...`, 187+571 charts ≈ 4 min, **estimates carry `class="chart_estimate"` and are
  round to $50k while finals are exact** — a machine-readable provisional-vs-final detector.
- **`web.archive.org` was hard-blocked on 07-27** (14 consecutive resets) while
  `archive.org/wayback/available` worked. Test any archive dependency before planning on it.
- Scanned + rejected before working up: SpaceX monthly launch counts (bad cadence); US
  measles ladders (annual $10.9k / monthly $16k — thin, but a first-passage/branching
  candidate); chess outrights (Kalshi 31); flu-hospitalization weeklies ($968, off-season);
  music-chart boards (kworb.net makes the state glanceable); IPO/acquisition boards (insider
  processes); `largest-company-end-of` (resolution variable is a live stock price).
- **Netflix weekly Top-10 — killed 07-25**, subscribers see the in-app daily Top 10 (our decay
  model 23%/42% argmax vs market 77%/83%). **Arena/LMArena** (07-25): satellites killed;
  `favourite-shrinkage` parked until ~08-10.
- Scan tool `roles/market-researcher/tools/scan/` (Gamma /events → CSV + summary). 20 pages
  ≈ 25k open market rows, ~1 min. Order `volume24hr` for "alive today". Series discovery:
  Gamma `/public-search?q=<text>&limit_per_type=50` — **vary the wording and union the
  slugs**, one query alone badly under-returns (18 variants gave 447 mention events).
  `closed=true&order=endDate&ascending=false` returns FUTURE end dates; bound it with
  `end_date_max`, and note sports swamp ~800 events/day.
- Landscape shape (stable 07-23→07-28): Sports ~9.3k mkts dominates count;
  Politics/Elections dominate volume ($2.4B/$1.5B). ~86% of open markets <$10k volume.
- Seen but unprobed: NSIDC arctic sea-ice min (Kalshi has 8 series **and** SIPN publishes a
  free multi-model ensemble — expect a fast kill); VEI-6 volcano; Cat-4 US hurricane landfall;
  EIA/AAA gas price (Kalshi 35 series); CDC counts. **Fit the market's implied σ before
  building anything**, and price both sides at executable prices before believing any of it.
- Scanned 07-25, don't re-scan cold: non-US central-bank decisions (rates desks on local OIS);
  **US primary-election winner boards** (18–50 legs, $160k–$2.1M) — genuine future candidate
  with real fine print (runoff triggers, Alaska top-4, advance-vs-win coherence), parked on
  election-calendar cadence, and primaries are live now through September.
- Weather city-dailies: bot-patrolled intraday — only pre-day/forecast angles remain.
  "Hit Price" one-touch family: trialing as barrier-touch/ladder-rv, don't re-propose.
- API gotchas: CLOB `prices-history?interval=max` **silently caps at ~30 days** — pass
  `startTs=<epoch>`. Gamma `outcomePrices`/`clobTokenIds` are JSON-encoded strings.
  Multi-outcome boards carry untraded **placeholder legs** pinned at 0.500 — filter on
  `volumeNum > 0` and `price != 0.5`. Wayback CDX must be called over **https**. Python
  `urllib` gets 403 from Gamma — shell out to `curl` (but Kalshi accepts `urllib` with a
  `User-Agent`, and 12 threads is safe there).
- All in the wiki recipe (read it, don't re-derive): Gamma offset paging dies at offset 2000
  returning the error as a **200-with-object**; deep history = date-windowed offset paging;
  the taker-fee formula and per-category rates — **`mentions_fees` = 0.04**, movies/culture
  0.05, geopolitics 0. Kalshi's own fee is `0.07·p·(1−p)` (`fee_type: quadratic`), ~1.75c at
  p=0.50, which is most of a typical mid-band edge on its own.

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — see `/wiki/market-selection.md`. Deep books are efficient;
  calibrated recurring crowds are efficient at window-open.
- **Classify every idea LEVEL vs SHAPE before filing, and say which.** Level = "we estimate
  the truth better" — died four times now (runningmax, gistemp, box office, mentions).
  Shape = "the crowd's own distribution is mis-allocated" — survived twice (ladder-rv wings,
  favourite-shrinkage). A shape claim passes the proxy-vs-primary and glanceable-state
  screens *by construction*; its risk is entirely execution cost + regime stability.
- **Screen ordering, MEASURED not described:** (1) Kalshi catalogue dump, check
  `settlement_sources`. (2) Does a specialist publish the simulation free — read the PAGE
  SOURCE, and check newsletters/forums/podcasts, not just web pages. (3) Fit the implied σ.
  (4) **Price both sides at executable prices** — this is now the cheapest kill we own and it
  runs before any modelling. (5) Rebuild ≥3 settled instances from the live feed. (6) Bookmaker
  cross-check, filtering both sides to real books.
- **Any "edge" must be decomposed by book state AND re-priced at the executable price before
  it is believed.** Three families produced double-digit phantom edges that vanished or
  inverted (esports, tennis, mentions). Report the live-book, executable number as the
  headline. Earthquake ladders scored 0/314 dead legs, so the gate discriminates.
- **The binding constraint is WHO ELSE IS HERE — but "nobody" is not sufficient.** Mentions
  had no incumbent of any kind and still had no edge, because the market's own two-sided
  flow had already priced it and the spread ate what was left. Absence of a counterparty
  removes our cheapest *check*; it does not create edge.
- **Glanceable-state screen, refined 07-26.** "The crowd can just look" kills a **LEVEL**
  claim. It does **not** kill a claim that the glanceable number is a **biased estimator of
  the number that settles**. Ask: *is the statistic they see the statistic that resolves?*
  If it is partially-realised — a running fraction, a cumulative count mid-window, a
  provisional print — the visible number is the anchor and the bias in it is the edge.
- **Does the crowd have an observation channel our pipeline lacks?** (GISTEMP upstream
  inputs; Netflix in-app daily list; METAR.) Mirror failure: **if the state is hidden from
  EVERYONE that is not edge either** — it is irreducible noise and the crowd's wide
  distribution is correct (Hormuz: ±18.6 ships at window close). Edge needs the state to be
  *recoverable by work*.
