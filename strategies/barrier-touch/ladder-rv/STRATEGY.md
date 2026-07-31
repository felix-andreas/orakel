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
contract feeds are DELETED, we must archive the active month ourselves), `vol`, then `live`
on active boards; freeze the candle+vol archive to R2 via r2data before committing the
manifest (`data/candles-<date>.tar.gz.r2.json`) — that frozen archive is the resolution
record. Since 2026-07-28 the refetch of an incomplete yesterday is **automatic**: a day-file
is re-pulled unless it was written after that day ended.

**Do not hand-write the freeze. Run `bash scripts/freeze.sh <date>`.** `data/.gitignore` ignores
`candles/ vol/ out/ tape/ clob*/`, so anything not in a tarball exists nowhere. The script holds
the required-contents manifest in git, cuts **both** archives (`candles-<date>` = candles+vol,
`live-<date>` = events_live+out+legs.csv+tape+clob), and **re-reads each tarball it just built**,
failing if a promised entry is missing — because `r2data verify` cannot see inside a tarball.
This exists because the duty used to be a `tar` line retyped every morning, and it went wrong
twice: day-6 cut only the candles freeze, so `predictions_2026-07-28.csv` survived in one
container (`results/archive-audit-2026-07-29.md`), and **day-4 cut no `live-*` freeze at all, so
`predictions_2026-07-26.csv` is permanently lost** (`results/archive-audit-2026-07-30.md`).

**Run `python3 scripts/resolve_sweep.py` daily, not just on resolution day.** It unions both
Gamma query forms, asserts the returned `conditionId` matches the one asked for, and treats
`closed` with non-final `outcomePrices` as UNSETTLED. On 2026-07-30 it found **3 ledger rows on 2
markets that had resolved YES on 07-29** and were in no plan and no `resolutions.csv`: every
completeness check we had asked "is the archive complete as of the last run", and none asked
**"did something resolve while we weren't looking"**. A market can leave the outstanding set
without any run touching it.

**A `verify` FAIL can be a transient R2 500.** Retry it and confirm with `r2data pull` before
concluding an archive is lost — re-freezing over an archive you wrongly believe is broken is the
destructive reflex.

**`live` takes ONE comma-separated argument**, not a space-separated list —
`ladderrv live data "slug-a,slug-b"`. Space-separated silently prices only the first board
and writes a prediction file that looks complete. `cmd_live` also **overwrites**
`data/out/predictions_<date>.csv` on every invocation, so run every board in one call.

