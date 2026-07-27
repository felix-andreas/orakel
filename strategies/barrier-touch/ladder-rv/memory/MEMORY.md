# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **WE WERE WRONG ON `will-wti-dip-to-85-in-july-2026`, AND THE CAUSE IS A STALE FEED, NOT
  CALIBRATION** (`results/legsum-null-and-stale-feed-2026-07-27.md`). It touched; we said
  no-touch four days running. The 07-26 run read a spot **28.8h old** — the WTI/metals feed
  is shut Fri 21:00Z → Sun 22:00Z — while the book repriced **0.475 → 0.715 across exactly
  that closure**. Same spot, same σ, same sessions left as the 07-25 run: **the model could
  not move.** CLU6 then opened **−7.79%** (90.46 → 83.68) and printed 83.17 in the first
  minute, through the barrier. No feed we hold (WTIU6, USOILSPOT, XAUUSD) printed at all
  during the closure — there was no input we ignored. Solving the market's 0.715 for spot
  gives 87.3–88.0: **it was pricing a LOWER LEVEL, not a wider one**, so no vol fix reaches
  it. `will-wti-dip-to-90-...-from-july-25` opened below its barrier: YES before its window
  had a minute in it.
- **→ PROPOSE THE STALE-FEED GATE (CEO decision pending).** Never treat a disagreement with
  the market as edge when the resolving feed has been shut for the whole period over which
  the market moved. **Two of our four batches were emitted on a shut feed**: day 3 (Sat
  07-25, 51 rows, 4.5h stale) and day 4 (Sun 07-26, 13 rows, 28.8h stale) = **64 of 95
  outstanding rows.** Never run WTI/gold/silver `live` on a Saturday or Sunday again
  without flagging it.
- **LEG-SUM / NULL-MODEL RE-CHECK: DONE (day-5), and the answer is NO ARTIFACT at our
  anchors.** A Hit Price ladder is *nested*, not mutually exclusive, so the literal
  leg-sum≈1 gate is vacuous; the right analogue is **Σmid = expected YES count vs Σwinner**.
  Ratios: creation **1.38** (85% of legs quote mid∈[.45,.55] — an unpriced book),
  window-open 1.11, daily-12Z 1.28. Log-loss vs nulls: **at creation the null WINS**
  (mkt 0.6630 vs base-rate 0.6524; gold/WTI/SPY/NVDA all lose). **At window-open and
  daily-12Z the market beats uniform and base-rate in every asset** — and those are the
  only anchors we use (gate 1 = ws+3h, gate 2 = daily 12:00Z in-window). Verified in code.
- **BUT the leg-sum gate kills one claim: gold's window-open Brier margin.** Model-minus-
  market Brier, gold at window-open: **−0.0189 (t −1.96) ungated → −0.0078 (t −0.90) →
  −0.0001** under avg_mid ≤ 0.40 / 0.30. That is the number day-3 used to upgrade gold to
  tradeable. **Gold's DAILY-checkpoint edge survives** (−0.00541, t −3.55, n=1619), so gold
  stays tradeable on that evidence instead. WTI is gate-invariant (−0.00901, t −6.07).
  Pooled window-open edge **reverses** under the gate (−0.00505 → +0.00417) — stop quoting it.
- **Day-5 (2026-07-27): 8 proposed rows** (`results/proposed-rows-2026-07-27.csv`,
  run_id `2026-07-27/daily`) — WTI ↑95/↑100/↓75/↓80, gold ↑4300/↓3900, silver ↓54/↓52.
  All pass spread + mid + tape + epsilon. Silver ↑64/↑66 fail the relative spread gate.
- **FILL PICTURE SPLITS BY BOARD FAMILY** (`book-and-tape-audit-2026-07-26.md`): reachable
  fraction of the scored midpoint = WTI 99%, BTC 100%, silver 89%, gold 82%, **SPY/NVDA
  weekly 38%** — the 2/21 headline was about equity weeklies and sub-3c wings, not the
  variant. Gold: best Brier edge, thinnest book. **Sting from 07-27: the reachable legs are
  the ones we lost on** (tradeability 2/21 → 6/25 because mid-board WTI legs at 0.4–0.7 are
  the liquid ones).
