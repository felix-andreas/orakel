# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **Day-6 (2026-07-28): 14 proposed rows**, `results/proposed-rows-2026-07-28.csv`, run_id
  `2026-07-28/daily`, 15-column ledger schema. WTI ↑100/↑95/↓75/↓80/↑90-from-jul-27,
  gold ↑4300/↓3900, silver ↓54, gold-weekly ↑4250/↑4200/↓4000/↓3950, silver-weekly ↑63/↓55.
  93 two-sided legs read, **79 suppressed**: 22 stale-feed (all equity), 34 mid<3c, 18 relative
  spread, 3 weekly/monthly de-dup, 2 tape gate. Inputs @01:21Z, **feed OPEN 0.1h**: WTI CLU6
  **81.84** (RV14 60.8%, intraday 49.5%, OVX 60.6), gold **4056.89** (20.6/19.9, GVZ 24.1),
  silver **57.40** (41.6/40.2, VXSLV 47.6).
- **WTI has fallen 90.46 → 83.68 → 81.84 in three sessions.** ↓80 now quotes 0.745 against
  q 0.711. The whole outstanding book is priced against one selloff — say so at the review.
- **THE STALE-FEED GATE IS STRUCTURAL FOR EQUITY, NOT OCCASIONAL.** SPY/NVDA resolve on Pyth
  RTH 13:30–20:00Z; the daily trigger fires ~01:07Z, when that feed has been shut ~5.4h.
  **The daily run can therefore NEVER legally predict an equity board.** All 22 equity legs
  suppressed today. Escalated: either a second run inside 13:30–20:00Z, or equity leaves the
  trial. (Model was −12c to −22c vs the book on the SPY down wing — the ↓85 shape exactly.)
- **BACKFILL, MEASURED NOT GUESSED** (`results/ledger-backfill-2026-07-28.csv`): feed_age_h /
  feed_open for all 132 pre-existing ledger rows, computed from the frozen archive + session
  calendars. Confirms 07-26 = 28.8h; **corrects 07-25 from "4.5h" to 4.9h** (batch stamped
  01:52Z, last WTI print Fri 21:00Z). New: **days 1–2 emitted 20 equity rows on a shut RTH
  feed too** (14.1h and 5.7h). So **68 of 132 rows — 52% — were priced off a shut feed**, not
  64/95; and **every equity row we have ever emitted was stale.** They resolved NO and scored
  well: lucky, not clean.
- **`cmd_candles` KEPT A PARTIAL YESTERDAY FOREVER — FIXED.** Only `today` was force-refetched,
  so yesterday's file, written mid-day by yesterday's run, was "cached" from then on. Today it
  held WTIU6 07-27 at 21.9KB against a true **69.7KB**, and **52-byte `no_data`** files for
  SPY/NVDA — Monday's entire RTH session missing from the inputs. Same failure that made day-4
  log RV14 48.8% vs a true 51.7%. Now: refetch unless the file was written after that day ended.
- **`live` TAKES ONE COMMA-SEPARATED ARG.** Space-separated silently prices only the FIRST
  board and still writes a plausible-looking prediction file. It also **overwrites**
  `data/out/predictions_<date>.csv` per invocation — run all boards in one call.
