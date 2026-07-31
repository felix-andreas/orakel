# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **Day-9 (2026-07-31): 4 proposed rows**, `results/proposed-rows-2026-07-31.csv`, run_id
  `2026-07-31/daily`, 15 cols, header md5 `c9ee0b9f…` byte-identical to the ledger,
  `pricer_version` unchanged. WTI ↑90-from-jul-27, WTI ↓80-from-jul-29, gold-weekly ↑4150,
  silver-weekly ↓56. 77 legs read, **73 suppressed**: 19 stale-feed (equity), 50 mid∉[3c,97c],
  4 rel-spread, 0 tape, 0 epsilon, 0 de-dup. Inputs @01:21Z, **feed OPEN 0.1h**: CLU6 **82.83**
  (RV14 65.3%, intraday 53.8%), gold 4092.73, silver 58.84. Did not write predictions.csv.
- **Today bought 0 new markets.** The one new relist (`will-wti-reach-85-in-july-2026-from-july-30`,
  listed 07-30 16:27Z) quotes **0.32/0.40 — 8c wide**, suppressed on rel-spread. So today's rows
  land in the **0–1d bucket** (flat, 3/20 fillable) on markets already predicted: **rows, not
  power**. Emitted anyway because the policy says so — changing emission on the last day on the
  basis of what it does to the trial's own numbers is what pre-registration exists to prevent.
- **RAN THE WHOLE FRIDAY RUNBOOK TODAY. Four things in it were wrong**, all fixed in place and
  marked `[DRY-RUN FIX]`; two steps can't be exercised before the close and are marked
  `[UNTESTABLE TODAY]` with what to check instead.
- **THE FIFTH SILENT-DATA BUG: `discover` is cached on `p.exists()`** — `fetch_all` has no
  `complete_through` guard, unlike candles/clob/tape which were all fixed. As written it printed
  **`fetched 0, cached 12`** and re-read Thursday's JSON; on Friday every July board would still
  read `closed:false`. **Not hypothetical**: clearing the cache took `legs.csv` **207 → 209** and
  surfaced two overnight relists plus a BTC leg flipping `closed` with `outcomePrices ["1","0"]`.
  Fix: `mv data/events` aside first, then require `fetched 12`.
- **NEW TRAP, the exact inverse of the old one: `tape`/`wash` take SPACE-separated args**
  (`args[3..]`); `discover`/`live` take **one comma-separated arg** (`args[3].split(',')`). A
  comma-joined string matches no board, runs the loop zero times, **exits 0 with no output**.
  It cost a row today: the stale tape read **0** taker trades within 5c on xagusd ↓56 (a
  tape-gate suppression); refetched properly it had **4**, and it is in today's emission.
- **Gate reconstruction re-validated before use**: reproduces 07-30 (19 stale/47 mid/12 spread
  → 5) and 07-29 (22/44/10 → 13, −1 tape = 12) **exactly**. Emitted rows = survivors of the
  book+stale gates, **not** only sell signals (today 4 survivors, 2 sell signals).