- **TAPE GATE** (07-26): ≥1 taker trade on our side, within 5c, in 7 days. NVDA
  week-of-Jul-27 quoted six legs 1–5c wide with $470–780 listed and **zero trades ever** on
  five. Spread gate is **relative**, `≤ min(5c, ½·mid)`.
- **Roll calendar / August** (detail in STRATEGY.md + `august-roll-model-2026-07-26.md`):
  CLQ6→CLU6 at the **Fri 17 Jul** session (the July monthly spans a roll — changed no
  answer, but that was luck), CLU6→CLV6 **Tue 18 Aug**, CLV6→CLX6 **Fri 18 Sep**.
  **Archive WTIX6 the day it appears (~Aug 20)**; WTIQ6 is delisted forever. August is
  priceable (`ladderrv roll` validated) but **not predictable**: 14 of 20 legs quote 46–98c
  and every leg where the roll bites is unquoted; it resolves after the 08-02 review.
- **BOTH KNOWN CODE DEFECTS FIXED (day-5), in one model.** `touch_prob_jump` = first-passage
  with an explicit initial jump, covering (a) a window that opens later (the defect logged
  07-26) and (b) a feed that is shut right now. Plus `realized_vol_intraday` + `gap_sd`
  splitting total RV into smooth and gap parts, and a new `ladderrv gaps` subcommand.
  Watch the direction: on a weekend-free horizon the fix LOWERS q (WTI ↓75 0.177→0.100),
  i.e. it flatters a seller. It is right (RV14 is inflated by Sunday's gap, no weekend left
  in the window; realised 4-session move 5.9% vs model 6.3%, old model 7.7%) — but watch it.
- **GAP SD, MEASURED** (`ladderrv gaps`, Apr 1 – Jul 27; weekend / overnight): USOILSPOT
  **3.78% / 0.35%**, WTIU6 4.25% / 0.40%, XAUUSD 0.74% / 0.13%, XAGUSD 1.20% / 0.18%,
  SPY 0.74% / **0.59%**, NVDA 1.38% / **1.43%**, BTC 0/0. **A WTI weekend gap ≈ a whole
  session's variance** (intraday session rms 3.56%); its overnight gap is a tenth (CME crude
  pauses 1h). **For RTH-only equity the OVERNIGHT gap ≈ a whole session** — we have priced
  17.5h of daily risk at zero τ since day 1, a plausible cause of the model losing to the
  market on SPY/NVDA (+0.0089 / +0.0052) while beating it on WTI and gold.
- **Selftest caught a sign error:** the first jump used martingale-in-*price*
  `exp(jZ − j²/2)`, injecting a −j²/2 **log**-drift that makes every ↓ leg likelier and
  every ↑ leg less likely — a tilt flattering a seller of the up wing. Driftless in log
  price is what `touch_prob` already assumes.
- **A skipped freeze cost a reproducibility check.** Day-4 logged RV14 48.8%; a complete
  archive gives **51.7%** and no truncation of Friday reproduces 48.8%. The day-4 candle
  store was incomplete in a way we can no longer identify, because its inputs were never
  snapshotted — and the error ran in the flattering direction. **Never skip the freeze.**
- **`GET /markets?condition_ids=` returns `[]` for CLOSED markets** (same as `?slug=`);
  `&closed=true` fixes it — a 07-31 scoring hazard for anything keyed on condition_id.
- **↓80 INVERTED, and it is the cleanest lesson available.** Day-4 proposed it as a tier-A
  **sell at q 0.0738 vs a 0.405 mid** (−33c "edge"). Day-5, same model: **q 0.4906 vs a
  0.490 mid.** The signal did not decay, it inverted, and the entire inversion is spot
  90.46 → 83.82. Never executed. **A 33-point edge on this variant can be a 33-point spot
  move in disguise.**
