# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **STILL OPEN — MANDATORY (CEO, 2026-07-25): leg-sum / null-model re-check.** Re-run the
  headline numbers with (a) leg-sum reported per checkpoint, (b) checkpoints gated to
  leg-sum ≤ ~1.05, (c) a naive null model through the same pipeline that must LOSE. See
  `wiki/reference/checkpoint-artifact.md`. **Not done day-4** — the day went on the August
  roll blocker, the book/tape audit and the 07-31 prep. Do it day-5; it is the last thing
  standing between us and an honest trial review on 08-02.

- **Day-4 (2026-07-26) DONE.** Roll-aware pricer built + validated; book/tape audit;
  07-31 identity check; **13 proposed rows** (`results/proposed-rows-2026-07-26.csv`,
  run_id `2026-07-26/daily`, model `claude-opus-5`) — WTI 7, gold 2, silver 4, all July
  monthlies. **Zero** August rows, **zero** week-of-Jul-27 rows.
- **THE FILL PICTURE SPLITS BY BOARD FAMILY** (`results/book-and-tape-audit-2026-07-26.md`).
  Forward from each row's own timestamp, over all 70 markets we have ever predicted:
  reachable fraction of the scored midpoint = **WTI 99%, BTC 100%, silver 89%, gold 82%,
  SPY/NVDA weekly 38%**. 24h bid-side taker flow on live WTI July legs: ↑95 $27.7k,
  ↑100 $11.4k, ↓85 $7.7k, ↓80 $3.0k. **The 2/21 headline was about equity weeklies and
  sub-3c wings, not about the variant.** Caveat: **gold has the best Brier edge and the
  thinnest book** — 0/11 markets ever showed a bid at our mid, tob$ $1–19.
- **NEW: TAPE GATE.** NVDA week-of-Jul-27 quotes six legs 1–5c wide with $470–780 listed
  liquidity — and **zero trades ever** on five of them ($28 on the sixth). A tight spread
  is not liquidity. Require ≥1 taker trade on our side, within 5c of the quote, in 7 days.
  Also: spread gate is now **relative**, `≤ min(5c, ½·mid)` — a flat 5c bar passes a
  0.003/0.019 book whose mid is 3.8× its bid.
- **ROLL CALENDAR CORRECTED — the JULY monthly spans a roll too.** CLQ6→CLU6 at the
  session for **Fri 17 Jul** (25 Jul is a Saturday → CLQ6 LTD Tue 21 Jul). Our gate-0 used
  WTIU6 for the CLQ6 half of July and of the week-of-Jul-13 weekly. **No answer changed**
  (CLQ6 ran ~67–81 vs barriers ≤65/≥95) but it was luck. WTIQ6 is delisted forever.
  CLU6→CLV6 at **Tue 18 Aug** session (2026-08-17 22:00Z) confirmed twice over; CLV6→CLX6
  at the **Fri 18 Sep** session. **Archive WTIX6 the day it appears (~Aug 20).**
- **August: priceable, not predictable.** `ladderrv roll` + `selftest` work; the naive
  model under-prices every August ↓ leg by 40–110% relative (↓80 0.365→0.508, ↓75
  0.167→0.285) and over-prices ↑90 as a certainty (1.000 vs 0.843). But 14 of 20 legs
  quote 46–98c spreads, the 6 gate-passing legs are wings where the roll is worth nothing,
  and **every leg where the roll bites is unquoted**. August also resolves 08-31, after
  the 08-02 review. Gold/silver August: all 28 legs fail the gate.
- **Two code defects found by reading (day-4).** (1) `SessionCal::build` stopped at
  2026-08-20 → τ truncated to 14 of the August board's 21 sessions, σ√τ 18% low —
  **FIXED** (calendar to 2026-10-31, Labor Day added, Columbus Day deliberately excluded).
  (2) `cmd_live` starts the diffusion at *today's* spot for a window that opens later
  (σ√τ_pre = 5.9% of spot for the August board) → under-prices every leg on a not-yet-open
  board. **NOT fixed**; `roll` handles it, `live` does not. Day-5 job.
- **`GET /markets?condition_ids=` returns `[]` for CLOSED markets**, same as `?slug=`;
  `&closed=true` fixes it. Our own identity is clean: **70/70 slugs → same conditionId,
  70/70 token_ids → `clobTokenIds[0]`, no drift.** Flagged to the CEO as a 07-31 scoring
  hazard for anything that looks markets up by condition_id.
- **Day-4 signals:** two tier-A WTI sells with real depth — ↓75 (mid 0.100, q_rv 0.006,
  bid $334) and ↓80 (mid 0.405, q_rv 0.074, bid $248, 54% on locked capital over 6 days).
  No gold or silver signal for the third day running (all fundable metals legs within ~2c
  of the model). Biggest disagreement is a BUY and therefore untakeable: `will-wti-reach-95`
  quotes 0.216 where the model says 0.476 (RV) / 0.609 (OVX) — the market implies ~27% vol
  against RV14 48.8% and OVX 68. **It resolves 07-31: the most informative row we hold.**
- Inputs @ 2026-07-24 21:00Z: WTI CLU6 90.46 (RV14 48.8%, OVX 68.0), gold 4053.31 (RV14
  20.4%, GVZ 24.3), silver 58.20 (RV14 41.3%, VXSLV 48.1). U6−V6 +$4.58.
