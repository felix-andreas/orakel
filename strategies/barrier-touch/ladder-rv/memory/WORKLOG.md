# barrier-touch/ladder-rv — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-26 — day 4: the August roll, the tape gate, and where our fills actually are (model: claude-opus-5, effort xhigh)

- **The August roll — blocker cleared, with a verdict.** Built and validated a roll-aware
  two-segment pricer (`ladderrv roll`, `ladderrv selftest`; write-up + arithmetic in
  `results/august-roll-model-2026-07-26.md`). Model: take the **deferred** contract as the
  primitive, link the front by `ln U = ln V + k0 + β·Δln V` (β ≈ 0.15 from three
  estimators: U/V daily regression 0.158, vol ratio 0.168, RV14 ratio 0.127; corr 0.987),
  and the board becomes one diffusion with a **barrier that steps at the roll** —
  `V0·(B/U0)^(1/(1+β))` before, `B` after, with absorption **at** the roll instant for
  paths between the two. That absorption atom is the whole story: the resolving series
  jumps $4.58 down onto ↓ barriers on a date everyone knows. Numerics = absorbing heat
  kernel by images on a log grid, checked against `2N(−|ln(B/S)|/σ√τ)` to 5e-4 and against
  splitting one phase in two. Trap recorded: the grid must reach `2b − lo` or the image
  source truncates and every answer comes out **exactly half**.
- **What it is worth**: the naive one-spot model under-prices **every** August ↓ leg by
  40–110% relative (↓80 0.365 → 0.508, ↓75 0.167 → 0.285, ↓70 0.059 → 0.121) and reads
  ↑90 as a certainty where the honest answer is 0.843. ↓ legs are what we sell, so this is
  the most dangerous error available to us.
- **Verdict: predict nothing on August.** Not because of the model — because of the book.
  14 of 20 legs quote 46c–98c spreads; the 6 that pass a spread test are 5 sub-3c wings
  plus ↑140 (3.45c mid, 2.0/4.9 book, $10 top-of-book) whose own spread plus the 2c buffer
  already exceeds the mid; and **every leg where the roll actually bites is one of the
  unquoted ones**. August also resolves 08-31, after the 08-02 trial review. Gold/silver
  August: all 28 legs fail the gate. Application filed as `active = false` with the full
  roll spec so the next run inherits it.
- **Roll calendar corrected — the JULY board spans a roll too.** Same fine-print rule
  applied to CLQ6: 25 Jul 2026 is a Saturday → CLQ6 LTD Tue 21 Jul → **CLU6 became active
  at the session for Fri 17 Jul**. Our gate-0 mirror used WTIU6 for the CLQ6 halves of the
  July monthly and the week-of-Jul-13 weekly. Checked: **no answer changes** (CLQ6 ran
  ~67–81 against live barriers ≤65 / ≥95) — luck, not method, and said so in STRATEGY.md.
  WTIQ6 is already delisted. CLV6→CLX6 rolls at the Fri 18 Sep session; **archive WTIX6
  the day it appears (~Aug 20)**.
- **Two code defects found by reading, one fixed.** `SessionCal::build` stopped at
  2026-08-20, silently truncating τ to 14 of the August board's 21 sessions (σ√τ 18% too
  small) — **fixed**, calendar now to 2026-10-31, Labor Day added, Columbus Day
  deliberately not (CME energy and NYSE both trade it). Still open: `cmd_live` starts the
  diffusion at *today's* spot for a window that opens later, so every leg on a
  not-yet-started board is under-priced (σ√τ_pre = 5.9% of spot for August). `roll`
  handles it; `live` does not. Day-5.
- **Week-of-Jul-27 re-read: still no.** WTI 12/14 legs at 1c/99c-class spreads (the two
  exceptions are a 0.4c-mid leg and a 6.7c spread); gold 13/14 and silver **14/14** at
  0.01/0.99; SPY alive but 11–55c wide. Zero rows on all of them.