- Inputs @ 2026-07-27 07:27Z (feed OPEN, 0.2h old): WTI CLU6 **83.82** (RV14 total 61.1%,
  **intraday 49.8%**, OVX 68.0), gold **4099.98** (20.9% / 20.3%, GVZ 24.3), silver
  **59.61** (42.0% / 40.6%, VXSLV 48.0). Every WTI leg now prices BELOW the market on q_rv
  and above on q_iv — the signature of an under-vol'd RV against OVX 68.
- **07-31 readiness: identity 51/51 clean** (95 outstanding ladder-rv rows over 51 markets;
  `&closed=true` used). **Epsilon screen clear** — closest are silver ↓54 1.44%, gold ↓3900
  1.55%, WTI ↑95 1.59%, WTI ↓80 1.75%, all outside 0.2%.
- **RESOLVED SO FAR:** `will-wti-dip-to-90-in-july-2026` YES (we said 0.8263 vs 0.82);
  `will-wti-dip-to-85-in-july-2026` **YES against us 4×**; `...-dip-to-90-...-from-july-25`
  **YES** (opened below its barrier). Headline now **−0.0172 over 25 rows**; ↓85 alone is
  −0.4510 and every other row still nets +0.0198. The CEO is fixing the repeat-row
  aggregation himself — **do not use the correlation point as a defence.**
- **Day-6 queue**: (1) CEO decision on the **stale-feed gate** — if adopted, implement it in
  `cmd_live` and stop emitting WTI/metals rows on Sat/Sun; (2) daily archive **every day**
  (WTIU6 **and** WTIV6 to Aug 20; **WTIX6 the day it appears ~Aug 20**); (3) re-read the
  week-of-Jul-27 books — they opened Sunday 22:00Z and were not re-read today; (4) 07-31
  21:00Z is the trial's real evidence, 95 rows; (5) consider promoting the RV/IV blend —
  OVX-anchored q was closer than RV on ↓85 (0.5156 vs 0.3928) and is above market on every
  WTI leg today; RV-primary is now the weakest link in the method.

## Medium-term

- **Where the edge is**: sell overpriced wings/extension legs; delayed sim beats instant
  for sells. Model beats market Brier on **WTI and GOLD only** — use the **daily-checkpoint,
  leg-sum-gated** numbers (WTI −0.00901 t −6.07; gold −0.00541 t −3.55), not the window-open
  ones, which do not survive the gate. spy +0.0089, nvda +0.0052, btc +0.0083 (model worse),
  silver ~0. Sells by asset (delayed, per trade): wti +14.4c, spy +19.6c(n=18), gold +7.1c,
  eth +8.3c, btc +6.4c, silver +3.0c, nvda −4.8c. **Cents/trade is the wrong unit**
  (`execution/DESIGN.md` §3): report return on locked capital, and `break-even-win-rate`'s
  q*/q/q⁻ table, before any promotion claim.
- **Resolution epsilon** (day-3, now `wiki/reference/venue-resolution-epsilon.md`):
  279/279 feed-touches resolved YES; 2/7 feed-*misses* within 0.10% resolved YES anyway →
  never sell a barrier within **0.2% of that leg's running window extreme**, from its TRUE
  window start. It screens *adjudication* risk, not price proximity — ↓85 sat 1.32% clear
  and touched anyway.
- **Four ways a quoted price can mislead**: dead book at 0.02/0.98 (`phantom-midpoints`);
  live-but-wide, mid ≠ bid (`midpoint-is-not-a-fill`); live-and-tight with no counterparty
  (`tape-gate`); and — new 07-27 — an honest quote against **our** stale feed.
- **Buys: unrescuable by drift/jump** (day-2, 255-leg sample). Failure is 100% crypto —
  the crypto market over-prices barrier touches ~4× (3–50c legs realized 0.038 vs mid
  0.148). Class-gated WTI+equity buys are positive but only 12/39 legs → **buys stay OFF**.
