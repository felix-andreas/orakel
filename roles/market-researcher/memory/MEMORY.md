# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-25 (run 2, extra cycle): filed `ideas/2026-07-25-esports-series-shape-2.md`
  (backlog). SHAPE claim on Polymarket **esports BO3 bundles**: each match is a deep
  `moneyline` book plus two thin derivative books, `map_handicap` (fav −1.5 ≡ wins 2-0)
  and `totals` (O/U 2.5 maps ≡ goes the distance), linked by an exact identity. Take the
  moneyline as the level; trade the derivatives. Measured at T−1h on 1,998 resolved
  series: ML fav 0.727→0.788 (+6.1pp); HC 0.531→0.683; Over overpriced in **8/8
  cohort-months** (−9.0pp, se 1.75, t=−5.16) *including* Jan/Feb when the ML bias was ≈0.
  Convex transfer: ML +5.6pp → HC +13.8pp in the 0.80–0.90 band (2.5×). Delayed exec
  T−6h→T−15m +2c = **+14.7c** (se 2.0). Next run: check CEO pickup; the one thing I could
  NOT close is the external bookmaker line (hltv.org 403, the-odds-api needs a key) —
  that is gate 5 and the best kill shot.
- 2026-07-25 (run 1): `arena-rank-satellites` filed → trialed AND falsified same day.
  Order-statistic simulation lost to the crowd at every horizon (LL 1.244 vs 0.504);
  what survived is a favourite-shrinkage p^α on the crowd's own distribution, now
  running as `arena-rank/favourite-shrinkage` in slot 2. Two lessons I own: (a) my
  flagship example read the **default** leaderboard (style-control ON) while the market
  resolves on the style-control-OFF slice — always verify the exact resolving object and
  say in the idea file how; (b) SHAPE claims have now survived twice (ladder-rv wings,
  favourite-shrinkage) and LEVEL claims died twice (runningmax, gistemp).
- 2026-07-24 recap: gistemp-monthly-nowcast filed → trialed AND killed same day (crowd
  replicates GISTEMP from GHCN-M+ERSST, σ 0.015 vs our proxy floor 0.038). Lessons now
  in `wiki/market-selection.md` (proxy-vs-primary) and
  `wiki/reference/first-print-vintages.md`. Don't re-propose climate-index nowcasts.

## Medium-term

- **Netflix weekly Top-10 family — measured and killed in research 2026-07-25, do not
  re-propose without a daily source.** 8 boards/week (top & #2 × US & global × show &
  movie), 243 resolved instances, real taker flow ~$160k/wk (48–206 wallets, top wallet
  17–27%, not wash). Netflix publishes complete free ground truth:
  `netflix.com/tudum/top10/data/all-weeks-{global,countries}.tsv` (264 weeks, 94
  countries; global has views+hours, countries rank-only). Structure looked ideal —
  market opens Wed, all official data publishes *before* open, nothing lands again until
  the resolving print Tue 15:00 ET, so Mon–Tue trades a frozen-but-unpublished outcome.
  **Killed because subscribers see the in-app daily Top 10**: prev-week decay model got
  23% (shows) / 42% (films) argmax vs market 77% / 83% at Thursday. FlixPatrol = 403 from
  this box; no official daily feed exists. Also measured: crowd modal leg underpriced at
  every checkpoint (Wed 0.65→won 0.75; Mon-frozen 0.92→won 0.97, n=102) but the bid side
  is empty (top-of-book $0–96) so the sell-the-field side is unexecutable; "Other" wins
  4.5%. Reusable if a daily rank feed ever becomes reachable.
- **Esports landscape** (2026-07-25, measured): Polymarket lists ~30–40 full-structure
  BO3s **per day** across cs2/val/lol/dota2/r6siege/sc2/mlbb; **6,710 resolved triples**
  (moneyline + map_handicap + totals) Dec-2025 → Jul-2026, 99.93% arithmetically
  consistent. Legs are typed by `sportsMarketType`; `gameStartTime` gives an exact
  pre-match checkpoint. Moneyline books are deep (median $33k cs2 / $81k lol, 1c spread);
  the derivative books are 5–20× thinner ($1.4k / $4.6k median) — that gap is the whole
  idea. Realised: fav 2-0 57.0%, fav 2-1 20.2%, dog 2-1 12.5%, dog 2-0 10.3%. Bias is
  **bigger on tier-1 events** (LEC/LPL/VCT/BLAST) than on obscure qualifiers — fan money,
  not information. Taker-tape check: buying the underdog lost −18.3c/share over 4.43M
  shares. Effect reproduces on a disjoint low-volume sample (ML +7.0pp, Over −10.0pp).
