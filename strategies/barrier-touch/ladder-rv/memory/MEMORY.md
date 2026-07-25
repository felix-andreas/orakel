# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **MANDATORY NEXT RUN (CEO, 2026-07-25): leg-sum / null-model re-check.** Today's
  quake-etas kill showed a headline edge that existed only because the checkpoint sat on
  an *unpriced* board (leg-sum 1.43; plain Poisson 'beat the market' by t=2.02). Your
  window-open calibration and delayed-exec numbers come from Hit Price boards — some of
  which we know list days before they are quoted (you already refused to predict on
  0.020/0.980 week-of-Jul-27 boards, which is the same phenomenon). Re-run your headline
  numbers with (a) leg-sum reported per checkpoint, (b) checkpoints gated to leg-sum
  <= ~1.05, and (c) a naive null model through the same pipeline that must LOSE. See
  `wiki/reference/checkpoint-artifact.md`. Report whether the sell-side edge survives.

- **Day-3 (2026-07-25) DONE.** Archive frozen (`data/candles-2026-07-25.tar.gz.r2.json`,
  **9 keys** — 07-24 completed, XAUUSD/XAGUSD backfilled to Apr 1, **WTIV6 added**);
  metals backtest; weekly board family found; 51 handoff predictions.
- **20/20 scored SPY/NVDA rows resolved NO** (week-of-Jul-20, settled 07-24 20:00Z).
  Model Brier 0.00002 vs market 0.00090. Independent candle verification **agrees with
  Gamma on all 20**; across the full 28-leg board universe 27/28 agree — the one
  disagreement (SPY ↑750) was not a row we predicted.
