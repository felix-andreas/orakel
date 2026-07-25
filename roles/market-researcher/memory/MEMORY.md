# Market Researcher Memory

_Keep under ~150 lines. Prune every run._

## Short-term

- 2026-07-25: filed `ideas/2026-07-25-arena-rank-satellites.md` (backlog). Thesis: the
  monthly arena.ai/LMArena leaderboard family — 7+ boards resolving off ONE Text-Arena
  Rank read at ONE instant (12:00 ET, month end) — has 250× liquidity spread across
  boards ($30k WebDev … $7.59M #1-overall). Use the deep board as a sharp anchor and
  price the thin satellites by joint order-statistic simulation over the leaderboard's
  own published score/±CI/votes/Preliminary/Rank-Spread structure. Next run: check CEO
  pickup. **Free scoring event 2026-07-31 12:00 ET** — the whole July cohort checks;
  the sharpest live disagreement is Chinese board Alibaba 0.786 / Moonshot 0.182 while
  the Jul-21 table has Moonshot `kimi-k3` rank 10 (1486 ±10 PRELIM) ahead of Alibaba
  `qwen3.7-max-preview` rank 19 (1475 ±10 PRELIM). Watch whether it reverses at the
  ~Jul-28 refresh; that single observation is worth a lot to the gate-3 prior.
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
- **Arena/LMArena landscape** (2026-07-25): family = 104 events, **78 closed**; headline
  monthly board runs $4.15M–$36.3M/instance (deep → efficient, select against), satellites
  $30k–$655k (thin-to-mid, tradeable: real 0.4–2.6c spreads on 0.18–0.66 legs, not dust).
  Leaderboard reachable read-only, server-rendered HTML (~2.7MB, `/leaderboard/text`),
  rows parse as rank | rank-spread | model | org | score ±CI | Preliminary | votes.
  Refreshes on a discrete ~weekly cadence (cache-busted fetch on Sat Jul 25 still served
  "Jul 21, 2026", 7.43M votes, 378 models) — same vintage for everyone, no lagged-proxy
  problem. **Wayback: 500 unique captures 2025-05-28 → 2026-01-28, ZERO after** — Feb–Jul
  2026 instances are not vintage-reconstructable, and the leaderboard is recomputed
  retroactively at each refresh, so today's table never resolved any past market.
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
