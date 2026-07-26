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
   beyond". **Weekly boards follow the asset's own session clock, not a calendar week**
   (fixed 2026-07-25): equity weeklies Mon 00:00Z → Fri 20:00Z, WTI/metals weeklies
   Sun 22:00Z → Fri 21:00Z. Gate-0 on the metals + WTI weeklies: 168/168.
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
   **Resolution-epsilon screen (added 2026-07-25):** never open a sell whose barrier sits
   within **0.2% of that leg's running window extreme**, measured from the leg's TRUE
   window start. Venue resolution error is one-directional against sellers: across 760
   clean-feed resolved legs, 279/279 feed-touches resolved YES (0 reversals, including 32
   inside 0.5%), but 2/7 feed-*misses* inside 0.10% of the barrier resolved YES anyway
   (SPY ↑750 — Pyth peaked 749.99002, verified against the 5-second tape; XAGUSD ↑69 —
   peaked 68.942). Both were ↑ legs at round numbers.
   **Book-quality gate (added 2026-07-25, tightened 2026-07-26):** a leg needs a genuine
   two-sided book before it is predicted at all. Freshly-listed weekly boards quote
   0.020/0.980 placeholders for their first days; a 0.50 "midpoint" off a 96c spread is
   not a market price and must never enter a prediction row. Three tests, all required:
   (a) **relative spread** `spread ≤ min(5c, ½·mid)` — a flat 5c bar is vacuous on a wing
   (August ↓20 quotes 0.003/0.019: a 1.6c spread "passes" on a book whose mid is 3.8× its
   bid); (b) **mid ∈ [3c, 97c]**; (c) **tape gate (new)** — at least one taker trade on
   the side we would take, within 5c of the quote, in the last 7 days. (c) exists because
   the NVDA week-of-Jul-27 ladder quotes 1–5c wide on six legs and has **zero trades ever**
   on five of them: a market maker quoting into an empty room passes every spread test
   there is. See `results/book-and-tape-audit-2026-07-26.md`.
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

Board families: each asset lists **monthly** and **weekly** ladders (the WTI/metals
weekly family was found 2026-07-25 — 26 resolved metals weeklies plus WTI weeklies were
invisible to us until then). Weeklies resolve in ≤5 sessions, so they are the preferred
trial vehicle.

**WTI active-month rolls (corrected 2026-07-26 — the 07-25 version was wrong about July).**
The rule is in every board's fine print: the next contract becomes active at the start of
the **third-from-last** session of the nearest one, whose LTD is 3 business days before
the 25th of the month preceding delivery (4 if the 25th is not a business day). Applied,
and cross-checked against Pyth's own feed names:

- **CLQ6 → CLU6 at the session for Fri 17 Jul** (2026-07-16 22:00Z): 25 Jul is a Saturday
  → CLQ6 LTD Tue 21 Jul. So the **July monthly board and the week-of-Jul-13 weekly DO span
  a roll** — our gate-0 mirror used WTIU6 for their CLQ6 halves. It changed no answer
  (CLQ6 ran ~67–81 against barriers ≤65 / ≥95) but it was luck. `WTIQ6` is already
  delisted and unrecoverable.
- **CLU6 → CLV6 at the session for Tue 18 Aug** (2026-08-17 22:00Z); CLU6 LTD Thu 20 Aug
  (Pyth: "PYTH WTI 20 AUGUST 2026"). The **week-of-Jul-27 board is CLU6-only**; the
  **August monthly spans this roll**, with the CLU6−CLV6 spread out from +$0.04 (Jul 1) to
  **+$4.58 (Jul 24 close)**.
- **CLV6 → CLX6 at the session for Fri 18 Sep**; CLV6 LTD Tue 22 Sep. The September
  monthly spans it. `WTIX6` does not exist on Pyth yet — **archive it the day it appears**
  (~Aug 20, when U6 expires and is deleted). Pyth carries exactly two CL contracts.

Roll-aware pricing is implemented (`ladderrv roll`, validated by `ladderrv selftest`):
model ln V (deferred) as the primitive, link the front by `ln U = ln V + k0 + β·Δln V`
with β ≈ 0.15 from the U/V regression, and price a barrier that **steps at the roll** —
`V0·(B/U0)^(1/(1+β))` before, `B` after, with absorption *at* the roll instant for paths
between the two. Derivation, calibration and the August table:
`results/august-roll-model-2026-07-26.md`. The naive one-spot model under-prices every
August ↓ leg by 40–110% relative (↓80: 0.365 → 0.508) — the single most dangerous error
available to us, since ↓ legs are what we sell.

Feed-mirror status: crypto (Binance), WTI (Pyth active-month CL + WTIU6 **and WTIV6**
archives), equity SPY/NVDA (Pyth RTH), and metals gold/silver (Pyth `Metal.XAU|XAG/USD`,
IV anchors GVZ/VXSLV) are all mirrored. **Natural gas is NOT viable yet**: NG boards
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

- `results/book-and-tape-audit-2026-07-26.md` — the fill question re-asked per board
  family over all 70 markets we have predicted on. Reachable fraction of the scored
  midpoint: **WTI 99%, BTC 100%, silver 89%, gold 82%, SPY/NVDA weekly 38%**. The 2/21
  headline was a fact about equity weeklies and sub-3c wings, not about the variant.
  Also: the tape gate, the 07-31 identity check (70/70 clean), and the
  `?condition_ids=` + `closed=true` scoring hazard.
- `results/august-roll-model-2026-07-26.md` — roll-aware two-segment pricer, its
  validation, the CLQ6/CLU6/CLV6/CLX6 roll calendar, and why August is priceable but not
  predictable today.
- `results/metals-backtest-2026-07-25.md` — day-3 metals gates on 441 resolved gold/silver
  legs (31 boards): gate 0 440/441; per-asset Brier table (gold best, equity/crypto worse
  than market); delayed sell sim by asset; the resolution-epsilon table; gold earned,
  silver denied.
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
- 2026-07-25 — day-3: **metals backtest** on 441 resolved legs → **gold upgraded to
  tradeable, silver stays prediction-only** (`results/metals-backtest-2026-07-25.md`).
  Method changes: (a) weekly boards now use the asset's session clock (`board_period`
  class-aware) — this exposed the WTI/metals **weekly** board family, gate-0 168/168;
  (b) **resolution-epsilon screen** (0.2% of running extreme) after finding venue
  resolution is one-directionally adverse to sellers; (c) **book-quality gate** (spread
  ≤ 5c) after freshly-listed boards were found quoting 0.020/0.980 placeholders.
  WTI Aug-monthly roll documented (CLU6→CLV6 at the Aug 18 session, spread +$4.78);
  WTIV6 archiving started. New applications: WTI/metals/equity week-of-Jul-27 (all
  listed, all awaiting a real book).
- 2026-07-26 — day-4: **roll-aware pricer built and validated** (`ladderrv roll`,
  `selftest`); roll calendar corrected — the **July** monthly spans a roll too. Two code
  defects found by reading: `SessionCal` stopped at 2026-08-20 (τ truncated to 14 of the
  August board's 21 sessions, σ√τ 18% low — **fixed**), and `cmd_live` does not diffuse
  spot from now to a future window open (**not fixed**). **Book gate tightened** to a
  relative spread plus a **tape gate**, after the NVDA week-of-Jul-27 ladder was found
  quoting 1–5c wide with zero trades ever. Fill evidence re-cut per board family: the
  commodity monthlies reach 82–100% of the scored midpoint, the equity weeklies 38%.
  **13 prediction rows** (WTI/gold/silver July monthlies only); **zero** on August and
  zero on every week-of-Jul-27 board.