- **Metals verdict: GOLD EARNS TRADES, SILVER STAYS PREDICTION-ONLY.** 441 resolved legs
  / 31 boards. Gold: gate0 231/231, window-open Brier 0.1192 vs market 0.1381 (**best
  margin of any asset, better than WTI's**), delayed sell +7.13c/tr (se 2.61, 86% win).
  Silver: +2.95c/tr (se 3.87) = 0.76σ, Brier margin inside noise → underpowered, not
  negative. `results/metals-backtest-2026-07-25.md`.
- **NEW BOARD FAMILY: WTI + metals WEEKLIES** (26 resolved metals weeklies were invisible
  to us). Needed one fix — `board_period` is now class-aware for weeklies (WTI/metals
  Sun 22:00Z→Fri 21:00Z; equity unchanged). Gate-0 on weeklies **168/168**, incl. **28/28
  on WTI weeklies from our own WTIU6 archive** (first contract-feed validation, not proxy).
- **The 5 new week-of-Jul-27 boards (WTI/gold/silver/SPY/NVDA) have NO REAL BOOK** — every
  leg quotes 0.020/0.980. Onboarded as `active = false`, **no prediction rows** (a 0.50
  mid off a 96c spread would poison scoring). **Day-4 action: re-read the books Monday.**
- **August monthlies NOT listed yet** (checked wti/gold/silver/btc/eth + spy week-of-Aug-3).
  July monthlies run to Aug 1 03:59Z, so August lists ~Jul 31/Aug 1. **Check day-4/5.**
- **WTI sell signals (all TIER B, weekend books):** only two clear the $100 depth bar —
  ↑110 @5.4c ($478) and ↓75 @7.5c ($278). Day-2's deep ↓80 book collapsed $1132 → $24
  (mid rose 18.5c→21.5c, consistent with the delayed-fill drift finding). Others
  diagnostic: ↑105 @14.4c ($46), ↑100 @22.7c ($8), ↑115 @3.4c ($1), BTC ↓57.5k ($35).
  Spot U6 90.46, RV 51.7%, OVX 68.0. **No gold/silver signal today** — every fundable
  metals leg sits within ~2c of the model with 6 days left.
- Vol: OVX 68.97→68.00, VIX 18.70→18.58, GVZ 25.14→24.33, VXSLV 49.53→48.05 (all easing).
- **Day-4 queue**: (1) daily archive (**WTIU6 + WTIV6 both, every day until Aug 20**);
  (2) **re-read the 5 week-of-Jul-27 books Monday** → onboard + predict the fundable legs
  (this is where gold's earned edge should finally pay); (3) watch for August monthlies →
  the WTI one needs the **roll-aware model** before any prediction; (4) WTI July board has
  6 days left — 14 live rows resolve Jul 31 21:00Z; (5) consider a WTI-weekly backtest now
  that the contract archive is validated (proxy no longer required going forward).

## Medium-term

- **Where the edge is**: sell overpriced wings/extension legs; delayed sim beats instant
  for sells. Model beats market Brier on **WTI (−0.0133) and GOLD (−0.0189)** only;
  spy +0.0291, nvda +0.0197, btc +0.0044, eth +0.0083, silver −0.0060 (noise) are all
  predict-only. Sells by asset (delayed, per trade): wti +14.4c, spy +19.6c(n=18),
  gold +7.1c, eth +8.3c, btc +6.4c, silver +3.0c, nvda −4.8c.
- **Resolution epsilon (day-3, important)**: venue resolution error is **one-directional
  against sellers**. 279/279 legs where our feed shows a touch resolved YES (0 reversals,
  incl. 32 within 0.5% of the barrier); but 2/7 feed-*misses* within 0.10% of the barrier
  resolved YES anyway — SPY ↑750 (Pyth peaked **749.99002**; verified against the 5-second
  Pyth tape max 749.98993 and every aggregation 1min→daily; market closed 16:41Z on 07-22)
  and XAGUSD ↑69 (peaked 68.942). Both ↑ legs at round numbers. → **screen: never sell a
  barrier within 0.2% of that leg's running window extreme** (from its TRUE window start).
- **Book-quality gate (day-3)**: newly-listed boards quote 0.020/0.980 placeholders for
  days. Require spread ≤ 5c before predicting a leg at all.
- **Buys: unrescuable by drift/jump** (day-2, 255-leg sample). Failure is 100% crypto —
  the crypto market itself over-prices barrier touches ~4× (3–50c legs realized 0.038 vs
  mid 0.148); the model over-prices more, so "underpriced→buy" chases expensive legs.
  Class-gated WTI+equity buys are positive but only 12/39 legs → **buys stay OFF**.
- **WTI roll (verified day-3)**: CLU6 active for every session Jul 1 → **Aug 17**; CLV6
  from the **Aug 18** session (CLU6 LTD Thu Aug 20 = 3 bizdays before Aug 25). July
  monthly + week-of-Jul-27 = **CLU6 only, no roll**. The **August monthly SPANS the roll**
  and the CLU6−CLV6 spread went +$0.19 (Jul 1) → **+$4.78 (Jul 24)**: the resolving series
  gaps DOWN ~5% mid-board (↓ barriers much easier, ↑ much harder). Driftless GBM on U6
  would badly misprice it. Also: WTI rallied **67 → 93** during July (real, verified).
- **Fine print (gate-0 verified, now 756/760 over 760 legs)**: crypto monthlies resolve on
  BINANCE USDT; "calendar month" legs resolve from listing; re-added strikes carry private
  window starts ("after market creation") — every metals weekly from week-of-June-8 onward
  has the clause, earlier ones do not; equity RTH-only; WTI/metals weeklies use the
  session clock, not the calendar week.
- **Data traps**: expired WTI per-contract Pyth feeds are DELISTED (only WTIU6 + WTIV6
  exist now — WTIX6 already 404s) — archive daily. USOILSPOT proxy basis p50 $0.21 p95
  $0.98 max $2.07; it caused 2 of the 4 gate-0 misses — never settle a near-barrier WTI
  question off it. Pyth benchmarks rate-limits (~1 req/s); its candle data is **stable on
  refetch** (verified, 0/392 candles changed) so the archive is authoritative. Polymarket
  headline volumeNum on WTI ≈ 20× real taker notional — capacity from book depth only.
- Vol anchors verified free: CBOE OVX/VIX/GVZ/VXSLV CSVs, Deribit DVOL. NVDA has none.
- Session calendars assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3); revisit for
  Sep (Labor Day) and the EST transition if the variant lives that long.

## Long-term (wiki candidates)

- Favorite-longshot INSIDE one-touch ladders, now confirmed on metals independently:
  51 metals legs quoted under 10c, **0 touched**; 80–95c bin hit 95.5%. Across all assets
  the 2–5c bin hit 0/58. Candidate for `wiki/reference/favorite-longshot-bias.md`.
- Delayed execution can IMPROVE sell fills: one-way lottery flow drifts wing prices up —
  the opposite of the runningmax speed-race decay. Observed live again day-2→day-3 (WTI
  ↓80 mid 18.5c→21.5c). Candidate `wiki/reference/delayed-execution-test.md`.
- **Venues resolve generously at round numbers**: a barrier the official feed misses by
  <0.1% still resolves YES ~29% of the time, while a feed-touch is never reversed. Any
  strategy selling barrier touches inherits a small one-directional tail. Strong wiki
  candidate — it generalises past this variant.
- **"Active month" ladders spanning a CME roll contain a deterministic price gap** equal
  to the calendar spread. Wiki candidate for anyone pricing futures-referenced barriers.
- Resolution-feed archaeology matters: resolution sources get deleted (Pyth contract
  feeds). Archive the resolving feed while the market is live, or lose gate-0 forever.