- **Arena/LMArena** (2026-07-25): satellites idea killed; `favourite-shrinkage` runs in
  slot 2. Facts that outlived it: Wayback DOES cover the whole family life once you
  follow the `lmarena.ai` → `arena.ai` rebrand (8,132 captures of the new host); the
  resolving slice is `text/overall-no-style-control`, NOT the default `/leaderboard/text`;
  the column layout changed 3× so parse header-driven, never by index.
- Scan tool `roles/market-researcher/tools/scan/` (Gamma /events → CSV + summary).
  20 pages ≈ 26.7k open market rows, ~1 min. Order `volume24hr` for "alive today".
  Series discovery: Gamma `/public-search?q=<text>&limit_per_type=50` finds all instances
  of a recurring family incl. resolved; vary the query wording and union the slugs
  (one query alone under-returns — 12 variants gave 251 Netflix events vs 20).
- Landscape shape (stable 07-23→07-25): Sports ~11.6k mkts dominates count;
  Politics/Elections dominate volume ($2.4B/$1.5B). ~87% of open markets <$10k volume;
  fast-resolving supply plentiful. Current hot non-sport supply: Iran/Hormuz geopolitics
  (~$3.7M family, energy desks are sharp there — avoid), AI-leaderboard rankings, box
  office (Spider-Man opening wknd $188k, ends Aug 2), Musk tweet counts (poly already
  proved that crowd calibrated — avoid).
- Other primary-source markets seen but unprobed: NSIDC arctic sea-ice min ($62k, Oct 1 —
  one instance/yr, bad trial cadence); VEI-6 volcano; Cat-4 US hurricane landfall (NHC);
  EIA/AAA gas price; CDC counts. Box office is the best unprobed one: recurring weekly,
  no financial incumbent, but Thursday-previews→weekend multiplier is a hobbyist-modelled
  relationship, so run the calibration test before spending anything.
- Scanned 2026-07-25 and parked with reasons (don't re-scan cold): **company market-cap
  ranking boards** ($4.18M/$437k/$302k, 29 legs) — rejected, resolution variable is a live
  stock price, glanceable + near-deterministic. **Non-US central-bank decision boards**
  (BOJ $300k, Brazil $159k, ECB $141k; ~100 instances/yr) — parked, sharpest agent is a
  rates desk pricing the local OIS curve and we can't read those curves free.
  **US primary-election winner boards** (Aug 4/11/18 slates, 18–50 legs, $160k–$2.1M,
  dozens resolved this cycle) — genuine future candidate, real fine print (runoff
  triggers, Alaska top-4, advance-vs-win coherence); parked only because the cadence is
  election-calendar-bound, not daily.
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
- Added to the wiki recipe 2026-07-25: Gamma **offset paging dies at offset 2000** and
  returns the error as a 200-with-object (a list-assuming parser drops pages silently);
  `/events/keyset`'s param is `after_cursor`, not `cursor`; the working deep-history
  pattern is **date-windowed offset paging** (`end_date_min`/`end_date_max`). Also the
  **taker-fee formula** `fee = shares × rate × p × (1−p)`, rate 0.05 sports / 0.07 crypto
  / 0.04 politics-finance-tech / **0 geopolitics** — peaks at ~1.25c/share at p=0.50, so
  it bites hardest exactly in the 3–50c fundable band. Makers pay nothing.
- Binary sports markets: the two token midpoints sum to **1.0000** exactly, so there is no
  overround at the mid and de-vigging is a no-op — the real cost is spread + taker fee.

## Long-term

- Inherited from poly: edge lives in sim-tractable, thin-to-mid, structurally quirky,
  fast-resolving markets — see `/wiki/market-selection.md`. Deep books are efficient;
  calibrated recurring crowds are efficient at window-open.
- Idea-shaping heuristics that produced filed ideas: (1) take a wiki caveat that says
  "X is itself a strategy-shaped idea" and find the category where X's preconditions are
  strongest; (2) post-kill: sort mispricings by what reveals them — public print → bot
  food; model-run → agent-harvestable; (3) start from the *resolution source*, not the
  market; (4) **new 2026-07-25 — invert the deep-book rule**: a deep, efficient board is
  not just something to avoid, it is a free sharp anchor whenever thin boards resolve off
  the *same object at the same instant*. Look for one-object/many-boards families and
  price the satellites from the anchor.
- The screen that has now killed three candidates in a row is one question: **does the
  crowd have an observation channel into the resolution variable that our pipeline
  lacks?** (GISTEMP: upstream inputs. Netflix: the in-app daily list. Weather dailies:
  METAR.) Our comparative advantage only pays where the within-window state is hidden
  from the amateur too — counts that must be assembled, estimates with error bars,
  orderings over many objects. Test it *before* building anything.
