# barrier-touch/ladder-rv — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- **Day-7 (2026-07-29): 12 proposed rows**, `results/proposed-rows-2026-07-29.csv`, run_id
  `2026-07-29/daily`, 15-col schema, header byte-identical to the ledger. WTI ↑95/↓75/↑85-from-jul-27/
  ↑90-from-jul-27, gold ↓3900, silver ↓54/↓52, gold-weekly ↑4200/↑4150, silver-weekly ↑62/↓56/↓55.
  89 two-sided legs read, **77 suppressed**: 22 stale-feed (all equity), 44 mid<3c, 10 relative
  spread, 1 tape gate. Zero de-dup (no surviving weekly barrier duplicates a live monthly).
  Inputs @01:14Z, **feed OPEN 0.0h**: CLU6 **82.71** (RV14 62.7%, intraday 50.7%, OVX 57.1),
  gold **4013.82** (18.8/18.1, GVZ 24.6), silver **57.00** (39.4/37.9, VXSLV 47.8).
- **WTI bounced: 81.84 → low 77.80 Tue → 82.71.** ↓80 touched at the Tuesday low and resolved
  YES; ↓75 survived. **OVX fell 60.6 → 57.1 and is now BELOW RV14 (62.7%)** for the first time
  in the trial — the IV-above-RV premise of the prereg is asset-specific and already softening.
- **FRIDAY POWER: the two n≥30 questions have DIFFERENT answers — say both.**
  (a) *Pricer split* (ledger rows, within `feed_open=1`): jump arm 25 → **37 rows / 19 markets**
  after today, ~49/~20 after Thursday. **Clears 30 in rows, NOT in markets**, and the readiness
  doc's own item 7 says markets is the honest unit. Today's 12 rows added only **4 new markets** —
  the board universe is exhausted, so more runs buy repeats, not power. **Settle rows-vs-markets
  before Friday, not after.** (b) *RV/IV prereg*: scores all legs in `data/out/predictions_*.csv`,
  not emitted rows — today's file alone has **67 legs** with q_rv/q_iv/q_blend, floor met.
- **dip-to-80 resolved YES: headline −0.0172/25 → −0.0466/31 rows** (CEO). It carried **2 jump-arm
  feed_open=1 rows**, which the readiness table had counted as outstanding — that is why (a) above
  is 37 and not 39.
- **CEO'S QUESTION ANSWERED: NOT exposure, NOT calibration — a one-sided TAIL**
  (`results/trend-exposure-2026-07-29.md`). On 5,927 checkpoints / 633 resolved legs, same
  `touch_prob` pricer as the losing rows: model **beats** market on **touched** legs (−0.01152,
  t −1.99) and is flat on untouched (−0.00046). WTI ↓ legs **trending ≥5% into the barrier** are
  the variant's **best** bucket (−0.01259, t −4.66). r(err, trend) = −0.031. The "structurally
  short downside touches" story is **refuted**. What IS real: per-leg sd 0.062, p99 +0.173, and
  **the 8 worst legs in the whole backtest are all `dip-to` legs** (silver/NVDA/SPY/gold — not a
  WTI fact). dip-to-80 = p98.7, dip-to-85 = p97; nested on one contract, so ~1 draw not 2.
  → 08-02 asks **"is this tail acceptable at our size, given how correlated the legs we hold are"**,
  which is a sizing/correlation question (`break-even-win-rate` q*/q/q⁻ + RoLC), not a Brier one.
  Caveats: backtest is the sample the method was gated on; `trend` is ex-post and **must never
  become a filter** (that is `lifetime-volume-is-look-ahead` exactly).
- **THE THIRD DATA BUG WAS NOT IN CODE** (`results/archive-audit-2026-07-29.md`).
  `data/out/` is **gitignored**, and the daily freeze is candles+vol only — so
  `predictions_2026-07-28.csv` was frozen **nowhere**, existing in one container. The `live-*`
  freeze is the one that carries `out/`, and day-6 never cut one. **Rescue-frozen today** in
  `live-2026-07-29`. A freeze is only as complete as the hand-written `tar` line that built it.
- **07-28's predictions file PREDATES q_iv/q_blend** — written 01:21:59Z, binary rebuilt 01:30Z,
  never regenerated. The prereg says those numbers are "already frozen"; they are not.
  **RV/IV is scorable from 07-29 + 07-30 only.** Did NOT re-derive: doing so after seeing today's
  numbers is a change to the comparison by the person who scores it.
  Also flagged pre-outcome: prereg says the anchor is **daily 12:00Z**, but `cmd_live` fires
  ~01:1xZ and that is what the q's carry. **Pick the anchor blind, before Friday 21:00Z.**
