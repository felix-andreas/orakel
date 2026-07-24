# barrier-touch/ladder-rv

> Thesis (from `ideas/2026-07-23-hit-price-ladder-rv.md` — read it fully): price
> "Hit Price" one-touch ladders with a first-passage model (spot from the resolving
> feed, σ from realized vol + listed-IV anchor) and fade the overpriced side: wing
> lottery legs, mid-window strike additions with private window starts, equity
> RTH-only fine print. Mispricing is model-revealed, not print-revealed — harvested
> by holding 1–9 days to resolution, no speed race (delayed-exec verified).

## Method (as-built 2026-07-23, day-1 backtest: `results/backtest-2026-07-23.md`)

1. **Resolution model per leg** (gate-0 verified 251/255): window =
   [max(period start, leg listing), period end]; crypto boards resolve on **Binance**
   USDT 1-min candles 24/7; equity boards on Pyth RTH 13:30–20:00Z weekdays; WTI on
   Pyth active-month CL futures, sessions 22:00Z→21:00Z Mon–Fri with the CME roll
   (next contract becomes active 3 sessions before LTD). **Metals** (gold XAUUSD /
   silver XAGUSD, added 2026-07-24) resolve on the continuous Pyth `Metal.XAU|XAG/USD`
   spot feed with a COMEX session (6pm ET Sun→5pm ET Fri, daily 5-6pm ET break) that is
   structurally identical to WTI's 22:00Z→21:00Z model — mapped to `Class::Wti` in code,
   no per-contract delisting. Touch = candle high ≥ B (↑) / low ≤ B (↓), "equal to or
   beyond".
2. **Touch probability**: driftless GBM in session time,
   P = 2·N(−|ln(B/S)|/(σ√τ)), τ = remaining session minutes / class minutes-per-year.
   σ primary = trailing-14d realized (5-min closes, session-annualized) from the same
   candle source that resolves; σ secondary = IV anchor (DVOL/VIX/OVX/GVZ/VXSLV;
   NVDA none).
3. **Predict** q_rv on active boards' legs with two-sided books; record CLOB midpoint.
   Model beats market Brier on WTI only (0.048 vs 0.058) — WTI boards get full-ladder
   predictions; equity/crypto boards only sell-edge legs (or parked). **Metals
   (gold/silver, added day-2)**: prediction-only pending a metals backtest — the live
   snapshot shows the model at/above market mid across the fundable 3–50c zone (no sell
   edge; the wing overpricing is confined to un-fundable sub-3c legs), so trades stay
   off until resolution tells us whether the model beats market on metals Brier.
4. **Trade signals: SELL-ONLY.** Sell YES when mid ∈ [3c, 50c], q_rv < mid − (spread+2c),
   barrier outside the 1-session-σ exclusion zone. Confidence tier A when q_iv agrees
   (also < mid − spread − 2c), tier B when RV only. **Buys disabled** (delayed sim:
   −7.3c/trade, crypto buys −17.5c) until a drift/jump-aware model earns them.
5. **Sizing/capacity**: read book depth, never headline volume (WTI headline = 20× real
   taker flow). Sub-3c legs and $<100 top-of-book books are diagnostics, not trades.

Backtest support (13 resolved boards, May–Jul 2026, t+24h delayed fills with frozen
inputs): sells +10.0c/trade (se 1.6, n=282), both halves positive, +4.2c/leg after
per-leg collapse (62/74 legs, 11/13 boards positive); wing premium 2.6× realized touch
losses. Not yet significant per-leg — the forward trial is the out-of-sample test.

## Applicability

A market fits when: it is a leg of a Hit Price ladder whose resolution feed we can
mirror (Pyth equity/WTI/**metals**, Binance crypto), quotes in the fundable 3–50c zone
with a two-sided book. Onboarding = `applications/<board>.toml` (asset, resolution feed
symbol, window end, IV source, per-leg barrier/direction/TRUE window start for
tradeable legs). Beware "after market creation" fine print — re-added strikes carry
private window starts; the board title lies.

Feed-mirror status: crypto (Binance), WTI (Pyth active-month CL + WTIU6 archive),
equity SPY/NVDA (Pyth RTH), and metals gold/silver (Pyth `Metal.XAU|XAG/USD`, IV
anchors GVZ/VXSLV) are all mirrored. **Natural gas is NOT viable yet**: NG boards
resolve on the *active-month* NG futures contract (per-contract, same delisting risk as
WTI) and the Pyth TV-shim returns `s=error` for the `Commodities.NGD*` symbols with no
continuous spot proxy and no free IV index — candidate sibling-variant work, not a
params-only add.

## How to run

```
cargo build --release   # in this folder
ladderrv discover <data> <board-slugs>          # Gamma events -> legs.csv
ladderrv candles <data> <KEY> <from> <to>       # 1-min candles (BTCUSDT|ETHUSDT|USOILSPOT|SPY|NVDA|WTIU6|XAUUSD|XAGUSD)
ladderrv vol <data>                             # OVX/VIX/DVOL anchors
ladderrv clob <data> 60 [boards]                # CLOB prices-history per leg
ladderrv analyze <data>                         # gates 0-3 + violation lifetimes
ladderrv tape/wash <data> <board>               # gate 4
ladderrv live <data> <board-slugs>              # books + model -> prediction rows CSV
```

Daily: pull candles for **yesterday+today** (all keys incl. WTIU6 — expired Pyth
contract feeds are DELETED, we must archive the active month ourselves; force a complete
refetch of yesterday, which was partial when captured), `vol`, then `live` on active
boards; freeze the candle+vol archive to R2 via r2data before committing the manifest
(`data/candles-<date>.tar.gz.r2.json`) — that frozen archive is the resolution record.

## Evidence

- `results/backtest-2026-07-23.md` — day-1 gates: gate 0 251/255 (misses = proxy/epsilon);
  favorite-longshot calibration table (2–5c bin: 0/27 hit); delayed-exec sell edge;
  attribution ratio 0.38; WTI-proxy basis measured; wash checks; coherence violations
  p50 20 min (mechanism 3 diagnostic-only, as the idea suspected).

## Changelog

- 2026-07-23 — variant created from the idea (run 2); slot 1 trial started.
- 2026-07-23 — day-1 backtest: pipeline built, gates run, method fixed to sell-only
  RV-primary; applications: WTI July (active), SPY/NVDA week-of-Jul-20 (active,
  sell-edge legs), BTC July (parked — market beats model on crypto).
- 2026-07-24 — day-2: daily resolution-feed archive frozen (07-23 completed + 07-24);
  live re-run (39 handoff predictions across 5 assets, 17 resolving same-day). WIDEN:
  symbol map extended for metals (gold XAUUSD / silver XAGUSD → `Class::Wti`, IV anchors
  GVZ/VXSLV); gold+silver July monthlies added as **prediction-only** applications (no
  fundable-zone sell edge; books thin). Natgas deferred (per-contract feed, shim errors).
  WTI sell signals live (all tier B now — OVX rose to 69% and closed the RV/IV gap).
