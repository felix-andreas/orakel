# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **Day-8 (2026-07-30): 5 proposed rows**, `results/proposed-rows-2026-07-30.csv`, run_id
  `2026-07-30/daily`, 15 cols, header md5-identical to the ledger. WTI ↑95, WTI ↑90-from-jul-27,
  **WTI ↓80-from-jul-29 (a NEW relisted market)**, silver-weekly ↓56/↓55. 83 two-sided legs read,
  **78 suppressed**: 19 stale-feed (all equity, structural), 47 mid∉[3c,97c], 12 relative spread,
  0 tape, 0 epsilon, 0 de-dup. Inputs @01:13Z, **feed OPEN 0.0h**: CLU6 **83.86** (RV14 64.0%,
  intraday 52.2%, OVX 67.6), gold **4080.60**, silver **58.44**. Did not write predictions.csv.
- **Everything rallied and the boards emptied**: 12 rows → 5. The gold book *degenerated* at
  end-of-life (weekly ↑4200 quoting 0.040/0.660, monthly ↓3900 at 0.003/0.672) — phantom-midpoint
  spreads appear at a board's **death** as well as its birth. Gate did its job; nothing to retune.
- **Gate replay validated**: my reconstruction reproduces 07-29 exactly (44 mid / 10 spread /
  22 stale → 13, minus 1 tape = the recorded 12). Day-6's published suppression counts don't
  balance (13 vs the 14 claimed) — bookkeeping slip, not data.
- **THE FOURTH BUG: `closed_time` was 0 for all 74 closed legs, always.** Gamma's `closedTime` is
  `2026-07-29 16:10:11+00` — space separator, 2-digit offset, **not RFC3339**; `parse_iso` rejected
  it and `.unwrap_or(0)` swallowed it. Never used in a calculation, which is why it lived 8 days —
  and it is exactly the field Friday reaches for to ask "has UMA settled this". **Fixed + selftest
  asserts all 3 formats. Pricer untouched, selftest numbers identical.**
- **3 ROWS ON 2 MARKETS RESOLVED YES ON 07-29 AND ARE NOT IN `resolutions.csv`**:
  `will-wti-reach-85-in-july-2026-from-july-27` (1 row) and `will-xauusd-dip-to-4000-by-july-27-2026`
  (2 rows). Confirmed by Gamma (identity-asserted) **and** our own candles (WTIU6 max 85.56;
  XAUUSD min 3996.19). Both went **against** us, so omitting them flatters the headline, and they
  would have made the completeness gate read unmet for a bookkeeping reason. **The general lesson
  is new: every check we had asked "is the archive complete as of the last run", none asked "did
  something resolve while we weren't looking". Run `scripts/resolve_sweep.py` DAILY.**
- **`predictions_2026-07-26.csv` is permanently lost** — no archive, no container, not in git (day 4
  cut no `live-*`; the 07-29 rescue was already too late). Lost: day-4's ~80 suppressed legs' book
  snapshots. **Not** lost: its 13 emitted rows, fills, candles. No Friday/08-02 number depends on it.
- **New tooling, both in git**: `scripts/freeze.sh` (contents manifest in git, re-reads the tarball
  it just built and fails on a missing entry — the actual fix for the hand-written `tar` line; now
  also freezes `tape/` + `clob*/`) and `scripts/resolve_sweep.py` (unions both Gamma query forms,
  asserts `conditionId` identity per market, treats closed-without-final-`outcomePrices` as
  UNSETTLED). Both archives cut today and **read back out of R2** (not just verified).
- **`r2data verify` can FAIL on a transient HTTP 500** — candles-2026-07-25 FAILed on HEAD and
  `pull` fetched it intact. **Retry a FAIL before believing it**; re-freezing over a good archive
  is the destructive reflex.
- **The candle archive's holes are all session-calendar artifacts** — checked every day-file from
  07-20 (weekend zeros, Sunday 120-min opens, Friday 1261, the 74-min freeze-time stub). Local
  07-29 now complete (WTIU6 1379, SPY/NVDA 390 RTH). **Standing risk: no run after 07-31 21:00Z
  ⇒ the 07-31 resolution record is a 74-min stub and gate 0 is unanswerable.**
- **I WAS WRONG ON 07-29 ABOUT OVX FALLING BELOW RV.** It compared OVX to **RV14 total**; the pricer
  uses **intraday** RV. σ_iv sat ABOVE the σ actually used on every asset on **both** scorable days
  (WTI 0.5773 vs 0.5133 on 07-29; 0.6793 vs 0.5261 today). **The prereg's premise never softened.**
  OVX went 57.15 → 67.59 on 07-29 with VIX 18.21→20.66 and a VXSLV 54.10 high — a real vol event.
