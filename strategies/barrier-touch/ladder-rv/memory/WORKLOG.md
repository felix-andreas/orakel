# barrier-touch/ladder-rv — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-24 — day 2: daily archive, live re-run, metals widen, buy-side experiment (model: opus (xhigh))

- **Candle archive (critical daily duty)**: restored day-1 archive from R2, force-refetched
  yesterday (07-23, was partial) + today (07-24) for all keys; 07-23 now COMPLETE
  (WTIU6 full 1381-min session archived — delisting insurance). Refreshed OVX/VIX/DVOL
  (+ new GVZ/VXSLV). Froze candles+vol (8 keys) → `data/candles-2026-07-24.tar.gz.r2.json`
  (15MB) and a live snapshot → `data/live-2026-07-24.tar.gz.r2.json`; both R2-verified.
- **Live re-run**: 39 handoff predictions across 5 assets (WTI 15 full ladder, SPY 9 +
  NVDA 8 sell-wings, gold 3 + silver 4); 17 resolve same-day (SPY/NVDA 20:00Z). Handed
  to CEO in the report (did NOT write predictions.csv). WTI spot 90.00→91.60, OVX 65→69
  → all WTI sell signals now tier B (day-1 tier-A ↑110 demoted); deepest tradeable = ↓80
  @18.5c ($1132 book).
- **Widen (metals)**: extended `ladderrv` — Asset::Gold/Silver (xauusd/xagusd) → Class::Wti
  (COMEX session == WTI 22:00Z→21:00Z, verified from market fine print), candle keys
  XAUUSD/XAGUSD (Pyth Metal.XAU|XAG/USD, continuous — no delisting), IV anchors GVZ/VXSLV.
  Added gold+silver July-monthly applications as PREDICTION-ONLY (no fundable-zone sell
  edge — model sits at/above market mid; thin books). Natgas deferred (per-contract
  active-month feed, Pyth shim errors, no free IV) → noted as candidate sibling work.
- **Buy-side rescue experiment (negative result)**: ran a drift-augmented first-passage
  model over the 255-leg resolved checkpoint sample (gate2_checkpoints.csv). No drift μ
  rescues crypto buys (root cause: market over-prices crypto touches ~4×, realized 0.038
  vs mid 0.148). Class-gated WTI+equity buys are +12.8c/tr but only 12/39 legs positive
  (fragile) → buys stay disabled. STRATEGY.md updated; buys unchanged.
- **Housekeeping**: tape "panic" diagnosed = transient data-api 429/5xx aborting a
  mid-board gate-4 run (rerun resumes via cached files); left validated gate code
  untouched. Week-of-Jul-27 + August boards not yet listed (01:29Z Fri) → flagged day-3.
- Escalation to CEO: none. Trial on track — 17 scored predictions land today across
  SPY/NVDA, plus WTI/gold/silver at Jul-31 (≥15-across-≥3-boards guideline comfortably met).

## 2026-07-23 — day 1: backtest-first, gates run, method fixed (model: fable (high))

- Gate-0 derisk first: Pyth Benchmarks TV-shim serves 1-min candles ≥1yr back for
  BTC/SPY/NVDA/USOILSPOT; **expired WTI contract feeds are delisted** → USOILSPOT proxy
  for backfill (basis measured), WTIU6 archived live from now on. Crypto boards resolve
  on Binance (fine print), pulled via data-api.binance.vision.
- Built `ladderrv` crate (discover/candles/vol/clob/analyze/tape/wash/live), reusing
  runningmax's fetch/CLOB patterns. 17 boards discovered (319 legs; 13 fully resolved
  = 243-leg gate sample). All data frozen: `data/backtest-2026-07-23.tar.gz.r2.json`.
- Gates (full numbers `results/backtest-2026-07-23.md`): gate 0 = 251/255 (all misses
  epsilon/proxy); window-open calibration shows textbook wing overpricing (2–5c bin
  0/27 hit); t+24h delayed sim with frozen inputs: **sells +10.0c/trade (se 1.6),
  both halves positive, +4.2c/leg per-leg collapsed; buys −7.3c → disabled**; gate 3
  ratio 0.38 (premium ≫ jump cost); violations p50 20min → mechanism 3
  diagnostic-only; wash: no disqualifier, WTI headline volume 20× real.
- Honest caveats logged: per-leg edge not yet significant (se 3.8c), one regime,
  equity/crypto ladders price better than the model (predict WTI fully, others
  sell-edge only).
- Applications: WTI-July active (sells ↑110 tier A, ↑105 tier B), SPY+NVDA
  week-of-Jul-20 active (wing sells, resolve Fri), BTC-July parked. 18 prediction
  rows to CEO (never wrote predictions.csv myself).
- Escalation to CEO: none needed — no kill, no blocker. Note for exec design:
  fills at t+24h were BETTER than instant for sells; daily cadence suffices.
