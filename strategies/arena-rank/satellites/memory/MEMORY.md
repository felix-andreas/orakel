# arena-rank/satellites — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **Day 1 (2026-07-25, opus-5 max) done. Founding thesis FALSIFIED; a smaller mechanism
  survives.** Full numbers: `results/backtest-2026-07-25.md`.
- Live rows reported to the CEO for the July cohort (checks 2026-07-31 12:00 ET, 6 days
  out) across 11 boards. 7 applications active, 4 inactive.
- **Day 2 first job: archive the live tables.** No forward vintage record exists unless we
  make it. Snapshot all six resolving slices daily (nosc / sc / math-nosc / coding-nosc /
  code-webdev / agent) — the July check is 2026-07-31 16:00 UTC and a refresh can land on
  the check morning (happened in 1 of 51 reconstructable instances; the venue used the
  fresher table).
- Watch the Chinese board: Alibaba 1476 vs Moonshot 1473 on the resolving table, both
  Preliminary. One refresh can flip it. It is also the highest-capacity satellite.

## Medium-term

- **The one fine-print fact that matters most:** `arena.ai/leaderboard/text` (default) is
  style control **ON**. Boards saying "style control off" resolve on
  `text/overall-no-style-control`, which is a *different ordering*. The founding document's
  flagship "mispricing" (Chinese board, Alibaba 0.786 vs Moonshot 0.182, called narrative
  anchoring) was just the wrong table — Alibaba is genuinely ahead. Always parse the
  resolving URL out of the rules text, never trust the slug or the default view.
- **Gate 2 killed the simulation** exactly as the idea specified: anchor-calibrated joint
  order statistics lose to the de-vigged market at every checkpoint (satellite log-loss
  1.244 vs 0.504; better in 1/10 cohort-months). Gate 3 (portfolio effect) also failed —
  anchor calibration picks incumbency = 0.0. Don't rebuild these; `src/simulate.py` is kept
  as the negative result.
- **What survives:** sharpen the crowd's own distribution, p^α renormalised, α = 1.75.
  OOS +0.111 log-loss (t = +2.63, 9/10 months); T−7d +0.106 (t = +7.49, 10/10). Survives
  t+24h delayed execution with raw-mid fills +2c adverse: +11.9c/trade, t = +4.16, no
  half-sample sign flip. It is the favourite-longshot bias, not the idea's mechanism.
- **Sub-arena boards have zero resolved instances** (Math/Coding/WebDev/Agent all created in
  the last six weeks). Every backtest number comes from overall-ranking boards. Biggest gap
  in the evidence; only time fixes it.
- Zone problem: the trade buys the favourite, and on this cohort favourites sit at
  0.93–0.99 — outside the fundable 3–50c band. In-band trades need a mid-priced favourite
  (rare; the Chinese board in 2026-04 was one).
- Gate 0 (98% on vintage-pinned instances), Gate 1 (repricing accrues over days, not
  minutes) and Gate 5 (books/capacity fine except WebDev and Coding) all pass.
- Regime caveat to keep testing: 2026-02 → 2026-06 all resolved to Anthropic. The 2025
  Google-era months are also positive and the single negative month (2025-12) was a
  Google/xAI split — so the bias is not purely the current regime, but n = 10 months.

## Long-term (wiki candidates — drafted 2026-07-25, see WORKLOG)

1. **"Published CI ≠ uncertainty of the published number."** Realised sd(Δ printed score)
   for top-25 arena models is 1.2 at 7d vs a mean published 95% CI of ±5.9, and the
   publisher's own Rank Spread (median width 15 ranks) is missed only 1.3% of the time
   against a median realised |Δrank| of 1. An index that publishes error bars about a
   *latent* quantity will make you fade favourites and lose if you use those bars as σ for
   the *printed* quantity. Measure the printed series' own persistence first.
   (Generalises: CPI/BLS intervals, poll MoE, Elo CIs.)
2. **Header-driven parsing for archived tables.** A fixed-index parser scored Gate 0 at 54%
   because the column set changed three times since 2025-05 and it silently read a vote
   count as a score. Same silent-corruption class as revised-vintage backtests, different
   cause: the vintage was right, the reader was wrong.
3. **Wayback access notes:** call CDX over **https** (the `http://` form is refused by the
   egress proxy with a misleading "host not in allowlist"), and `…/web/<ts>id_/<url>`
   returns raw gzip bytes. Also: "no captures after date X" may just mean the site
   rebranded — check the successor host before declaring vintages unobtainable.
4. **Stamped-data-date vintage pinning.** When a page stamps its own data date and refreshes
   discretely, a capture need not sit at the check instant: use a dense capture series of
   any slice to date the refresh, then take the resolving slice's capture carrying that same
   date. Turned 103 sparse captures into 47/47 exact resolution reproductions.