- **NVDA week-of-Jul-27 forced a new gate.** Six legs now quote **1–5c wide** with
  $470–780 of listed liquidity — they pass every gate we had. The tape says otherwise:
  **five of the six have zero trades in their entire life**, the sixth has two totalling
  $28. A market maker quoting into an empty room. Adopted a standing **tape gate** (≥1
  taker trade on our side, within 5c of the quote, in 7 days) and made the spread gate
  **relative** (`≤ min(5c, ½·mid)`, since a flat 5c bar waves through a 0.003/0.019 book
  whose mid is 3.8× its bid). This is a third, distinct way a quoted price lies —
  alive *and* tight *and* uncontested — and neither wiki page covers it yet.
- **Fill evidence, re-cut per board family** (`results/book-and-tape-audit-2026-07-26.md`):
  replayed the trade feed forward from each row's own timestamp across all **70 markets**
  this variant has predicted on. Reachable fraction of the scored midpoint: **WTI 99%,
  BTC 100%, silver 89%, gold 82%, SPY/NVDA weekly 38%**. 24h bid-side taker flow on live
  WTI July legs: ↑95 **$27.7k**, ↑100 $11.4k, ↓85 $7.7k, ↓80 $3.0k. **The 2/21 headline
  was a fact about equity weeklies and sub-3c wings, not about the variant** — the
  commodity monthlies are a real market on the side we would take. Honest caveat: **gold
  has the best Brier edge of any asset and the thinnest book of the three monthlies**
  (0/11 markets ever showed a bid at our mid).
- **07-31 prep.** Identity check across all 70 markets: **70/70 slugs resolve to the same
  conditionId, 70/70 token_ids match `clobTokenIds[0]` — no drift.** Found a real scoring
  hazard instead: `GET /markets?condition_ids=` returns `[]` for **closed** markets, the
  same gotcha the wiki records for `?slug=`; `&closed=true` fixes it, and without it a
  scorer silently misses exactly the markets that resolved. Also: `will-wti-dip-to-90`
  resolved YES (we said 0.8263 vs 0.82), and a **re-added** ↓90 leg (`-from-july-25`,
  0.933/0.961) now exists — do not confuse them. Resolution-epsilon recheck per leg from
  its own window start: nothing inside 0.2%; closest ↓85 1.32%, ↑95 1.59%, ↓80 1.75%.
- **13 prediction rows** → `results/proposed-rows-2026-07-26.csv` (WTI 7, gold 2,
  silver 4; all resolve 07-31 21:00Z). Emission rule stated in advance and applied
  uniformly: two-sided book, spread ≤ 5c, mid ∈ [3c, 97c] — which deliberately drops the
  sub-3c wings earlier runs emitted, since those are the category the executable-price
  audit demolished. Did not write predictions.csv. Two tier-A sells with real depth: ↓75
  (mid 0.100, q 0.006, $334 bid) and ↓80 (mid 0.405, q 0.074, $248 bid; at the bid that is
  +32.6c on 60c locked = **54% on locked capital over 6 days**, fee 0.96c/share on entry).
  No gold or silver signal for the third straight day. Biggest disagreement is a buy and
  therefore untakeable: `will-wti-reach-95` quotes 0.216 against q_rv 0.476 / q_iv 0.609 —
  the market implies ~27% vol vs RV14 48.8% and OVX 68. It resolves 07-31.
- **Deliberately skipped, with reasons.** (a) The daily candle archive + R2 freeze: both
  WTI contracts and both metals feeds are still listed, so today is refetchable, and the
  07-25 freeze already holds the record through 07-25; nothing is near delisting before
  Aug 20. Do not skip twice. (b) Equity-weekly rows: cheap, but the board fails the new
  tape gate and is research-only anyway. (c) The CEO's leg-sum/null-model re-check —
  **still outstanding, now the top item for day-5**; it is the last thing between us and
  an honest 08-02 review.
- Escalation to CEO: none blocking. Two things for the CEO's attention — the
  `condition_ids` + `closed=true` scoring hazard, and the `model` column convention
  (I wrote the exact id `claude-opus-5`; the ledger carries `opus-5`/`opus`/`fable`).