- **Fine print (gate-0 verified 756/760)**: crypto monthlies resolve on BINANCE USDT;
  "calendar month" legs resolve from listing; re-added strikes carry private window starts
  ("after market creation"); equity RTH-only; WTI/metals weeklies use the session clock,
  not the calendar week; WTI boards resolve on the **active month**, which rolls mid-board.
- **Data traps**: Pyth carries exactly **two** CL contracts and deletes expired ones
  (WTIQ6 gone, WTIX6 not yet listed) — archive daily. USOILSPOT is a CFD near the front,
  not the spliced active month (basis vs U6 p50 $0.21 p95 $0.98 max $2.07, no gap at the
  Jul 17 roll) — never settle a near-barrier WTI question off it. Pyth rate-limits ~1 req/s;
  candles stable on refetch (0/392). Polymarket headline volumeNum on WTI ≈ 20× real taker
  notional; `liquidityNum` is listed, not traded, size — capacity from book depth only.
  **data-api.polymarket.com 403s without a User-Agent header.**
- Vol anchors verified free: CBOE OVX/VIX/GVZ/VXSLV CSVs, Deribit DVOL. NVDA has none.
- Session calendars assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3, **Sep 7**);
  Columbus Day is NOT a closure. Revisit before the 2026-11-01 EST transition.

- **Checkpoint hygiene, settled 2026-07-27.** Anchor gate 1 at window-open and gate 2 at
  daily 12:00Z in-window — **never at board creation**, where 85% of legs quote ~0.50 and a
  flat base-rate beats the market. Report `Σmid / Σwinner` (expected vs realised YES count)
  beside every headline; that is this family's leg-sum. Gate board-snapshots on
  `avg_mid ≤ 0.40` before quoting any Brier margin.

## Long-term (wiki candidates)

- **A model whose only inputs are a closed feed cannot update; the market can.** When the
  resolving venue is shut and the prediction market is not, our disagreement with the
  market is evidence about *us*. Generalises to every feed-resolved market with a session
  calendar — commodities/equities over weekends and holidays, anything RTH-only. The gate
  is cheap: compare feed-print age to the market's move since that print. Strong wiki
  candidate; it is a different failure from all three "the quoted price lies" pages,
  because here the quote is honest and *our* number is the stale one.
- **Session-time vol models must carry a close-to-open gap term.** Amortising gap variance
  across session minutes gets the total roughly right and the *shape* wrong, and shape is
  what a first-passage question is about. Measured: a WTI weekend gap ≈ a whole session's
  variance; an RTH equity overnight ≈ a whole session's variance; a WTI overnight ≈ a tenth.
  Also: the same variance delivered as a jump gives a strictly smaller touch probability
  than delivered as diffusion (reflection counts round trips; a jump has no path) — in the
  jump-only limit exactly half.
- **"Active month" ladders spanning a CME roll contain a deterministic price gap** equal
  to the calendar spread, on a date known in advance. The right model is one diffusion
  with a **stepped barrier** plus absorption at the roll instant; the naive one-spot model
  errs 40–110% and always in the direction that flatters a seller of the down wing.
  Strong wiki candidate — generalises to any futures-referenced barrier market.
- ALREADY GRADUATED, don't re-derive: `tape-gate` (a tight spread is not liquidity),
  `venue-resolution-epsilon` (venues resolve generously at round numbers),
  `midpoint-is-not-a-fill`, `phantom-midpoints`, `favorite-longshot-bias` (2–5c bin 0/58
  here; 51 metals legs under 10c, 0 touched), `delayed-execution-test` (delay can IMPROVE
  sell fills — one-way lottery flow drifts wing prices up).
- Resolution-feed archaeology matters: resolution sources get deleted. Archive the
  resolving feed while the market is live, or lose gate-0 forever.