- **GAMMA `closed` IS A FILTER WITH DEFAULT `false`, NOT AN OVERRIDE.** `?condition_ids=<cid>`
  returns an OPEN market and `[]` for a closed one; `&closed=true` is the exact reverse.
  **No single query finds a market in both states — try both.** Friday's set will be MIXED
  (boards close 21:00Z, UMA lags), so a one-form scorer drops half, silently, since `[]` is a
  valid 200. Yesterday's memory had only half this rule. Also: **`?condition_id=` (singular)
  is not an error** — Gamma ignores it and returns an arbitrary market (it handed back "New
  Rihanna Album before GTA VI?" for a WTI condition id).
- **07-31 READINESS** (`results/friday-2026-07-31-readiness.md`): **120 outstanding rows over
  58 markets, identity 58/58 clean.** 104 rows resolve Fri 21:00Z (WTI/gold/silver monthlies
  + week-of-Jul-27 weeklies), 16 BTC rows Sat 04:00Z. Gamma's `endDate` for the monthlies says
  Aug 1 03:59Z — the **window** still ends 07-31 21:00Z; never score off `endDate`.
- **PRICER SPLIT IS CONFOUNDED WITH FEED STATE.** Outstanding: `touch-prob` 50 open / **45
  shut**, `touch-prob-jump` **25 open / 0 shut**. Every stale row is also an old-pricer row.
  → **Compare pricers only within `feed_open=1`: 50 vs 25.** The jump arm is under the n≥30
  floor; Wed and Thu runs are the only way to fix that. **Skipping either kills the split.**
- **RV/IV IS PRE-REGISTERED, NOT SWITCHED** (`results/prereg-rv-iv-blend-2026-07-28.md`).
  IV sits **above** RV on **62 of 62** WTI/gold/silver legs (OVX 60.6 vs 49.7; GVZ 24.1 vs
  20.0; VXSLV 47.6 vs 40.2). Higher σ raises every q, and we are sell-only, so IV *destroys*
  sell signals: on today's 27 fundable legs **RV 4, blend 3, IV 1**. Decision rule fixed
  before the outcome; blend weight w=0.5 never tuned; tradeability veto can fail it even if
  Brier passes. Fixed one asymmetry first: `q_iv` used raw IV where `q_rv` used bumped σ.
- **DE-DUP WEEKLY AGAINST MONTHLY** (day-5 rule, applied again): the week-of-Jul-27 boards end
  at the same instant as the monthlies, so a weekly leg whose barrier is live-and-untouched on
  the monthly measures the same event twice. WTI weekly is now fully priced (12 legs) and
  still yields **zero** rows — its only non-duplicate barrier is H125 at a 0.5c mid.
- **TAPE GATE EARNED ITS KEEP AGAIN:** silver-weekly ↑62 and ↑61 pass spread *and* mid *and*
  the board trades actively — but have **zero taker trades within 5c in 7 days**. An actively
  traded ladder can still contain legs nobody has ever transacted at.
- **RESOLVED SO FAR:** `will-wti-dip-to-90-in-july-2026` YES (0.8263 vs 0.82);
  `will-wti-dip-to-85-in-july-2026` **YES against us 4×**; `...-dip-to-90-...-from-july-25`
  **YES** (opened below its barrier). Headline **−0.0172 over 25 rows**; ↓85 alone is −0.4510
  and every other row nets +0.0198. The CEO owns the repeat-row aggregation — **do not use the
  correlation point as a defence.**
- **↓80 INVERTED, and it is still the cleanest lesson here.** Day-4 called it a tier-A sell at
  q 0.0738 vs a 0.405 mid (−33c "edge"); day-5, same model, q 0.4906 vs 0.490. The entire
  inversion is spot 90.46 → 83.82. **A 33-point edge on this variant can be a 33-point spot
  move in disguise.** Never executed.
- **Day-7 queue**: (1) run Wed AND Thu — the pricer split needs them; (2) daily archive every
  day (WTIU6 **and** WTIV6 to Aug 20; **WTIX6 the day it appears ~Aug 20**); (3) CEO decision
  on an in-RTH equity run; (4) 07-31 21:00Z is the trial's real evidence, 120 rows;
  (5) apply the ledger backfill before Friday or 95 rows score as `unversioned`.

## Medium-term

- **Where the edge is**: sell overpriced wings/extension legs; delayed sim beats instant for
  sells. Model beats market Brier on **WTI and GOLD only** — use the **daily-checkpoint,
  leg-sum-gated** numbers (WTI −0.00901 t −6.07; gold −0.00541 t −3.55), never the window-open
  ones, which do not survive the gate. spy +0.0089, nvda +0.0052, btc +0.0083 (model worse),
  silver ~0. Sells by asset (delayed, per trade): wti +14.4c, spy +19.6c(n=18), gold +7.1c,
  eth +8.3c, btc +6.4c, silver +3.0c, nvda −4.8c. **Cents/trade is the wrong unit**
  (`execution/DESIGN.md` §3): report return on locked capital, and `break-even-win-rate`'s
  q*/q/q⁻ table, before any promotion claim.
- **A model whose only inputs are a closed feed cannot update; the market can.** Now
  `wiki/reference/stale-feed-gate.md`, adopted firm-wide. Rule 1 is the operational one:
  every row carries `feed_age_h` and `feed_open` — the ledger has columns for both since
  2026-07-27, plus `pricer_version`.
- **Checkpoint hygiene.** Anchor gate 1 at window-open and gate 2 at daily 12:00Z in-window,
  **never at board creation** (85% of legs quote ~0.50 there and a flat base-rate beats the
  market). This family is **nested**, so a literal leg-sum is vacuous — report `Σmid` vs
  `Σwinner`. Folded into `wiki/reference/checkpoint-artifact.md` on 07-28. Gate
  board-snapshots on `avg_mid ≤ 0.40` before quoting a Brier margin.
- **Resolution epsilon** (`wiki/reference/venue-resolution-epsilon.md`): 279/279 feed-touches
  resolved YES; 2/7 feed-*misses* within 0.10% resolved YES anyway → never sell a barrier
  within **0.2% of that leg's running window extreme**, from its TRUE window start. It screens
  *adjudication* risk, not price proximity — ↓85 sat 1.32% clear and touched anyway. Clear
  today; closest are gold-weekly ↓4000 1.37%, silver ↓54 1.44%, WTI ↓80 1.75%.
- **Five ways a quoted price can mislead**: dead book at 0.02/0.98 (`phantom-midpoints`);
  live-but-wide, mid ≠ bid (`midpoint-is-not-a-fill`); live-and-tight with no counterparty
  (`tape-gate`); an honest quote against **our** stale feed (`stale-feed-gate`); and a venue
  API that answers a filtered query with `[]` rather than an error.
- **FILL PICTURE SPLITS BY BOARD FAMILY** (`book-and-tape-audit-2026-07-26.md`): reachable
  fraction of the scored midpoint = WTI 99%, BTC 100%, silver 89%, gold 82%, **SPY/NVDA weekly
  38%**. The 2/21 headline was about equity weeklies and sub-3c wings, not the variant.
  **The reachable legs are the ones we lost on** — mid-board WTI legs at 0.4–0.7 are the liquid
  ones. Book gate: relative spread `≤ min(5c, ½·mid)`, mid ∈ [3c,97c], tape ≥1 taker trade on
  our side within 5c in 7 days.
- **Buys: unrescuable by drift/jump** (day-2, 255-leg sample). Failure is 100% crypto — the
  crypto market over-prices barrier touches ~4× (3–50c legs realized 0.038 vs mid 0.148).
  Class-gated WTI+equity buys are positive but only 12/39 legs → **buys stay OFF**.
- **GAP SD, MEASURED** (`ladderrv gaps`; weekend / overnight): USOILSPOT **3.78% / 0.35%**,
  WTIU6 4.25% / 0.40%, XAUUSD 0.74% / 0.13%, XAGUSD 1.20% / 0.18%, SPY 0.74% / **0.59%**,
  NVDA 1.38% / **1.43%**, BTC 0/0. A WTI weekend gap ≈ a whole session's variance; for RTH
  equity the **overnight** gap ≈ a whole session — we priced 17.5h of daily risk at zero τ
  until day 5, a plausible cause of losing to the market on SPY/NVDA.
- **Roll calendar / August**: CLQ6→CLU6 at the **Fri 17 Jul** session (the July monthly spans a
  roll — changed no answer, but that was luck), CLU6→CLV6 **Tue 18 Aug**, CLV6→CLX6 **Fri 18
  Sep**. **Archive WTIX6 the day it appears (~Aug 20)**; WTIQ6 is delisted forever. August is
  priceable (`ladderrv roll` validated) but not predictable, and resolves after the review.
- **Fine print (gate-0 verified 756/760)**: crypto monthlies resolve on BINANCE USDT;
  "calendar month" legs resolve from listing; re-added strikes carry private window starts
  ("after market creation"); equity RTH-only; WTI/metals weeklies use the session clock, not
  the calendar week; WTI boards resolve on the **active month**, which rolls mid-board.
- **Data traps**: Pyth carries exactly **two** CL contracts and deletes expired ones — archive
  daily. USOILSPOT is a CFD near the front, not the spliced active month (basis vs U6 p50
  $0.21 p95 $0.98 max $2.07) — never settle a near-barrier WTI question off it. Pyth
  rate-limits ~1 req/s. Polymarket headline volumeNum on WTI ≈ 20× real taker notional;
  `liquidityNum` is listed, not traded, size. **data-api.polymarket.com 403s without a
  User-Agent header.**
- Vol anchors verified free: CBOE OVX/VIX/GVZ/VXSLV CSVs, Deribit DVOL. NVDA has none.
- Session calendars assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3, **Sep 7**);
  Columbus Day is NOT a closure. Revisit before the 2026-11-01 EST transition.