**Checkpoint discipline (settled 2026-07-27, `results/legsum-null-and-stale-feed-2026-07-27.md`).**
This family's legs are **nested**, not mutually exclusive, so the wiki's `leg-sum ≈ 1` gate
is vacuous; the equivalent checkable quantity is `Σmid` (the market's expected YES count)
against `Σwinner`. Report it beside every headline. **Never anchor a checkpoint at board
creation** — 85% of legs quote a mid in [45c, 55c] there and a flat base-rate beats the
market's own log-loss. Window-open and daily-12:00Z both beat every null in every asset,
so the trial's numbers stand; but gate board-snapshots at `avg_mid ≤ 0.40` before quoting a
Brier margin, because the **pooled window-open** margin reverses under that gate.

**Stale-feed blindness (found 2026-07-27 by losing `will-wti-dip-to-85-in-july-2026`).**
`q` is a function of (spot, σ, τ). The WTI/metals feed is shut Fri 21:00Z → Sun 22:00Z, so
on a Saturday or Sunday run the calendar freezes spot and τ and the model **cannot update**,
while the CLOB trades throughout. On 2026-07-26 the book moved 0.475 → 0.715 across that
closure and CLU6 then opened −7.79% through the barrier. Solving the market's quote for spot
gives 87.3–88.0 against our 90.46: the market was pricing **a lower level**, which no vol
model recovers from a stale close. Mitigations now in code: `cmd_live` reports feed age and
prints a `STALE FEED` banner, and `touch_prob_jump` prices the coming close-to-open gap as a
lump (WTI weekend gap sd **3.78%**, ≈ a whole session's variance; RTH-equity *overnight* gap
≈ a whole session's variance). **Proposed and awaiting the CEO: a stale-feed gate** — do not
treat disagreement with the market as edge when the feed has been shut for the whole period
over which the market moved.

## Evidence

- `results/legsum-null-and-stale-feed-2026-07-27.md` — the leg-sum / null-model re-check
  (creation anchor fails, window-open and daily-12Z clear in every asset; gold's window-open
  margin does not survive the gate, its daily margin does), and the full reconstruction of
  the `dip-to-85` loss: the stale-feed mechanism, the measured close-to-open gap table, and
  the jump-aware pricer that closes both `cmd_live` defects.
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
- 2026-07-27 — day-5: **leg-sum / null-model re-check run** — no artifact at the anchors we
  use, but the **creation** anchor fails outright and **gold's window-open Brier margin does
  not survive a leg-sum gate** (gold stays tradeable on its daily-checkpoint margin,
  −0.00541, t −3.55). `will-wti-dip-to-85` resolved YES against us: traced to a **shut
  resolving feed**, not calibration. Both remaining `cmd_live` defects fixed by one
  `touch_prob_jump` model, with `realized_vol_intraday` + `gap_sd` and a new `ladderrv gaps`
  subcommand; `cmd_live` now reports feed staleness. Daily archive freeze restored.
  **8 prediction rows**; identity 51/51 clean for 07-31.
- 2026-07-28 — day-6: first run under the **stale-feed gate**. WTI/gold/silver feed OPEN
  (0.1h) → priced; SPY/NVDA feed SHUT (5.4h) → **all 22 equity legs suppressed**, and
  structurally: the RTH feed is *always* shut at the 01:07Z trigger, so the daily run can
  never predict an equity board. **14 prediction rows** (WTI 5, gold 2, silver 1, gold-weekly
  4, silver-weekly 2). Two code fixes: `cmd_candles` silently kept a **partial yesterday**
  forever (07-27 WTIU6 21.9KB vs a true 69.7KB; SPY/NVDA were 52-byte `no_data`) — a day-file
  is now refetched unless written after that day ended; and rows carry `pricer_version`,
  plus `q_iv`/`q_blend`/`sigma_*` for the pre-registered RV/IV comparison
  (`results/prereg-rv-iv-blend-2026-07-28.md` — a comparison scored 07-31, **not** a switch:
  IV sits above RV on 62/62 legs, which for a sell-only variant cuts sell signals 4 → 1).
  Friday readiness: `results/friday-2026-07-31-readiness.md`, identity **58/58**, 120 rows.
- 2026-07-29 — day-7: **the third data bug was not in code** — `data/out/` is gitignored and the
  day-6 freeze was candles+vol only, so `predictions_2026-07-28.csv` was frozen nowhere
  (`results/archive-audit-2026-07-29.md`); rescue-frozen. `cmd_clob` and `cmd_tape` had
  `cmd_candles`' `exists()`-means-cached bug — both fixed via `complete_through`. The
  "structurally short downside touch" hypothesis **refuted** on 633 legs / 5,927 checkpoints
  (`results/trend-exposure-2026-07-29.md`): the model **beats** the market on touched legs and
  WTI ↓ legs trending into the barrier are its **best** bucket; what is real is a one-sided tail
  whose 8 worst legs are all `dip-to`. **12 prediction rows.**
- 2026-07-30 — day-8, **last run before the evidence froze**. **5 prediction rows** (WTI 3 incl. a
  newly relisted ↓80-from-jul-29, silver-weekly 2) from 83 legs; 78 suppressed, 19 of them the
  structurally-shut equity feed. Two more archive holes
  (`results/archive-audit-2026-07-30.md`): **3 ledger rows on 2 markets had resolved YES on 07-29
  and were in no `resolutions.csv`** — nothing was watching for a leg that settles between runs —
  and **`predictions_2026-07-26.csv` is permanently lost** (day 4 cut no `live-*` freeze).
  **Fourth silent-data bug fixed**: Gamma's `closedTime` is not RFC3339, so `closed_time` had been
  `0` for all 74 closed legs forever; `parse_iso` widened, selftest asserts all three formats,
  pricer untouched. Root-caused the hand-written `tar` line into `scripts/freeze.sh` (verifies its
  own contents) and the both-ways lookup into `scripts/resolve_sweep.py` (asserts identity).
  Corrected my own 07-29 claim: **OVX never fell below the σ the pricer actually uses** (intraday
  RV, not RV14) — the prereg premise never softened. Pricer split reaches **40 rows / 19 markets**
  in the jump arm: clears 30 in rows, not in markets, reported **INCONCLUSIVE** as pre-settled.
  **08-02 prepared as a sizing question** (`results/sizing-2026-08-02-prep.md`): the sell-side
  break-even bound **clears at nominal n=356 and fails at effective n=173** (ρ = 0.325 within
  monotone families), and the tail is a **cliff** — the WTI down-ladder is net +0.49 at the realised
  −14% and −5.81 three points lower. Friday procedure: `results/friday-2026-07-31-runbook.md`.
  Wiki: new `nested-ladders-are-one-draw`, extended `existence-is-not-completeness`.
- 2026-07-31 — day-9, **last predicting day**. **4 prediction rows** from 77 legs (73 suppressed:
  19 stale-feed equity, 50 mid-band, 4 rel-spread), **0 new markets** — the one overnight relist
  quotes 8c wide. **Dry-ran the whole Friday runbook against live data and four of its steps were
  wrong**, two failing silently with exit 0: **`discover` is cached on file existence** (no
  `complete_through` guard — it re-read Thursday's boards and hid two overnight relists, `legs.csv`
  207 → 209); **`tape`/`wash` take SPACE-separated args** while `discover`/`live` take one
  comma-separated arg, so the documented comma rule silently disabled the tape gate and cost a row;
  `selftest` does not print `ok` on every line; and the Saturday BTC pass checked every key except
  BTC. All corrected in place, appendix traps 15–18. **08-02 sizing closed**
  (`results/sizing-2026-08-02-close-2026-07-31.md`): **the edge is smaller than the spread** —
  nominal margin +0.73pp against a 1.00c median half-spread, so selling at the **bid** fails by
  −0.27pp at nominal n with a zero fee; between-family ρ measured, `n_eff` ∈ **[118, 173]**, failing
  across the range; **Kelly at the 95% lower bound is negative**, which answers "at our size"
  without a bankroll. Wiki: new `clustering-coarser-is-not-safer`.
