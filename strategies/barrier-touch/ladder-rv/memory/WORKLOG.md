# barrier-touch/ladder-rv — Worklog

One dated entry per run. Name the model that did the work.

---

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