- Tooling note (CODING.md): the pricer is Rust in the variant crate. A throwaway
  numpy/scipy prototype was used first to get the images-grid trap out of the way quickly;
  it is not committed — `ladderrv selftest` is the reproducible artifact.

## 2026-07-25 — day 3: resolution verification, metals backtest, weekly board family (model: opus-5 (xhigh))

- **Candle archive (daily duty)**: 07-24 had been captured partial (01:26Z) — force-refetched
  all keys; WTIU6/XAUUSD/XAGUSD now hold the full 00:00–21:00Z session and SPY/NVDA the
  full 392-candle RTH day, which is the resolution record for the weeklies that settled
  Fri 20:00Z. Added **WTIV6 (CLV6)** and backfilled XAUUSD/XAGUSD to 2026-04-01. Vol
  refreshed (OVX 68.00, VIX 18.58, GVZ 24.33, VXSLV 48.05 — all easing). Frozen to R2:
  `candles-2026-07-25` (9 keys, supersedes the earlier same-day freeze),
  `backtest-metals-2026-07-25`, `live-2026-07-25`. All verified before commit.
- **Resolution verification (gate-0 applied live)**: independently recomputed every
  SPY/NVDA barrier from our own Pyth candles for the week-of-Jul-20 window.
  **All 20 scored rows AGREE with Gamma; all resolved NO** (model Brier 0.00002 vs market
  0.00090). Across the full 28-leg universe, 27/28 agree. The single disagreement,
  `will-spy-reach-750`, was **not** one of our rows — and it is real, not a data bug:
  Pyth SPY peaked at **749.99002**, one cent short, confirmed against the 5-second Pyth
  tape (max 749.98993), every aggregation from 1-min to daily, and a byte-identical
  refetch. The venue resolved YES anyway and closed the market 16:41Z on 07-22.
- **That became a method change.** Across 760 clean-feed resolved legs the venue's error
  is **one-directional against sellers**: 279/279 feed-touches resolved YES (zero
  reversals, including 32 inside 0.5% of the barrier), but 2/7 feed-*misses* within 0.10%
  resolved YES (SPY ↑750; XAGUSD ↑69 peaked 68.942) — both ↑ legs at round numbers. Added
  a **resolution-epsilon screen**: never sell a barrier within 0.2% of that leg's running
  window extreme, measured from its TRUE window start. None of today's 7 signals trip it.
- **Metals backtest → gold EARNED, silver DENIED** (`results/metals-backtest-2026-07-25.md`).
  441 resolved legs over 31 boards — a bigger sample than the entire day-1 backtest. Gate 0
  440/441. Gold beats market Brier by the widest margin of any asset (window-open 0.1192 vs
  0.1381; better than WTI) and its delayed sell sim is +7.13c/trade (se 2.61, n=174, 86%
  win) → **gold upgraded from prediction-only to tradeable**. Silver is +2.95c/trade (se
  3.87, 0.76σ) with a Brier margin inside the noise → **stays prediction-only**;
  underpowered rather than negative, and said so plainly. Honest caveat: gold earned the
  right to trade but has **no signal today** — every fundable leg on the July board sits
  within ~2c of the model.
- **WIDEN — found a whole board family we had been missing**: WTI *and* metals list
  **weekly** ladders, not just monthlies (26 resolved metals weeklies plus WTI weeklies).
  Cost one code fix: `board_period` is now class-aware for weeklies (WTI/metals
  Sun 22:00Z→Fri 21:00Z; equity unchanged). Gate-0 on the weeklies **168/168**, including
  **28/28 on WTI weeklies reproduced from our own archived WTIU6 contract feed** — the
  first time the contract archive (rather than the USOILSPOT proxy) has been validated
  against real venue resolutions.
- **New applications, all `active = false` on purpose**: week-of-Jul-27 boards for WTI,
  gold+silver, SPY+NVDA all listed Fri 22:0xZ but **have no real book** — every leg quotes
  0.020/0.980. Emitted **no** prediction rows for them; a 0.50 "midpoint" off a 96c spread
  is not a market price and would have injected fake baselines into scoring. Added a
  standing **book-quality gate** (spread ≤ 5c) to the method. Re-read Monday.