- **Friday's power, measured after today** (incl. my 5 rows), within `feed_open=1`: jump arm
  **40 rows / 19 markets**; old arm 48 / 36; `feed_open=0` 43 / 33 (own line, never pooled).
  **Clears 30 in rows, not in markets — INCONCLUSIVE, and markets is the honest unit** (13 of 19
  jump markets carry >1 row). Today bought 5 rows and **exactly 1 new market**: exhaustion
  confirmed, as predicted. **131 outstanding rows / 62 markets** go into Friday.
- **RV/IV prereg power**: 67 legs (07-29) + 64 (07-30), **union 68 distinct legs**, 63 in both.
  Clears n≥30 in legs *and* in markets — unlike the pricer split. **Its limitation is two days and
  one regime, not small n.** Tradeability veto baseline, recorded blind: A_rv/B_iv/C_blend =
  4/1/1 (07-29) and 2/1/1 (07-30) → **B and C both fail the veto on both days**, so the prereg's
  "everything passes" branch is effectively unreachable.
- **Day-9/Friday queue**: `results/friday-2026-07-31-runbook.md` is the whole procedure —
  freeze first, `resolve_sweep.py`, both freeze moments (Fri 21:00Z and Sat 04:00Z for BTC).

## Medium-term

- **08-02 IS A SIZING QUESTION AND THE ANSWER IS MEASURED** (`results/sizing-2026-08-02-prep.md`).
  Selling YES at `p` = **buying NO at `c = 1−p`**, so every legal trade is a favourite-side buy at
  50–97c. Pooled sell-signal band: `q*` 0.822, `q` 0.868, `q⁻` **0.829 clears at nominal n=356** and
  **0.808 FAILS at effective n=173**. Effective n from **ρ = 0.325** intraclass correlation of the
  loss indicator within a monotone family, 4.24 legs/family, design effect 2.05.
  Edge is at **both ends** (3–10c clears, 35–50c clears at +30.9% RoLC) and **absent in 10–35c**.
- **The tail is a CLIFF.** WTI down-ladder: 1.15 premium over 21 rows; at the realised −14.0% low
  (77.80) it loses 0.66 → net **+0.49**; at 75 it loses 6.96 → net **−5.81**. Next 5% down costs
  **548% of the family's entire premium**. 90% of premium sits on the two rungs nearest spot — the
  first two a continuing move takes. Deep wings = 4.7% of premium, 5.92 of loss exposure.
  **Size by family (asset,dir,window), never by leg.** Missing: between-family ρ (so 173 is an
  UPPER bound on effective n), fees/fills folded into `q*`, and a capital model.
- **Where the edge is**: sell overpriced wings/extension legs; delayed sim beats instant for sells.
  Model beats market Brier on **WTI and GOLD only**, on the **daily-checkpoint, leg-sum-gated**
  numbers (WTI −0.00901 t −6.07; gold −0.00541 t −3.55) — never the window-open ones.
  spy +0.0089, nvda +0.0052, btc +0.0083, silver ~0. **Cents/trade is the wrong unit**; report RoLC
  and the `q*`/`q`/`q⁻` table.
- **Risk is a fat one-sided tail on `dip-to` legs, not a calibration error.** The "structurally
  short downside touch" story is **REFUTED** (633 legs / 5,927 checkpoints): model **beats** market
  on touched legs (−0.01152, t −1.99), and WTI ↓ legs trending ≥5% into the barrier are its **best**
  bucket. `trend` is ex-post and **must never become a filter**. Do not re-derive.
- **Entry selection must use the FIRST qualifying checkpoint, not the last.** My own first cut of
  the break-even table used the last and produced −66% RoLC in the 20–35c band: for a leg that
  eventually touches the mid **rises toward 1**, so "last checkpoint still under 50c" selects the
  moment before the loss. `lifetime-volume-is-look-ahead` in a new costume.
- **A model whose only inputs are a closed feed cannot update; the market can.** Every row carries
  `feed_age_h`/`feed_open`. **Equity is structurally unpredictable on the daily cadence** — RTH is
  13:30–20:00Z, the trigger fires ~01:1xZ, so the feed is always shut. 19–22 legs suppressed every
  run. Still awaiting the CEO: second in-RTH run, or equity leaves the trial.
- **Checkpoint hygiene.** Daily 12:00Z in-window for gate 2, window-open for gate 1, **never board
  creation** (85% of legs quote ~0.50 there and a flat base rate beats the market). Nested family ⇒
  literal leg-sum is vacuous; report `Σmid` vs `Σwinner` (1.38 creation / 1.11 open / 1.28 daily).
  Gate board-snapshots on `avg_mid ≤ 0.40` before quoting a Brier margin.