## Long-term (wiki candidates)

- **Session-time vol models must carry a close-to-open gap term.** Amortising gap variance
  across session minutes gets the total roughly right and the *shape* wrong, and shape is what
  a first-passage question is about. Also: the same variance delivered as a jump gives a
  strictly smaller touch probability than delivered as diffusion (reflection counts round
  trips; a jump has no path) — in the jump-only limit exactly half.
- **"Active month" ladders spanning a CME roll contain a deterministic price gap** equal to the
  calendar spread, on a date known in advance. The right model is one diffusion with a
  **stepped barrier** plus absorption at the roll instant; the naive one-spot model errs
  40–110% and always in the direction that flatters a seller of the down wing. Generalises to
  any futures-referenced barrier market.
- **A cache whose key is "does the file exist" silently freezes partial data.** Any daily
  archiver that writes a file covering a period still in progress must record *when* it wrote
  it and refetch until the period closes. Cost here: two separate reproducibility failures,
  both in the flattering direction, and the second was invisible for four days.
- **A gate that cannot fail is worse than no gate**, because it gets written down as passed.
  Before adopting any coherence check, ask what data would make it fail. (leg-sum on a nested
  ladder; now folded into `checkpoint-artifact`.)
- ALREADY GRADUATED, don't re-derive: `stale-feed-gate`, `tape-gate`,
  `venue-resolution-epsilon`, `midpoint-is-not-a-fill`, `phantom-midpoints`,
  `favorite-longshot-bias`, `delayed-execution-test`, `checkpoint-artifact`.
- Resolution-feed archaeology matters: resolution sources get deleted. Archive the resolving
  feed while the market is live, or lose gate-0 forever.
