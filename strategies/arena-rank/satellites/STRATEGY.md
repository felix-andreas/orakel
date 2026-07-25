# arena-rank/satellites

**Status:** trial (slot 2, started 2026-07-25) · **Day-1 verdict:** founding mechanism
falsified; a smaller mechanism on the same boards survives. See
[`results/backtest-2026-07-25.md`](results/backtest-2026-07-25.md).

## The market family

Polymarket runs a monthly cohort of 8–12 boards that all resolve off **one reading of the
arena.ai (ex-LMArena) leaderboard at one instant** — 12:00 PM ET on the last day of the
month. Each board asks: *which company owns the k-th ranked model in arena slice S?*

Slices in use: Text Overall with style control **off** (#1/#2/#3, and the Chinese-company
subset), Text Overall with style control **on** (#1/#2/#3), Text Math, Text Coding, Code
WebDev, Agent. 123 events found, 63 resolved.

**Read the rules text, not the slug.** The resolving URL is named explicitly in every 2026
board's description, the family has been slugged four ways, and the site rebranded
`lmarena.ai` → `arena.ai` mid-life. Critically, `arena.ai/leaderboard/text` (the default
view) is style control **ON**; the boards that say "style control off" resolve on
`text/overall-no-style-control`, **a different ordering**. Misreading this flips the Chinese
board's answer.

## Method (as built, after day-1 backtesting)

The method is **not** the one the idea proposed. The joint order-statistic simulation was
built, anchor-calibrated leave-one-month-out on the deep #1 board, and **failed Gate 2** —
it loses to the de-vigged market at every checkpoint (satellite log-loss 1.244 vs market
0.504, better in 1/10 cohort-months). It is retained in `src/simulate.py` as a diagnostic
and as the negative result, and is not used to price.

What is used:

1. **Resolve the slice.** Parse the board's rules text for the resolving leaderboard URL,
   the place `k`, any company restriction, and the check instant
   (`src/classify.py`).
2. **Read the table.** Fetch that slice and parse it header-driven — the column set changed
   three times since 2025-05 and a fixed-index parser silently reads a vote count as a score
   (`src/arena_parse.py`). Archive it daily: it is the only vintage record that will exist
   forward.
3. **Coherence screen.** Confirm the board's current favourite is the company owning rank
   `k` in that table, and note the score gap to the next company. Empirical head-to-head
   persistence over one refresh: gap 0–2 → 0.79, 3–5 → 0.98, 6–10 → 0.995, 11+ → 0.997.
   Flag Preliminary / <5k-vote rows: their scores drift with sd ≈ 6–7 vs ≈ 1.2 for
   established models.
4. **Price by sharpening the crowd** (`src/predict.py`). De-vig the board's live mids, then

       p_sharp(i) = p_mkt(i)^α / Σ_j p_mkt(j)^α ,  α = 1.75

   clipped to [0.003, 0.995]. α is the low end of the leave-one-month-out range (1.75–2.5);
   the cap exists because no 10-month sample supports the 0.9999 an uncapped α = 2 implies
   on a 0.99 favourite.
5. **Book gate.** A board is only marked tradeable if the favourite's spread ≤ 5c and depth
   within 10c ≥ $500 (`wiki/reference/thin-market-price-read.md`).

### Why sharpening rather than modelling

The satellite crowds are **underconfident in their own favourite**: at T−7d the favourite
wins 9.2pp more often than its de-vigged price implies (se 1.9pp, t = 4.77, clustered by
cohort-month, 9/10 months positive), confirmed independently by the Herfindahl benchmark
(+0.066, se 0.019). Sharpening gains **+0.111 log-loss out of sample** (t = +2.63, 9/10
months; at T−7d +0.106, t = +7.49, 10/10 months) and survives t+24h delayed execution with
raw-mid fills and 2c adverse selection: **+11.9c/trade over 131 trades / 10 months
(t = +4.16), no sign flip across halves.**

The modelling route is closed for a structural reason worth stating: the resolving quantity
(the *printed* score) is far more persistent than the published ±CI suggests — realised
sd(Δscore) for top-25 models is 1.2 at 7d against a mean published 95% CI of ±5.9 — so the
outcome is near-deterministic given the current table, and the crowd already knows it. The
residual uncertainty is dominated by **new model releases** (~7.7 new top-20 models per 30
days; 20 of 37 refreshes put one in the top-10), which is release timing, i.e. private
information. Nearly-deterministic plus unmodelable leaves no room for a simulator.

## Known limits

- **No backtest exists for the sub-arena boards.** Math #1, Coding #1, WebDev, Agent were
  all created within the last six weeks and have zero resolved instances. All numbers come
  from overall-ranking boards.
- **Zone mismatch.** The trade buys the favourite; on the current cohort favourites sit at
  0.93–0.99, outside the fundable 3–50c band. The in-band version needs a mid-priced
  favourite and is a minority of board-months.
- **Regime risk.** 2026-02 → 2026-06 all resolved to Anthropic. The 2025 Google-era months
  are also positive and the one negative month (2025-12) was a Google/xAI split, so the bias
  is not purely the current regime — but the sample is 10 months.
- **Refresh-vs-check timing** is a live adjudication risk: in 1 of 51 reconstructable
  instances a refresh landed inside the check window, and the venue used the fresher table.
- **Housekeeping (unchanged from the idea):** Anthropic is the favourite on most of these
  boards and this firm's agents are Anthropic models. No information advantage exists — we
  have no access to arena vote data — but Felix may prefer we abstain from Anthropic legs.

## Layout

```
src/arena_parse.py    header-driven leaderboard parser (3 layouts since 2025-05)
src/vintage.py        Wayback CDX + capture fetch, vintage pinning helpers
src/build_vintages.py fetch + parse captures around every board check instant
src/fetch_cohorts.py  Gamma discovery of the whole family
src/classify.py       rules-text -> (slice, place, restriction, check instant)
src/resolve.py        apply a board's resolution rule to a table; pin the resolving vintage
src/drift.py          empirical score drift, Rank-Spread validity, entrant arrivals
src/simulate.py       joint order-statistic Monte Carlo (numpy) — FAILED Gate 2, diagnostic only
src/calibrate.py      anchor calibration on the deep #1 board, leave-one-month-out
src/gate0.py          resolution reproduction
src/gate2_market.py   crowd precision + modal calibration
src/gate2_model.py    model vs market, paired, monthly-clustered
src/gate_flb.py       the surviving rule + delayed-execution test
src/fetch_prices.py   CLOB price history for every leg
src/fetch_books.py    live books + Data-API tape (capacity / book quality)
src/predict.py        live predictions for the open cohort
```

Data frozen to R2 (manifests in `data/*.r2.json`): Gamma events, 304 Wayback captures,
parsed vintages, CLOB price history, live books, entrant spec.