- **Resolution epsilon**: 279/279 feed-touches resolved YES; 2/7 feed-*misses* within 0.10% resolved
  YES anyway → never sell a barrier within **0.2%** of that leg's running extreme, from its TRUE
  window start. Screens *adjudication* risk, not proximity — ↓85 sat 1.32% clear and touched.
- **Five ways a quoted price misleads**: dead book at 0.02/0.98 (`phantom-midpoints`); live-but-wide
  (`midpoint-is-not-a-fill`); tight with no counterparty (`tape-gate`); honest quote against **our**
  stale feed (`stale-feed-gate`); and an API answering a filtered query with `[]` rather than error.
- **Fill picture splits by board family**: reachable fraction of the scored midpoint = WTI 99%,
  BTC 100%, silver 89%, gold 82%, **SPY/NVDA weekly 38%**. **The reachable legs are the ones we lost
  on.** Book gate: rel spread `≤ min(5c, ½·mid)`, mid ∈ [3c,97c], ≥1 taker trade on our side within
  5c in 7d.
- **Buys: unrescuable** by drift/jump (255 legs). Failure is 100% crypto (~4× over-priced touches).
  Class-gated WTI+equity buys are positive but only 12/39 legs → **buys stay OFF**.
- **GAP SD (weekend / overnight)**: USOILSPOT **3.78% / 0.35%**, WTIU6 4.25% / 0.40%, XAU 0.74% /
  0.13%, XAG 1.20% / 0.18%, SPY 0.74% / **0.59%**, NVDA 1.38% / **1.43%**, BTC 0/0.
- **Roll calendar**: CLQ6→CLU6 at the **Fri 17 Jul** session, CLU6→CLV6 **Tue 18 Aug**, CLV6→CLX6
  **Fri 18 Sep**. **Archive WTIX6 the day it appears (~Aug 20)**; WTIQ6 delisted forever. Never
  reconstruct a July touch from WTIU6 before the 07-16 22:00Z roll — and note the July monthly's
  epsilon screen inherits that limitation (USOILSPOT is a CFD proxy, not the spliced active month).
  August is priceable (`ladderrv roll` validated), not predictable, and resolves after the review.
- **Fine print (gate-0 756/760)**: crypto monthlies resolve on BINANCE USDT; "calendar month" legs
  resolve from listing; re-added strikes carry private window starts ("after market creation") —
  today's ↓80-from-jul-29 starts 2026-07-29 16:27:11Z; equity RTH-only; WTI/metals weeklies use the
  session clock, not the calendar week; WTI resolves on the **active month**, which rolls mid-board.
- **Data traps**: Pyth carries exactly **two** CL contracts and deletes expired ones. USOILSPOT is a
  CFD near the front, never settle a near-barrier WTI question off it. Pyth ~1 req/s.
  Polymarket headline volumeNum on WTI ≈ 20× real taker notional. data-api.polymarket.com 403s
  without a User-Agent. **`ladderrv live` takes ONE comma-separated arg** — space-separated silently
  prices only the first board. `cmd_live` **overwrites** `predictions_<date>.csv` per call.
- Vol anchors verified free: CBOE OVX/VIX/GVZ/VXSLV CSVs, Deribit DVOL. NVDA has none.
- Session calendars assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3, **Sep 7**); Columbus Day
  is NOT a closure. Revisit before the 2026-11-01 EST transition.

## Long-term (wiki candidates)

- ALREADY GRADUATED, don't re-derive: `nested-ladders-are-one-draw` (**new today**),
  `existence-is-not-completeness` (**extended today**: a *field* that parses and carries no
  information, and a verify FAIL that is transient), `stale-feed-gate`, `tape-gate`,
  `venue-resolution-epsilon`, `midpoint-is-not-a-fill`, `phantom-midpoints`,
  `favorite-longshot-bias`, `delayed-execution-test`, `checkpoint-artifact`,
  `lifetime-volume-is-look-ahead`, `break-even-win-rate`, `depth-lives-where-the-edge-is-not`.
- **A mean edge and a fat one-sided tail are different promotion questions.** Brier and
  cents/trade both average; neither sees that the 8 worst legs of 633 share a direction.
- **Session-time vol models must carry a close-to-open gap term.** Amortising gap variance across
  session minutes gets the total right and the *shape* wrong, and shape is what a first-passage
  question is about. The same variance as a jump gives a strictly smaller touch probability than as
  diffusion — in the jump-only limit exactly half.
- **"Active month" ladders spanning a CME roll contain a deterministic price gap** on a known date.
  One diffusion with a **stepped barrier** plus absorption at the roll; the naive one-spot model
  errs 40–110%, always flattering a seller of the down wing.
- **A gate that cannot fail is worse than no gate**, because it gets written down as passed.
- Resolution sources get deleted. Archive the resolving feed while the market is live.