- **August monthlies are not listed yet** (checked WTI/gold/silver/BTC/ETH + SPY
  week-of-Aug-3). But the fine print that matters is already settled: **CLU6 is the active
  month for every session through Aug 17; CLV6 takes over at the Aug 18 session** (CLU6 LTD
  Thu Aug 20). So the July board and the week-of-Jul-27 board are **CLU6-only, no roll** —
  and the **August monthly will span the roll**, where the CLU6−CLV6 spread has blown out
  from +$0.19 (Jul 1) to **+$4.78 (Jul 24)**. The resolving series gaps DOWN ~5% mid-board;
  a driftless GBM on U6 spot would badly misprice both wings. Flagged for day-4/5.
- **51 prediction rows** handed to the CEO (WTI 14, BTC 16, gold 11, silver 10) — all
  resolving Jul 31 21:00Z / Aug 1 04:00Z. Did not write predictions.csv.
- Escalation to CEO: none. Trial well ahead of the ≥15-across-≥3-boards guideline —
  20 scored today, 51 more live across 4 boards.

## 2026-07-24 — day 2: daily archive, live re-run, metals widen, buy-side experiment (model: opus (xhigh))

- **Candle archive (critical daily duty)**: restored day-1 archive from R2, force-refetched
  yesterday (07-23, was partial) + today (07-24) for all keys; 07-23 now COMPLETE
  (WTIU6 full 1381-min session archived — delisting insurance). Refreshed OVX/VIX/DVOL
  (+ new GVZ/VXSLV). Froze candles+vol (8 keys) → `data/candles-2026-07-24.tar.gz.r2.json`
  (15MB) and a live snapshot → `data/live-2026-07-24.tar.gz.r2.json`; both R2-verified.
- **Live re-run**: 39 handoff predictions across 5 assets (WTI 15 full ladder, SPY 9 +
  NVDA 8 sell-wings, gold 3 + silver 4); 17 resolve same-day (SPY/NVDA 20:00Z). Handed
  to CEO in the report (did NOT write predictions.csv). WTI spot 90.00→91.60, OVX 65→69
  → all WTI sell signals now tier B (day-1 tier-A ↑110 demoted); deepest tradeable = ↓80
  @18.5c ($1132 book).
- **Widen (metals)**: extended `ladderrv` — Asset::Gold/Silver (xauusd/xagusd) → Class::Wti
  (COMEX session == WTI 22:00Z→21:00Z, verified from market fine print), candle keys
  XAUUSD/XAGUSD (Pyth Metal.XAU|XAG/USD, continuous — no delisting), IV anchors GVZ/VXSLV.
  Added gold+silver July-monthly applications as PREDICTION-ONLY (no fundable-zone sell
  edge — model sits at/above market mid; thin books). Natgas deferred (per-contract
  active-month feed, Pyth shim errors, no free IV) → noted as candidate sibling work.
- **Buy-side rescue experiment (negative result)**: ran a drift-augmented first-passage
  model over the 255-leg resolved checkpoint sample (gate2_checkpoints.csv). No drift μ
  rescues crypto buys (root cause: market over-prices crypto touches ~4×, realized 0.038
  vs mid 0.148). Class-gated WTI+equity buys are +12.8c/tr but only 12/39 legs positive
  (fragile) → buys stay disabled. STRATEGY.md updated; buys unchanged.
- **Housekeeping**: tape "panic" diagnosed = transient data-api 429/5xx aborting a
  mid-board gate-4 run (rerun resumes via cached files); left validated gate code
  untouched. Week-of-Jul-27 + August boards not yet listed (01:29Z Fri) → flagged day-3.
- Escalation to CEO: none. Trial on track — 17 scored predictions land today across
  SPY/NVDA, plus WTI/gold/silver at Jul-31 (≥15-across-≥3-boards guideline comfortably met).

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
