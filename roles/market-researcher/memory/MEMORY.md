# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-29 (run 8). **Filed nothing positive.** Killed **post-count ladders** ("how many
  times will @elonmusk post this week", Trump Truth Social equivalent) —
  `ideas/2026-07-29-post-count-ladders-discarded.md`. Distinct from mention markets: integer
  count of a *partially-realised* cumulative process, not a Bernoulli on utterance content.
  I picked it precisely because it is the canonical instance of the one glanceable-state
  structure my own long-term memory said survives ("a cumulative count mid-window is an
  anchor and the bias in it is the edge"). **That hypothesis has now been measured and it is
  false here.** Family is large and real: 129 settled boards back to 2024, Elon weeklies
  $3–32M each, 1,153 legs harvested.
- **THE KILL, and it is the new fourth way a quote lies: DEPTH LIVES WHERE THE EDGE IS NOT.**
  Board volume $1.56M; median notional resting at the best ask on the legs my rule buys is
  **$7**. Walking the live book: **+1.72c at $100, +6.54c at $500, +14.36c at $2,000** on a
  2.65c-mid leg. Against the +11.14pp edge, q⁻=0.0709 vs q*: clears by **+0.07pp at $100**,
  fails **−4.95pp at $500**, **−13.04pp at $2,000**. New page
  `wiki/reference/depth-lives-where-the-edge-is-not.md`. **Depth concentrates at the mode;
  mispricing lives in the wings; the two sets are ANTI-correlated** — the property that makes
  a leg mispriced is what makes it unfillable. A board-level tape/liquidity gate is
  structurally incapable of seeing this. **Walk the book at your own price band, for your own
  size, before the modelling.**
- **The decomposition that did the work.** Pooled crowd calibrated (−0.05 / −0.39 / −0.29 /
  +1.21pp at T−120/72/48/24h). One band looked alive — 0.02–0.10, same sign at all five
  checkpoints, **+3.28pp, board-clustered t=+2.46**, Wilson lower 0.0684 > mid 0.0531. The
  **mirror test did NOT fire** (buy-NO on the same legs loses hard) so it was not the spread
  and had to be decomposed. **100% of it is the open-ended TOP bucket**: +15.89pp (n=57) vs
  interior +2.31pp (n=747, fails break-even at both spread assumptions) — and that residue is
  **5 board-wins, all the same leg (Trump `200+`), all May–July**, a cap that stopped tracking
  its underlying. **Always decompose a surviving band by leg TYPE, not just price.**
- **The incumbent screen fired on the mechanism itself.** `KXELONTWEETS` = dormant shell,
  **0 markets** (no venue incumbent on Elon). `KXTRUTHSOCIAL` = **live incumbent on Trump**,
  102 markets / 92 settled, 100–300k contracts and 57–158k OI per week, 1c spreads. And
  Kalshi **re-cut its cap `>220` → `>240`** as the regime moved while Polymarket kept cutting
  at `200+` since May. The counterparty had already fixed the exact stale bucket that
  generated my entire measured edge. Best incumbent result yet: not "their line is unbiased"
  but "their *board design* already corrects the thing you found".
- **Polymarket `prices-history` `p` IS the book midpoint** — median(p − live mid) = 0.00c,
  verified on a live board. Yesterday's print-clusters-at-the-ask trap does not apply to it.
  Worth knowing: the mid is an honest *starting* point; it is still not a fill.
- 2026-07-28 (run 7). Killed **mention markets** ("will X say WORD") — cleared every positive
  screen we own including **no free specialist anywhere** and the tape gate outright. **THE
  KILL is a method:** crowd looked 4.6–6.5pp too high in every band, t up to +6.93; re-priced
  at what we can get (buy NO at the bid, buy YES at the ask) it is **−2.51pp and −7.92pp**.
  *Both sides lose at once* → I measured the spread; closes to the cent (mean(last trade −
  bid) 6.18c). **A LAST-TRADED PRICE IS NOT A FILL EITHER.**
  `wiki/reference/midpoint-is-not-a-fill.md`.
- **THE TRAP: `volume_fp ≥ 20k` gives +21.15pp at t=+7.30 and is LOOK-AHEAD** — only 14.3% of
  a mention leg's lifetime volume trades before T−6h; rebuilt honestly, **−3.06pp, t=−0.72**.
  `wiki/reference/lifetime-volume-is-look-ahead.md`. Audit question: *was this number's value
  at my checkpoint the same as the value I read now?* Suspects: `volumeNum`, `liquidityNum`,
  `open_interest`. Confirmed live again 07-29 — Elon `500+` shows **$96,266 lifetime volume
  and no quote on either side**.
- **Cross-venue screens must filter BOTH sides to real books.** Raw 52 Kalshi↔Polymarket
  pairs, median |Δ| 10.5pp — 33 were phantom midpoints. Real books both sides (19 pairs):
  **+1.87pp, se 0.59, 18/19 within 5pp.** An unpriced leg does not vote.
- 2026-07-27 (run 6). Killed **box office weekend ladders**. Everything upstream passed —
  Kalshi 0/12,187, Pinnacle/Smarkets empty, 98/98 boards rebuilt exactly. Killed by implied
  lognormal **σ 0.120 vs our best in-sample 0.171** + a **free named-analyst forecast**
  (Shawn Robbins, `boxofficetheory.substack.com`, Wednesdays, ~10% MAPE = the implied σ).
- **A hole in the Kalshi catalogue is NOT a positive signal.** An empty slot says no *venue*
  prices the object; an *analyst* still may. Three families lost to "a specialist publishes it
  free" — golf/DataGolf, MLB/FanGraphs, box office/a PNG in a Substack. Run "does a specialist
  publish this free?" as question ONE, and **fit the implied σ early**.
- 2026-07-26 (runs 4–5). Killed shipping-chokepoint ladders (Kalshi runs the identical contract
  off our exact PortWatch URL, unbiased t=0.42; feed restates −9%→+247% so 7/19 boards can't be
  rebuilt — **unbacktestable, not merely efficient**). And `tomatometer/arrival-drift`, promoted
  same day and **killed day 1 because I named Kalshi as gate 0 and described it instead of
  measuring it** → the PLAYBOOK RULE: **if you name an incumbent you must MEASURE it.**
- **THE TOOL — use it first, every run.** `api.elections.kalshi.com/trade-api/v2/series?limit=1000`
  → **12,298 series** (07-29; 12,231 on 07-28), unauthenticated, with `settlement_sources`.
  Per-series `/markets` gives `volume_fp`, `floor_strike`, `yes_sub_title` (the bucket text),
  `result`, and **`expiration_value` = the exact settled value**;
  `/series/<T>/markets/<tk>/candlesticks?...&period_interval=60` gives the hourly path
  **including `yes_bid`/`yes_ask` on 100% of candles** — that is what makes the executable
  test possible on someone else's venue. In `wiki/reference/sharp-line-screen.md`.
  **Also compare their BUCKET CUTS to ours** (07-29): a rival that has re-cut a ladder cap our
  venue left stale has already priced away the mispricing you are about to "find".
- **Kalshi coverage map (07-26 → 07-29, don't re-derive):** RT (244), Netflix ranks (25),
  MrBeast/YouTube views, GPU rental prices, metro home values, reality-TV eliminations, chess
  (31), earthquakes (9), UK by-elections (16), Emmys (30), hurricanes (62), Suez + Panama,
  **Mentions (397)**, sea-ice (8), measles (6), SpaceX launches (57), gas prices (35), IPOs
  (32), city temperatures (84), **primaries (175)**, **Trump post-counts (`KXTRUTHSOCIAL`)**.
  Category totals 07-29: Sports 3,023, Entertainment 2,487, Politics 2,119, Elections 1,521.
  Two holes worked up and both killed: domestic box office, and Elon post-counts
  (`KXELONTWEETS` exists but is a **0-market shell** — a listed series is not a live one,
  always check `/markets` before calling it an incumbent). Read the map as "where the cheap
  kill is available", never "where nobody is looking".
- **KILLED, do not re-propose:** (a) PGA top-5/10/20 (DataGolf free JSON in page source);
  same kill hits MLB playoff ladders (FanGraphs). (b) Tennis 14-leg derivative ladders
  (Polymarket = Pinnacle to +0.07pp). (c) earthquake ladders. (d) esports BO3 derivatives.
  (e) RT/Tomatometer ladders. (f) ALL IMF PortWatch chokepoint boards. (g) ALL Polymarket
  box office boards. (h) **ALL "will X say WORD" mention boards, both venues** — the only
  version not already dead is a **maker-side** construction (Polymarket fees are taker-only,
  so resting orders are free), and that is an execution idea, not a research idea.
  (i) **ALL post/tweet-count ladders, both accounts, both venues** — same maker-side caveat,
  and that is now TWO consecutive families whose only live thread is resting orders.
- **Scanned + rejected 07-29 before working up:** US primary-election winner boards (Michigan
  Aug 4, $1.4M) — Kalshi runs **175 primary series** including margin-of-victory and
  vote-percent ladders, so the cheap kill is available and free polling aggregators sit
  behind it; UK Clacton by-election (Kalshi 16 UK by-election series + Smarkets/Betfair);
  `largest-company-end-of-month` and `grvt-fdv` (resolution variable is a live quoted price →
  Felix's standing instruction); `which-company-has-best-ai-model` (= parked arena-rank);
  `highest-grossing-movie-2026` (box office, dead). Iran/Hormuz/frontline families are the
  bulk of today's top-volume tape and are **all war-adjacent — blocked pending Felix's ruling**.

## Medium-term

- **Scrapers kept from dead families.** RT: `rottentomatoes.com/m/<slug>` embeds
  `<script id="media-scorecard-json">`, curl + browser UA, rounding half-up 2,128/2,128.
  The Numbers: `/box-office-chart/weekend/YYYY/MM/DD`, **estimates carry
  `class="chart_estimate"` and are round to $50k while finals are exact** — a
  machine-readable provisional-vs-final detector.
- **`web.archive.org` was hard-blocked on 07-27** (14 consecutive resets) while
  `archive.org/wayback/available` worked. Test any archive dependency before planning on it.
- Scanned + rejected before working up: SpaceX monthly launch counts (bad cadence); US
  measles ladders (thin, but a first-passage/branching candidate); chess outrights (Kalshi
  31); flu-hospitalization weeklies (off-season); music-chart boards (kworb.net makes the
  state glanceable); IPO/acquisition boards (insider processes); non-US central-bank
  decisions (rates desks on local OIS); Netflix weekly Top-10 (killed 07-25, in-app daily
  list). Arena/LMArena: satellites killed, `favourite-shrinkage` parked until ~08-10.
- **US primary-election winner boards** — was my best parked candidate; **downgraded 07-29**,
  Kalshi runs 175 primary series incl. margin-of-victory and vote-% ladders, and free polling
  aggregators sit behind that. Real fine print still interesting (runoff triggers, Alaska
  top-4, advance-vs-win coherence) but expect a fast double kill. Live through September.
- Scan tooling: `roles/market-researcher/tools/scan/` (Gamma /events → CSV). Order
  `volume24hr` for "alive today"; 12 pages of 100 ≈ 1,200 events ≈ 1 min via plain curl.
  Series discovery: Gamma `/public-search?q=<text>&limit_per_type=50` — **vary the wording
  and union the slugs**, one query alone badly under-returns (5 variants gave 189 post-count
  events, 18 gave 447 mention events). Then `/events?slug=<slug>` **and** `&closed=true` —
  they are different filters, union them (`wiki/recipes/polymarket-api.md`).
- Landscape shape (stable 07-23→07-29): Sports ~9.3k mkts dominates count;
  Politics/Elections dominate volume. ~86% of open markets <$10k volume. As of 07-29 the
  top-volume non-sports tape is overwhelmingly **Iran/Hormuz/ceasefire** — war-adjacent and
  blocked pending Felix's ruling, which materially thins what I can legally scan.
- Seen but unprobed: NSIDC arctic sea-ice min (Kalshi 8 series **and** SIPN publishes a free
  multi-model ensemble — expect a fast kill); VEI-6 volcano; Cat-4 US hurricane landfall;
  EIA/AAA gas price (Kalshi 35 series); CDC counts. Weather city-dailies: bot-patrolled
  intraday, only pre-day/forecast angles remain. "Hit Price" one-touch: trialing as
  barrier-touch/ladder-rv, don't re-propose.
- API gotchas (rest are in the wiki recipe — read it, don't re-derive): CLOB
  `prices-history?interval=max` **silently caps at ~30 days** — pass `startTs=<epoch>`; its
  `p` **is the book midpoint** (verified 07-29, median diff 0.00c). Gamma
  `outcomePrices`/`clobTokenIds` are JSON-encoded strings; offset paging dies at offset 2000
  returning the error as a **200-with-object**. Multi-outcome boards carry untraded
  **placeholder legs** at 0.500. Python `urllib` gets 403 from Gamma — shell out to `curl`
  (Kalshi accepts `urllib` with a `User-Agent`; 12 threads safe on both). Fees:
  `mentions_fees` 0.04, **`culture_fees` 0.05** (tweet/post boards), movies 0.05,
  geopolitics 0; Kalshi's own is `0.07·p·(1−p)`, ~1.75c at p=0.50.

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — see `/wiki/market-selection.md`. Deep books are efficient;
  calibrated recurring crowds are efficient at window-open.
- **Classify every idea LEVEL vs SHAPE before filing, and say which.** Level = "we estimate
  the truth better" — died four times (runningmax, gistemp, box office, mentions). Shape =
  "the crowd's own distribution is mis-allocated" — survived twice (ladder-rv wings,
  favourite-shrinkage) and **died once, 07-29 (post-counts)**. A shape claim passes the
  proxy-vs-primary and glanceable-state screens *by construction*, so its risk is entirely
  **execution cost + regime stability — and that is exactly what killed the first one**:
  the wings where shape claims live are the wings nobody quotes. Test a shape claim's
  fillability FIRST, because its only two failure modes are the two you can check cheapest.
- **Screen ordering, MEASURED not described:** (1) Kalshi catalogue dump, check
  `settlement_sources` — and compare the rival's **bucket cuts**, not just its line. (2) Does
  a specialist publish the simulation free — read the PAGE SOURCE, and check
  newsletters/forums/podcasts, not just web pages. (3) Fit the implied σ. (4) **Price both
  sides at executable prices**, and (4b, added 07-29) **walk the live book at the price band
  your rule would buy in, for the size you would buy** — one `/book` call per leg, before any
  modelling. Report VWAP, never top-of-book, never the mid. (5) Rebuild ≥3 settled instances
  from the live feed. (6) Bookmaker cross-check, filtering both sides to real books.
- **Decompose a surviving band by leg TYPE before believing it.** Open-ended wing buckets have
  unbounded support and are not comparable to interior buckets quoted at the same price; on
  07-29 they carried 100% of an apparent +3.28pp edge while the 747 interior legs carried
  none. Ask of any pooled band: *are these legs the same kind of object?*
- **Any "edge" must be decomposed by book state AND re-priced at the executable price before
  it is believed.** Three families produced double-digit phantom edges that vanished or
  inverted (esports, tennis, mentions). Report the live-book, executable number as the
  headline. Earthquake ladders scored 0/314 dead legs, so the gate discriminates.
- **The binding constraint is WHO ELSE IS HERE — but "nobody" is not sufficient, and after
  07-29 it may not even be the binding one.** Mentions had no incumbent and no edge (the
  spread had it). Post-counts had no incumbent on Elon and no edge (the *depth* had it).
  Two consecutive families died **behind** the incumbent wall, to execution rather than to a
  counterparty. Absence of a counterparty removes our cheapest *check*; it does not create
  edge. **The wall we actually keep hitting now is taker-side reachability.**
- **Glanceable-state screen — refined 07-26, then TESTED AND DEMOTED 07-29.** "The crowd can
  just look" kills a **LEVEL** claim, and does not by itself kill a claim that the glanceable
  number is a **biased estimator of the number that settles** (running fraction, cumulative
  count mid-window, provisional print). But I built a family on exactly that structure and
  the crowd was calibrated to −0.05/+1.21pp. **Being partially-realised makes the object
  eligible, not promising** — it is a reason not to discard, never a reason to expect edge.
  Where the residual is a public, exactly-observable count, the crowd extrapolates it as well
  as we do, and the ladder's *design* (bucket cuts) is the only thing that goes stale.
- **A stale bucket cut is a real mispricing and a worthless one.** A ladder whose open-ended
  cap stops tracking a drifting underlying will genuinely misprice that leg — I measured
  +15.89pp. But it is one regime, it is fixed the moment the venue re-cuts (Kalshi did,
  Polymarket had not), and the leg is unfillable. Check whether a rival venue has re-cut its
  ladder: that comparison dates the mispricing and tells you how long it has left.
- **Does the crowd have an observation channel our pipeline lacks?** (GISTEMP upstream
  inputs; Netflix in-app daily list; METAR.) Mirror failure: **if the state is hidden from
  EVERYONE that is not edge either** — it is irreducible noise and the crowd's wide
  distribution is correct (Hormuz: ±18.6 ships at window close). Edge needs the state to be
  *recoverable by work*.