- **VERIFIED WORKING** (don't re-test): the candle refetch rule — 07-30's **73-minute stubs
  repaired to 1381/1382/392**, the single most load-bearing mechanism in the runbook;
  `resolve_sweep.py` (60 markets/128 rows, **0 needing a human**); `vol`; `clob 60`; `analyze`;
  `freeze.sh` (both archives cut, pushed **and read back out of R2** — 77 legs, cols 15–18 right);
  `scoring` (`ci_lo`/`ci_hi` present in `scores.csv`, **not** in the printed table).
- **Friday candle counts, measured**: a Friday is **1261** WTI/metals (session ends 21:00Z), a
  mid-week day **1381/1382** (runs to 23:59Z), SPY/NVDA **392**, BTC **1440**. So the runbook's
  "≈1260" is right *for Friday only*. Saturday: **BTCUSDT is the only key that matters**.
- **The 3 settled rows are ALREADY APPENDED** (resolutions.csv lines 23–24) — skip that step.
  And they went **against us on P&L while improving the per-market statistic** (+0.0319, +0.0178),
  moving the variant mean −0.0127@21 markets → **−0.0094@23, CI [−0.0280, +0.0092]**.
  **"Lost money" and "beat the market" are different signs on the same row.**
- **`git pull --rebase` fails here** ("Cannot rebase onto multiple branches"). Use
  `git pull --rebase origin main`.
- **Day-10/Friday queue**: `results/friday-2026-07-31-runbook.md`, now corrected and with a
  pre-declared list (§5e) of the seven things Friday cannot settle.

## Medium-term

- **08-02 IS A SIZING QUESTION AND ALL THREE GAPS ARE NOW CLOSED**
  (`results/sizing-2026-08-02-close-2026-07-31.md`; the 07-30 prep doc is superseded on §6).
  **The decisive number needs no correlation argument: the nominal margin is +0.73pp and the
  median half-spread on gate-passing legs is 1.00c.** Selling at the **bid** rather than the mid
  gives `q* = 0.8316` vs `q⁻ = 0.8289` → **fails by −0.27pp at nominal n, at a zero fee.**
  Break-even half-spread 0.73c; median gate-passing book 2.0c wide. **The edge is smaller than
  the spread.**
- **Between-family ρ measured: `n_eff` ∈ [118, 173]**, failing across the whole range
  (−1.21pp family / −2.65pp asset×direction). Reproduction of 07-30 is exact
  (356 / 0.8216 / 0.8680 / 0.8289 / ρ 0.326 / 84 families / deff 2.05) — **the fundable band is
  [0.03, 0.50), half-open**; read closed it gives 365 and every number moves.
- **ρ at board level (0.073) is a quarter of family level (0.326)** — a board mixes up- and
  down-legs, driven by opposite tails of one path, so pooling averages the correlation against
  its own opposite. **Direction is the clustering unit, not the board.** And **ρ = 0.000 at the
  asset level is 7 clusters failing to identify an ICC, not independence** — it is the only
  level that "clears" and must never be quoted.
- **Kelly at the 95% lower bound is NEGATIVE** (−6.8% at n_eff 173, −14.9% at 118; +26.0% on the
  point estimate is the trap). This answers "at our size" **without a bankroll**, so the missing
  bankroll is no longer why the question is open. Size by family: k̄ 4.24, so a per-leg cap of x
  is a per-draw cap of 4.24x.
- **The tail is a CLIFF.** WTI down-ladder: 1.15 premium over 21 rows; at the realised −14.0% low
  (77.80) it loses 0.66 → net **+0.49**; at 75 it loses 6.96 → net **−5.81**. Next 5% down costs
  **548% of the family's entire premium**. 90% of premium sits on the two rungs nearest spot.
  Deep wings = 4.7% of premium, 5.92 of loss exposure.
- **Where the edge is**: sell overpriced wings/extension legs; delayed sim beats instant for sells.
  Model beats market Brier on **WTI and GOLD only**, on the **daily-checkpoint, leg-sum-gated**
  numbers (WTI −0.00901 t −6.07; gold −0.00541 t −3.55) — never the window-open ones.
  **Cents/trade is the wrong unit**; report RoLC and the `q*`/`q`/`q⁻` table. Only the **35–50c**
  band clears with room (+5.2pp, +30.9% RoLC, n=65) — a **new pre-registration for August**, never
  a filter fitted to a trial in progress.
- **Risk is a fat one-sided tail on `dip-to` legs, not a calibration error.** The "structurally
  short downside touch" story is **REFUTED** (633 legs / 5,927 checkpoints): model **beats**
  market on touched legs (−0.01152, t −1.99). `trend` is ex-post and **must never become a
  filter**. Do not re-derive.
- **Entry selection must use the FIRST qualifying checkpoint, not the last** — the last gives
  −66% RoLC in the 20–35c band, because a touching leg's mid **rises toward 1** first.
  `lifetime-volume-is-look-ahead` in a new costume.
- **A model whose only inputs are a closed feed cannot update; the market can.** **Equity is
  structurally unpredictable on the daily cadence** — RTH 13:30–20:00Z, trigger ~01:2xZ, so the
  feed is always shut. 19–22 legs suppressed **every single run**. Still awaiting the CEO: a
  second in-RTH run, or equity leaves the trial.
- **Checkpoint hygiene.** Daily 12:00Z in-window for gate 2, window-open for gate 1, **never board
  creation** (85% of legs quote ~0.50 there). Nested family ⇒ literal leg-sum is vacuous; report
  `Σmid` vs `Σwinner` (1.38 creation / 1.11 open / 1.28 daily). Gate board-snapshots on
  `avg_mid ≤ 0.40` before quoting a Brier margin.
- **Resolution epsilon**: 279/279 feed-touches resolved YES; 2/7 feed-*misses* within 0.10%
  resolved YES anyway → never sell a barrier within **0.2%** of that leg's running extreme, from
  its TRUE window start.
- **Five ways a quoted price misleads**: dead book at 0.02/0.98 (`phantom-midpoints`); live-but-wide
  (`midpoint-is-not-a-fill`); tight with no counterparty (`tape-gate`); honest quote against **our**
  stale feed (`stale-feed-gate`); and an API answering a filtered query with `[]` rather than error.
  Books also **degenerate at a board's death**, not only its birth.
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
  reconstruct a July touch from WTIU6 before the 07-16 22:00Z roll. August is priceable
  (`ladderrv roll` validated), not predictable, and resolves after the review.
- **Fine print (gate-0 756/760)**: crypto monthlies resolve on BINANCE USDT; "calendar month" legs
  resolve from listing; **re-added strikes carry private window starts** and Polymarket **relists a
  barrier as soon as it is touched** (↑85-from-jul-27, ↓80-from-jul-29, ↑85-from-jul-30,
  ↑65k-from-jul-28/30) — always read `startDate`, never assume the board's; equity RTH-only;
  WTI/metals weeklies use the session clock; WTI resolves on the **active month**, which rolls
  mid-board.
- **Data traps**: Pyth carries exactly **two** CL contracts and deletes expired ones. USOILSPOT is a
  CFD near the front, never settle a near-barrier WTI question off it. Pyth ~1 req/s.
  Polymarket headline volumeNum on WTI ≈ 20× real taker notional. data-api.polymarket.com 403s
  without a User-Agent. **`cmd_live` overwrites `predictions_<date>.csv` per call** — all boards in
  ONE call (I walked into this today and had to re-run).
- Vol anchors verified free: CBOE OVX/VIX/GVZ/VXSLV CSVs, Deribit DVOL. NVDA has none.
- Session calendars assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3, **Sep 7**); Columbus Day
  is NOT a closure. Revisit before the 2026-11-01 EST transition.