- **`cmd_clob` AND `cmd_tape` had the `cmd_candles` bug — both fixed today.** `fetch_all` skips on
  `p.exists()`; both fetch **growing** series. Stale clob60 would have scored Friday against a
  price history stopping **2026-07-25**, dropping legs one at a time via `price_at`'s max-age
  guard — no error, just a smaller n. Now one shared `complete_through(path, l.we)`. Selftest
  passes; **pricer untouched**.
- **Tape gate suppressed gold-weekly ↓3950** — 26 trades in 24h but all at 0.32 / 0.61+, none
  within 5c of the 0.380 bid. Fires correctly **as written** but for a reason it was not designed
  for (repricing, not an empty room). Applied unchanged; **do not retune it mid-trial**. Method
  question for after 08-02.
- **STALE-FEED GATE IS STRUCTURAL FOR EQUITY.** SPY/NVDA resolve on Pyth RTH 13:30–20:00Z; the
  daily trigger fires ~01:07Z, feed 5.3h shut. **The daily run can NEVER legally predict an equity
  board.** 22 legs suppressed again today. Still awaiting the CEO's call: second in-RTH run, or
  equity leaves the trial.
- **GAMMA `closed` IS A FILTER DEFAULTING TO `false`, NOT AN OVERRIDE.** `?condition_ids=<cid>`
  returns an OPEN market and `[]` for a closed one; `&closed=true` is the exact reverse. **No
  single query finds both — try both.** Friday's set will be MIXED. `?condition_id=` (singular) is
  silently ignored and returns an arbitrary market.
- **Day-8 queue**: (1) run Thursday — but know it adds ~0–3 new markets, not power; (2) daily
  archive **and a `live-*` freeze, every day** — candles alone loses `out/`; (3) settle
  rows-vs-markets for the pricer floor; (4) settle the RV/IV anchor (01:1xZ vs 12:00Z) blind;
  (5) fresh `discover` + `clob 60` after 07-31 21:00Z.

## Medium-term

- **Where the edge is**: sell overpriced wings/extension legs; delayed sim beats instant for
  sells. Model beats market Brier on **WTI and GOLD only** — use the **daily-checkpoint,
  leg-sum-gated** numbers (WTI −0.00901 t −6.07; gold −0.00541 t −3.55), never the window-open
  ones, which do not survive the gate. spy +0.0089, nvda +0.0052, btc +0.0083 (model worse),
  silver ~0. Sells by asset (delayed, per trade): wti +14.4c, spy +19.6c(n=18), gold +7.1c,
  eth +8.3c, btc +6.4c, silver +3.0c, nvda −4.8c. **Cents/trade is the wrong unit**
  (`execution/DESIGN.md` §3): report return on locked capital, and `break-even-win-rate`'s
  q*/q/q⁻ table, before any promotion claim.
- **The variant's risk is a fat one-sided tail on `dip-to` legs, not a calibration error.**
  Mean edge is real and comes from the touched legs. See day-7 above; do not re-derive.
- **A model whose only inputs are a closed feed cannot update; the market can.**
  `wiki/reference/stale-feed-gate.md`, firm-wide. Every row carries `feed_age_h` / `feed_open`.
- **Checkpoint hygiene.** Anchor gate 1 at window-open and gate 2 at daily 12:00Z in-window,
  **never at board creation** (85% of legs quote ~0.50 there and a flat base-rate beats the
  market). This family is **nested**, so a literal leg-sum is vacuous — report `Σmid` vs
  `Σwinner`. Gate board-snapshots on `avg_mid ≤ 0.40` before quoting a Brier margin.
- **Resolution epsilon** (`wiki/reference/venue-resolution-epsilon.md`): 279/279 feed-touches
  resolved YES; 2/7 feed-*misses* within 0.10% resolved YES anyway → never sell a barrier
  within **0.2% of that leg's running window extreme**, from its TRUE window start. It screens
  *adjudication* risk, not price proximity — ↓85 sat 1.32% clear and touched anyway.
- **Five ways a quoted price can mislead**: dead book at 0.02/0.98 (`phantom-midpoints`);
  live-but-wide, mid ≠ bid (`midpoint-is-not-a-fill`); live-and-tight with no counterparty
  (`tape-gate`); an honest quote against **our** stale feed (`stale-feed-gate`); and a venue
  API that answers a filtered query with `[]` rather than an error.
- **FILL PICTURE SPLITS BY BOARD FAMILY**: reachable fraction of the scored midpoint = WTI 99%,
  BTC 100%, silver 89%, gold 82%, **SPY/NVDA weekly 38%**. **The reachable legs are the ones we
  lost on** — mid-board WTI legs at 0.4–0.7 are the liquid ones. Book gate: relative spread
  `≤ min(5c, ½·mid)`, mid ∈ [3c,97c], tape ≥1 taker trade on our side within 5c in 7 days.
