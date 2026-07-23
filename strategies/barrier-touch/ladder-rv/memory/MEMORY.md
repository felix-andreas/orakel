# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- Day-1 (2026-07-23) backtest DONE: gates alive, method fixed to **sell-only, RV-primary**
  (`results/backtest-2026-07-23.md`). 18 prediction rows handed to CEO: 15 WTI-July
  full ladder + SPY L715/L720 + NVDA L192 (weeklies resolve Fri 2026-07-24 20:00Z —
  check scoring day 2!). WTI board resolves Jul 31 21:00Z.
- Trades flagged (sell-side): WTI ↑$110 @ 9.3c (tier A), ↑$105 @ 16.6c (tier B).
- Day-2 queue: (1) score Friday weeklies + new week-of-Jul-27 boards (discover Friday
  22:0xZ listing); (2) daily candle archive incl. WTIU6 (expired Pyth contract feeds
  get DELETED — if we skip days we lose the resolution record); (3) widen: gold/silver/
  natgas monthlies + more equity weeklies (need Pyth symbol map + IV anchors GVZ/VXSLV?);
  (4) buy-side rescue idea: drift/jump model or use market-implied drift; (5) chase
  transient panic in `tape` subcommand (rerun works).
- BTC July board PARKED (market beats model on crypto, all legs in RV/IV straddle).

## Medium-term

- **Where the edge is**: sell overpriced wings/extension legs; delayed sim +10c/trade,
  +4.2c/leg (se 3.8c) — NOT yet significant per-leg; forward trial must confirm. 11/13
  boards positive. Buys lose (crypto buys −17.5c/trade): deep books embed info the
  driftless model lacks. Model beats market Brier ONLY on WTI (0.048 vs 0.058).
- **Fine print (gate-0 verified)**: crypto monthlies resolve on BINANCE USDT (not Pyth);
  "calendar month" original legs actually resolve from listing; re-added strikes carry
  private window starts ("from creation of this market"); equity RTH-only; WTI
  active-month roll = 3 sessions before LTD. 251/255 reproduced; misses are epsilon.
- **Data traps**: expired WTI per-contract Pyth feeds are DELISTED from Benchmarks
  (USOILSPOT proxy basis p50 $0.13-0.21, p95 $0.42-0.65, max $2.07 — never settle a
  near-barrier WTI question off the proxy). Pyth benchmarks rate-limits (~1 req/s ok).
  Polymarket headline volumeNum on WTI boards ≈ 20× real taker notional — capacity from
  book depth only. Tape endpoint sometimes needs a re-run for resolved boards.
- Vol anchors verified free: CBOE OVX/VIX CSVs, Deribit DVOL (public). NVDA has none —
  RV only, wider internal spread. IV ≈ 1.2–1.5× RV14d across assets currently.
- Session calendars in code assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3);
  revisit for Sep (Labor Day) and EST transition if variant lives that long.

## Long-term (wiki candidates)

- Favorite-longshot INSIDE one-touch ladders quantified: open-mid 2–5c bin hit 0/27,
  5–10c 4%, while 80–95c hit 94% — wing premium ≈ 2.6× realized touch losses
  (gate-3). Candidate addition to `wiki/reference/favorite-longshot-bias.md`.
- Delayed execution can IMPROVE sell fills (+7.7c→+10.0c at t+24h): one-way lottery
  flow drifts wing prices up — the opposite of the runningmax speed-race decay.
  Candidate note for `wiki/reference/delayed-execution-test.md`.
- Resolution-feed archaeology matters: resolution sources get deleted (Pyth contract
  feeds). Archive the resolving feed while the market is live, or lose gate-0 forever.