## Long-term (wiki candidates)

- ALREADY GRADUATED, don't re-derive: `clustering-coarser-is-not-safer` (**new today**),
  `nested-ladders-are-one-draw`, `existence-is-not-completeness`, `stale-feed-gate`, `tape-gate`,
  `venue-resolution-epsilon`, `midpoint-is-not-a-fill`, `phantom-midpoints`,
  `favorite-longshot-bias`, `delayed-execution-test`, `checkpoint-artifact`,
  `lifetime-volume-is-look-ahead`, `break-even-win-rate`, `depth-lives-where-the-edge-is-not`.
- **A margin thinner than the half-spread is not a small edge, it is no edge** — and it is
  checkable in one line, before any correlation or sample-size argument. Candidate wiki page if a
  second variant hits it.
- **A runbook that has never been run is a document, not a procedure.** Four of its steps were
  wrong today, and two of the four failed *silently with exit 0*. Dry-run the procedure on the day
  before, against live data, not the day of.
- **A mean edge and a fat one-sided tail are different promotion questions.** Brier and
  cents/trade both average; neither sees that the 8 worst legs of 633 share a direction.
- **Session-time vol models must carry a close-to-open gap term.** The same variance as a jump
  gives a strictly smaller touch probability than as diffusion — in the jump-only limit exactly half.
- **"Active month" ladders spanning a CME roll contain a deterministic price gap** on a known date.
  The naive one-spot model errs 40–110%, always flattering a seller of the down wing.
- **A gate that cannot fail is worse than no gate**, because it gets written down as passed.
- Resolution sources get deleted. Archive the resolving feed while the market is live.