- **Buys: unrescuable by drift/jump** (255-leg sample). Failure is 100% crypto — the crypto
  market over-prices barrier touches ~4×. Class-gated WTI+equity buys are positive but only
  12/39 legs → **buys stay OFF**.
- **GAP SD, MEASURED** (weekend / overnight): USOILSPOT **3.78% / 0.35%**, WTIU6 4.25% / 0.40%,
  XAUUSD 0.74% / 0.13%, XAGUSD 1.20% / 0.18%, SPY 0.74% / **0.59%**, NVDA 1.38% / **1.43%**,
  BTC 0/0. A WTI weekend gap ≈ a whole session's variance; for RTH equity the **overnight** gap
  ≈ a whole session.
- **Roll calendar / August**: CLQ6→CLU6 at the **Fri 17 Jul** session, CLU6→CLV6 **Tue 18 Aug**,
  CLV6→CLX6 **Fri 18 Sep**. **Archive WTIX6 the day it appears (~Aug 20)**; WTIQ6 is delisted
  forever. August is priceable (`ladderrv roll` validated) but not predictable, and resolves
  after the review.
- **Fine print (gate-0 verified 756/760)**: crypto monthlies resolve on BINANCE USDT;
  "calendar month" legs resolve from listing; re-added strikes carry private window starts
  ("after market creation"); equity RTH-only; WTI/metals weeklies use the session clock, not
  the calendar week; WTI boards resolve on the **active month**, which rolls mid-board.
  **CLU6 traded ~69.8 on Jul 1 while the board resolved on CLQ6** — never reconstruct a July
  touch from WTIU6 before the 07-16 22:00Z roll.
- **Data traps**: Pyth carries exactly **two** CL contracts and deletes expired ones — archive
  daily. USOILSPOT is a CFD near the front, not the spliced active month — never settle a
  near-barrier WTI question off it. Pyth rate-limits ~1 req/s. Polymarket headline volumeNum on
  WTI ≈ 20× real taker notional. **data-api.polymarket.com 403s without a User-Agent header.**
- **`r2data verify` checks the blob, not the contents.** It HEADs the object and compares size.
  `candles-2026-07-27` verified OK daily while holding a WTIU6 file truncated to 21.9KB of 69.7KB.
  "Frozen and verified" means the bytes we uploaded are there — nothing more.
- Vol anchors verified free: CBOE OVX/VIX/GVZ/VXSLV CSVs, Deribit DVOL. NVDA has none.
- Session calendars assume EDT + 2026 US holidays (May 25, Jun 19, Jul 3, **Sep 7**);
  Columbus Day is NOT a closure. Revisit before the 2026-11-01 EST transition.

## Long-term (wiki candidates)

- **A cache whose key is "does the file exist" silently freezes partial data.** Now found in
  three places in one crate (`cmd_candles`, `cmd_clob`, `cmd_tape`) and, worse, **once outside
  code**: a daily freeze whose `tar` line omits a directory. The general rule is
  `complete_through(file, period_end)` — and the audit question is not "is the file there" but
  **"was this file written after the thing it describes stopped changing?"** Ask it of archives
  and of freeze scripts, not just fetchers.
- **A mean edge and a fat one-sided tail are different promotion questions.** Brier and
  cents/trade both average; neither sees that the 8 worst legs in 633 share a direction. Any
  sell-only variant needs its p99 leg and its holdings' correlation reported beside the mean.
- **Session-time vol models must carry a close-to-open gap term.** Amortising gap variance
  across session minutes gets the total roughly right and the *shape* wrong, and shape is what
  a first-passage question is about. The same variance delivered as a jump gives a strictly
  smaller touch probability than as diffusion — in the jump-only limit exactly half.
- **"Active month" ladders spanning a CME roll contain a deterministic price gap** equal to the
  calendar spread, on a date known in advance. Model it as one diffusion with a **stepped
  barrier** plus absorption at the roll instant; the naive one-spot model errs 40–110% and
  always in the direction that flatters a seller of the down wing.
- **A gate that cannot fail is worse than no gate**, because it gets written down as passed.
- ALREADY GRADUATED, don't re-derive: `stale-feed-gate`, `tape-gate`,
  `venue-resolution-epsilon`, `midpoint-is-not-a-fill`, `phantom-midpoints`,
  `favorite-longshot-bias`, `delayed-execution-test`, `checkpoint-artifact`,
  `lifetime-volume-is-look-ahead`.
- Resolution-feed archaeology matters: resolution sources get deleted. Archive the resolving
  feed while the market is live, or lose gate-0 forever.