- **`will-wti-dip-to-90-in-july-2026` RESOLVED YES** (we said 0.8263 vs 0.82 mid). A
  **re-added** ↓90 leg exists — `...-from-july-25`, quoting 0.933/0.961, window opens at
  Monday's session open. Do not confuse them when scoring.
- **Day-5 queue**: (1) the leg-sum/null-model re-check — overdue; (2) daily archive (WTIU6
  **and** WTIV6, every day to Aug 20) — **skipped day-4**, both feeds still listed so
  refetchable, but do not skip twice; (3) re-read the week-of-Jul-27 books after the
  Monday 22:00Z open; (4) fix `cmd_live`'s missing pre-window diffusion; (5) 07-31 is the
  trial's real evidence — 51 rows from day-3 plus 13 from day-4 resolve at 21:00Z.

## Medium-term

- **Where the edge is**: sell overpriced wings/extension legs; delayed sim beats instant
  for sells. Model beats market Brier on **WTI (−0.0133) and GOLD (−0.0189)** only;
  spy +0.0291, nvda +0.0197, btc +0.0044, eth +0.0083, silver −0.0060 (noise) are all
  predict-only. Sells by asset (delayed, per trade): wti +14.4c, spy +19.6c(n=18),
  gold +7.1c, eth +8.3c, btc +6.4c, silver +3.0c, nvda −4.8c. **But cents/trade is the
  wrong unit** (`execution/DESIGN.md` §3): report return on locked capital too — selling a
  40c leg locks 60c, selling a 9c leg locks 91c for the same nominal edge.
- **Resolution epsilon (day-3)**: venue resolution error is **one-directional against
  sellers**. 279/279 feed-touches resolved YES (0 reversals, incl. 32 within 0.5%); 2/7
  feed-*misses* within 0.10% resolved YES anyway (SPY ↑750, Pyth peaked 749.99002;
  XAGUSD ↑69, peaked 68.942) — both ↑ legs at round numbers. → never sell a barrier within
  **0.2% of that leg's running window extreme**, from its TRUE window start. Day-4 recheck
  of the July board: nothing inside 0.2%; closest are ↓85 (1.32%), ↑95 (1.59%), ↓80 (1.75%).
- **Three distinct ways a quoted price lies**, now all observed here: dead book quoting
  0.02/0.98 (`phantom-midpoints`); live-but-wide book whose midpoint is not the bid
  (`midpoint-is-not-a-fill`); **live-and-tight book with no counterparty at all** (new,
  day-4, NVDA weekly). Only the tape can see the third.
- **Buys: unrescuable by drift/jump** (day-2, 255-leg sample). Failure is 100% crypto —
  the crypto market over-prices barrier touches ~4× (3–50c legs realized 0.038 vs mid
  0.148). Class-gated WTI+equity buys are positive but only 12/39 legs → **buys stay OFF**.
- **Fine print (gate-0 verified 756/760)**: crypto monthlies resolve on BINANCE USDT;
  "calendar month" legs resolve from listing; re-added strikes carry private window starts
  ("after market creation"); equity RTH-only; WTI/metals weeklies use the session clock,
  not the calendar week; WTI boards resolve on the **active month**, which rolls mid-board.
- **Data traps**: Pyth carries exactly **two** CL contracts and deletes expired ones
  (WTIQ6 already gone, WTIX6 not yet listed) — archive daily. USOILSPOT is a CFD near the
  front, not the spliced active month: basis vs U6 p50 $0.21 p95 $0.98 max $2.07, and it
  shows **no gap at the Jul 17 roll** — never settle a near-barrier WTI question off it.
  Pyth benchmarks rate-limits (~1 req/s); its candles are stable on refetch (0/392 changed).
  Polymarket headline volumeNum on WTI ≈ 20× real taker notional — capacity from book depth
  only; Gamma's `liquidityNum` is listed size, not traded size, and can be $780 on a leg
  that has never traded.
- Vol anchors verified free: CBOE OVX/VIX/GVZ/VXSLV CSVs, Deribit DVOL. NVDA has none.
- Session calendars assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3, **Sep 7**);
  Columbus Day is NOT a closure. Revisit before the 2026-11-01 EST transition.

## Long-term (wiki candidates)

- **"Active month" ladders spanning a CME roll contain a deterministic price gap** equal
  to the calendar spread, on a date known in advance. The right model is one diffusion
  with a **stepped barrier** plus absorption at the roll instant; the naive one-spot model
  errs 40–110% and always in the direction that flatters a seller of the down wing.
  Strong wiki candidate — generalises to any futures-referenced barrier market.
- **A tight spread is not liquidity.** A whole ladder quoted 1–5c wide with zero lifetime
  trades. Spread gates, depth gates and *tape* gates are three different tests.
- Favorite-longshot INSIDE one-touch ladders, confirmed on metals independently: 51 metals
  legs under 10c, **0 touched**; 80–95c bin hit 95.5%; 2–5c bin 0/58 across all assets.
- Delayed execution can IMPROVE sell fills: one-way lottery flow drifts wing prices up.
- **Venues resolve generously at round numbers**: a barrier the feed misses by <0.1% still
  resolves YES ~29% of the time, while a feed-touch is never reversed.
- Resolution-feed archaeology matters: resolution sources get deleted. Archive the
  resolving feed while the market is live, or lose gate-0 forever.
