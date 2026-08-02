# Wiki

Durable, **cross-strategy** knowledge only. Run-specific notes stay in memories; market-
specific notes stay in variant folders. The market researcher maintains this wiki —
promoting long-term memory bullets and retirement post-mortems that generalize, merging
overlaps, pruning stale pages. One concept per file.

## THIS FILE HAS ONE OWNER: the market researcher

**Any other agent may add or edit a wiki _page_, and must not edit `index.md`.** Report the
index line you want in your final message; the market researcher or the CEO adds it.

This is not tidiness. `index.md` is the hottest shared file in the repo — every agent that
learns something durable wants to append one line to it, on the same day, in the same
checkout. It has now been swept three times, most recently on 2026-07-30 when a variant
researcher's commit removed two entries the market researcher had added hours earlier. Each
time the *pages* survived and the index silently lost its pointers to them, which is the worst
shape of the failure: knowledge that exists and cannot be found. And per `AGENTS.md` the
history does not get rewritten to fix it, because another agent is always live.

Two agents cannot both append to one list concurrently and both win. Sole ownership is the
cheapest rule that actually holds.

## Selection

- [market-selection.md](market-selection.md) — where edge is findable (and where it isn't)

## Reference (base rates, biases, methods)

- [reference/sharp-line-screen.md](reference/sharp-line-screen.md) — if a bookmaker prices it, check their line FIRST; cheapest kill we have. **Kalshi's 12,186-series catalogue with declared settlement URLs is ONE unauthenticated call — run it before anything else.** But it has a blind spot: an empty catalogue slot says no *venue* prices the object, never that it is unforecast — box office died to a free Substack, and the market's implied sigma named the analyst before we found him. **And when your thesis is "this venue is too high/low" rather than "we forecast better", check the gap's SIGN, not its size** — 8 of 8 matched tail pairs put Kalshi *above* us (mean +1.52pp, sign-test p=0.0039), which is the wrong side for a fade
- [reference/implied-sigma-names-the-incumbent.md](reference/implied-sigma-names-the-incumbent.md) — run BEFORE the modelling: if the market's implied σ is tighter than your in-sample model, someone published the answer. Box office 0.120 vs 0.171 found a free Substack no venue scan could reach — the third family lost this way
- [reference/checkpoint-artifact.md](reference/checkpoint-artifact.md) — an unpriced board manufactures edge; gate on leg-sum, and if your null model wins, audit the checkpoint
- [reference/phantom-midpoints.md](reference/phantom-midpoints.md) — a dead book quotes 0.05/0.95 and the API calls it 0.51; pooling those fabricated +14pp of fake edge
- [reference/midpoint-is-not-a-fill.md](reference/midpoint-is-not-a-fill.md) — our own first batch beat the market 21/21 and was reachable 2/21; always report the fillable count next to the Brier. **A last-traded price is not a fill either, and the one-line diagnostic is: price both sides — if buying AND selling both lose, you measured the spread (+4.56pp became −1.63pp and −7.92pp, the gap identical to mean(last trade − bid))**
- [reference/lifetime-volume-is-look-ahead.md](reference/lifetime-volume-is-look-ahead.md) — the gate that cheats in OUR favour: "keep the liquid legs" read off a settled record turned −3.06pp into +21.15pp at t=+7.30, because only 14% of the family's volume had traded by the checkpoint
- [reference/existence-is-not-completeness.md](reference/existence-is-not-completeness.md) — never ask "is the file there", ask "was it written after the thing it describes stopped changing". **Five instances in one week**, four in a variant's archive and one in the firm's own tooling: a truncated `fills.csv` from a crashed run read as 38% tradeable against the true 43%, and it parsed cleanly every time
- [reference/tape-gate.md](reference/tape-gate.md) — a 1c spread with listed depth and ZERO trades ever; the third way a quote lies, and the one that revised our own 2-of-21 headline
- [reference/depth-has-a-time-coordinate.md](reference/depth-has-a-time-coordinate.md) — the depth wall rotated onto the **time** axis, and it bites boards that have no price mode to hide behind. On a contract settling at a fixed clock time, **85.5% of the entire tape printed AFTER the resolution instant** at a median 0.994 — settlement carry, not a forecasting market — while the T−24h checkpoint where a forecast is still worth something held a median of **$76** a leg and **38 of 132 legs had no tape at all**. Split the tape on the resolution instant taken from the *rules text* (not `endDate`); over ~50% post-resolution means your reachable size is two orders of magnitude below the headline. And the carry is not the consolation prize: 120/121 legs at a 0.9883 mean entry, Wilson lower **0.9547**, fails by −3.37pp
- [reference/depth-lives-where-the-edge-is-not.md](reference/depth-lives-where-the-edge-is-not.md) — the way a quote lies that survives all three other checks: the board is genuinely liquid, the tape is real, the mid is honest, and the *leg* your edge sits on has **$7** at the ask. Depth concentrates at the mode, mispricing lives in the wings, so the two sets are anti-correlated and a board-level gate cannot see it — walk the book at your own price band, for your own size. **It needs a mode, though**: a standalone binary has no board to hide the depth elsewhere, and the same walk returned **$19.4M at the ask with zero slippage to $10,000**. Ask where the mode is before you walk
- [reference/nested-ladders-trade-depth-for-power.md](reference/nested-ladders-trade-depth-for-power.md) — a 12-leg nested ladder is ONE observation, not twelve. Cumulative "by &lt;date&gt;" boards are the first family to beat the depth wall — no mode, so the unquoted legs are the already-decided ones — and they die on power instead: 219 live boards, 96 settled, **29 independent draws** arriving at 0.88/month against the 91 needed. Count events, compute required n, then divide by the arrival rate — all before the backtest. **Companion to `nested-ladders-are-one-draw.md`** (converged on independently the same day): that page owns ρ and effective n, this one owns the depth/power tradeoff
- [reference/cross-venue-gaps-need-a-shared-scalar.md](reference/cross-venue-gaps-need-a-shared-scalar.md) — two venues 17pp apart on "the same" market have not necessarily disagreed. Where the contracts truly matched, Kalshi and Polymarket agreed to **0.00pp**; the one 16pp gap was one clause of English ("called GPT-6 or greater" vs "recognized as a successor to GPT-5"). If both sides can lose, you measured the **definition** — and every earlier cross-venue kill was safe only because both contracts settled on a shared external number
- [reference/stale-feed-gate.md](reference/stale-feed-gate.md) — the FOURTH way a quote misleads, and the only one where OUR number is the broken one: 64 of 95 rows priced off a feed shut for the weekend, while the book repriced 0.475 → 0.715 during the closure
- [reference/break-even-win-rate.md](reference/break-even-win-rate.md) — a band that went 16/16 with t=+10.3 and is still uninvestable; report q*, q and the 95% lower bound, refuse when the bound is under break-even
- [reference/nested-ladders-are-one-draw.md](reference/nested-ladders-are-one-draw.md) — a ladder is ONE bet on how far the underlying travels, paid k times: 356 legs sat on 84 monotone families with ρ=0.325, so effective n was **173** and the same evidence **clears** its break-even bound at the leg count and **fails** it at the draw count. **Bounded 08-02: the kill is about NESTING, not multi-leg boards** — a six-leg panel of *different calendar days* measured ρ = **−0.008**, design effect 0.96, n_eff = leg count. Ask whether the legs are monotone functions of one random variable; if not, the leg count IS the draw count. And the premium sits on the rungs nearest spot — the rungs a continuing move takes first, which makes it a **cliff, not a tail**: −14% left the book at +0.49, −18% at −5.81
- [reference/clustering-coarser-is-not-safer.md](reference/clustering-coarser-is-not-safer.md) — grouping correlated rows into bigger buckets is not automatically the conservative choice: n_eff went 173 → 238 → 118 → 356 down one nesting ladder, and the only level that "cleared" had ρ = 0.000 because seven clusters cannot identify an ICC
- [reference/sharpen-only-what-persists.md](reference/sharpen-only-what-persists.md) — a favourite-longshot correction is conditional on the ranking persisting; measure persistence on the resolution variable's own archive, and never quote a pooled statistic across a sub-population
- [reference/rare-event-edges-need-rare-event-samples.md](reference/rare-event-edges-need-rare-event-samples.md) — state the carry hurdle as a **break-even event rate** `π* = 1 − a_eff·(1 + r·d/365)` and the power wall and the carry wall become one calculation. **0 of 169** settled tail legs resolved Yes — **+12.97pp over risk-free at executable prices** — and the 95% upper bound (2.22% raw, 3.05% clustered) still failed π\* at every horizon. A perfect record over 169 rare-event draws is not enough, and the inversion is the trap: the safer the leg looks, the smaller π\*, the *bigger* the sample. Also: `d_max` kills some legs on arithmetic alone (0.5c with 152 days to run), weight π\* buckets by **volume** (97.8% of the money sat where π\* ≤ 0.75%), and take the arrival rate off the **settled** record — counting the open book gave "4–22 years" when the truth was ~1,120 draws/yr
- [reference/leg-sum-edge-scales-with-leg-count.md](reference/leg-sum-edge-scales-with-leg-count.md) — on a board where exactly N of K legs resolve Yes, the mid-based leg-sum carries an artifact of exactly **K·s̄/2**, so the boards where the edge looks biggest are the ones where the fake part is biggest. `executable = |Σmid − N| − K·s̄/2` reproduced the fillable number **to four decimals on four independent boards**. The one that cleared the arithmetic then died twice more: a basket is sized by its **thinnest** leg (1,244 shares against a 75,760-share headline leg, \$8.88 total), and a guaranteed profit is not an edge until it clears the **risk-free rate on the capital it locks** — +0.35% annualised over a 9-month lock

- [reference/favorite-longshot-bias.md](reference/favorite-longshot-bias.md) — tails rich, favorites shaved inside bucket families
- [reference/recurring-crowd-calibration.md](reference/recurring-crowd-calibration.md) — cheap test: is this recurring market's crowd already calibrated?
- [reference/delayed-execution-test.md](reference/delayed-execution-test.md) — re-run intraday backtests with t+15min fills; "is the edge inside the first 3 minutes?"
- [reference/published-ci-vs-printed.md](reference/published-ci-vs-printed.md) — a source's error bars describe the latent quantity; the market resolves on the printed number
- [reference/venue-resolution-epsilon.md](reference/venue-resolution-epsilon.md) — the venue can resolve Yes on a feed near-miss; never sell inside the epsilon
- [reference/first-print-vintages.md](reference/first-print-vintages.md) — index markets resolve on the FIRST print; reconstruct vintages or corrupt the backtest. **Rebuild ≥3 settled instances from the live feed before modelling: one "public machine-readable" feed restated by +247% and got 37% of boards wrong**
- [reference/rounded-threshold-ladders.md](reference/rounded-threshold-ladders.md) — the strike IS a rounding boundary; verified half-up 2,128/2,128, and Python's round() is wrong on exactly the 12 ties that decide the bet
- [reference/thin-market-price-read.md](reference/thin-market-price-read.md) — when the midpoint is an artifact; tape, wallets, sibling curves
- [reference/wash-trading.md](reference/wash-trading.md) — detecting fake volume; what to trust instead of the mid

## Recipes (copy-paste code patterns)

- [recipes/polymarket-api.md](recipes/polymarket-api.md) — Gamma / CLOB / Data API endpoints, deep-history pagination, **taker-fee formula by category**, gotchas, Rust snippets

_Seeded 2026-07-22 from the poly experiment's scored findings; everything else starts
clean._
