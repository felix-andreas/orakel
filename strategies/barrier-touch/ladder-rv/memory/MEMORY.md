# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **Day-2 (2026-07-24) DONE.** Archive frozen to R2 (`data/candles-2026-07-24.tar.gz.r2.json`,
  8 keys incl WTIU6 + new XAUUSD/XAGUSD; 07-23 now COMPLETE; + `live-2026-07-24` snapshot).
  **39 handoff predictions across 5 assets** (WTI 15 full ladder + SPY 9 + NVDA 8 sell-wings
  + gold 3 + silver 4). **17 resolve TODAY 20:00Z (SPY+NVDA) — SCORE them day 3.** WTI +
  gold + silver resolve Jul 31 21:00Z.
- Vol moved: **OVX 65.3→69.0, VIX 16.6→18.7** (both up; VIX intraday hi 20.3 on 07-23);
  DVOL flat; GVZ 25.1, VXSLV 49.5.
- **WTI sell signals (all TIER B now** — OVX↑ closed the RV/IV gap; day-1's tier-A ↑110
  demoted): tradeable (book>$100) ↓80 @18.5c (**deep $1132 book, best capacity**; was
  day-1 "watch only", spot rose to 91.6 → clean sell), ↑110 @8.1c ($665), ↑100 @37c
  ($191, but q_iv 0.40 disagrees + near-money → low conviction), ↑105 @17c ($123),
  ↓75 @6c ($550, wide 2c spread). Tier-A ↑115 @5.5c blocked by thin $57 book. Spot U6
  91.60 (↑ from 90.00), RV 52.8%.
- **WIDEN**: metals mapped (gold XAUUSD/silver XAGUSD → Class::Wti, GVZ/VXSLV anchors);
  gold+silver July monthlies added **PREDICTION-ONLY** (model at/above market mid across
  fundable zone → no sell edge; thin books; classify vs market at Jul-31). Natgas
  DEFERRED (per-contract active-month feed, Pyth shim `s=error`, no free IV).
- **Buy-side rescue = negative result.** Drift μ sweep (−0.5..+0.5) does NOT rescue crypto
  buys (all −14 to −19.5c). Root cause: market itself over-prices crypto touches ~4×
  (3–50c zone: realized touch 0.038 vs mid 0.148) → model "buy" chases expensive legs.
  Class-gated WTI+equity buys +12.8c/tr BUT only **12/39 legs positive** (fragile vs
  sells' 62/74) → **buys STAY DISABLED**; the lever is asset gating, not drift.
- BTC July board PARKED (re-checked day-2: every leg still in RV/IV straddle).
- **Day-3 queue**: (1) SCORE the 17 same-day equity predictions (resolve today 20:00Z);
  (2) **discover new boards** — week-of-Jul-27 SPY/NVDA + August monthlies (WTI/gold/
  silver/BTC-Aug) NOT listed yet at 01:29Z Fri → list Fri eve/weekend; (3) daily candle
  archive (now incl XAUUSD/XAGUSD); (4) **metals backtest** (gate-0/1/2 on resolved
  gold/silver monthlies) to earn/deny metals trades; (5) tape transient (data-api 429
  mid-board abort, rerun resumes, gate-4 only) — harden only if it recurs.

## Medium-term

- **Where the edge is**: sell overpriced wings/extension legs; delayed sim +10c/trade,
  +4.2c/leg (se 3.8c) — NOT yet significant per-leg; forward trial must confirm. 11/13
  boards positive. Model beats market Brier ONLY on WTI (0.048 vs 0.058).
- **Buys: unrescuable by drift/jump (day-2 experiment on the 255-leg resolved sample).**
  Failure is 100% crypto — 142 crypto buy signals, realized touch 0.000, model q~0.305.
  Mechanism: the crypto market itself over-prices barrier touches ~4× (3–50c legs realized
  0.038 vs mid 0.148); the model over-prices even more, so its "underpriced→buy" call
  chases legs expensive vs reality. No μ∈[−0.5,+0.5] flips it positive. WTI+equity buys
  ARE positive (+22.8c / +5.3c/tr) but class-gated buys collapse to 12/39 legs positive
  (skewed, a few WTI winners) → far less robust than sells (62/74) → keep buys OFF. The
  buy lever is asset gating, not a drift term.
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
