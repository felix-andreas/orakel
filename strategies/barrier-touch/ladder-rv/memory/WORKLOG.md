# barrier-touch/ladder-rv — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-29 — day 7: the third data bug was not in the code (model: claude-opus-5, effort xhigh)

- **Feed verified open, not assumed.** 01:14Z: WTI/gold/silver **OPEN, 0.0h** (Tuesday's session
  opened 07-28 22:00Z); SPY/NVDA **SHUT, 5.3h**, reopens in 12.3h. **12 rows proposed** from 89
  two-sided legs across 8 boards (`results/proposed-rows-2026-07-29.csv`, run_id
  `2026-07-29/daily`, header byte-identical to the ledger): WTI ↑95/↓75/↑85-from-jul-27/
  ↑90-from-jul-27, gold ↓3900, silver ↓54/↓52, gold-weekly ↑4200/↑4150, silver-weekly ↑62/↓56/↓55.
  **77 suppressed**: 22 stale-feed (all equity, again), 44 mid<3c, 10 relative spread, 1 tape gate.
  Zero de-dup this time — no surviving weekly barrier duplicates a live monthly. Did not write
  predictions.csv. CLU6 **82.71** (RV14 62.7%, intraday 50.7%, OVX 57.1), gold 4013.82, silver 57.00.
- **OVX has fallen below RV for the first time in the trial** (57.1 vs 62.7 total / 50.7 intraday).
  The pre-registration's premise — IV sits above RV on 62/62 legs — is already softening. It does
  not change the prereg (the decision rule is fixed and stays fixed), but Friday should not be
  surprised by it.
- **THE THIRD SILENT-DATA BUG WAS NOT IN CODE, AND IT IS THE WORST ONE**
  (`results/archive-audit-2026-07-29.md`). `data/.gitignore` ignores `out/`; the daily freeze tars
  `candles vol`. The freeze that carries `out/` is `live-*`, and **day-6 never cut one**. So
  `data/out/predictions_2026-07-28.csv` — the per-leg record behind day-6's 14 ledger rows — was
  frozen **nowhere** and existed in exactly one container. Verified by pulling
  `candles-2026-07-28.tar.gz` and listing it rather than reading its note. **Rescue-frozen today**
  in `live-2026-07-29` (with 07-27 and 07-29). The lesson generalises past fetchers: *a freeze is
  only as complete as the hand-written `tar` line that built it*, and `r2data verify` cannot see
  inside the tarball — it verified `candles-2026-07-27` OK every day while that archive held a
  WTIU6 file truncated to 21.9KB of 69.7KB.
- **And the 07-28 file does not contain what the pre-registration says it does.** Written
  01:21:59Z; `main.rs` and the binary rebuilt 01:30Z the same morning, after the `q_iv`/`q_blend`
  columns were added. Never regenerated. The prereg states those numbers are "already frozen in
  the daily archive" — they are not. **RV/IV is scorable from 07-29 and 07-30 only.** I did **not**
  re-derive 07-28's IV columns: the inputs exist in the frozen archive, but re-deriving them now,
  after seeing today's numbers, is a change to the comparison made by the person who scores it.
  Its power floor is met regardless — today's file alone carries **67 legs** with all three
  pricers, all `feed_open=1`, all resolving 07-31 21:00Z, against n ≥ 30.
- **Flagged pre-outcome, deliberately unresolved:** the prereg fixes the metric at the **daily
  12:00Z** checkpoint, but `cmd_live` fires ~01:1xZ and that is the timestamp the recorded q's
  carry ("12:00Z" was inherited from the backtest's gate-2 anchor). Scoring at 01:14Z needs no
  re-derivation; scoring at a true 12:00Z does. Both defensible; **choosing after Friday's outcome
  is not.** The CEO should pick blind, before 21:00Z.
- **Two more `exists()`-means-cached bugs, both fixed.** `cmd_clob` and `cmd_tape` fetch **growing**
  series and `fetch_all` skips any path that exists. Stale `clob60` would have scored Friday against
  a price history stopping **2026-07-25** — and it would not have errored: the file parses,
  `load_series` returns a valid series, and `price_at`'s max-age guard then drops each later
  checkpoint silently, so legs leave the scored set one at a time. `cmd_tape`'s version turns "any
  taker trade in the last 7 days" into "...in the 7 days before whenever we fetched". Both now use
  one shared `complete_through(path, l.we)`. `selftest` passes; **the pricer is untouched.**
- **CEO's question — calibration or directional exposure? Measured answer: neither**
  (`results/trend-exposure-2026-07-29.md`). Restored the 633-leg / 5,927-checkpoint backtest from
  R2; `cmd_analyze` prices it with plain `touch_prob`, the **same pricer as every losing row**, so
  it is apples-to-apples. The model **beats** the market on the **touched** legs (−0.01152, t −1.99)
  and is flat on the untouched (−0.00046). WTI ↓ legs with the underlying trending **≥5% into the
  barrier** are the variant's **best** bucket (−0.01259, t −4.66); r(err, trend) = −0.031. **The
  "structurally short downside touches" story is refuted on this data.** What is real is a
  **one-sided tail**: per-leg sd 0.062, p99 +0.173, and **the 8 worst legs in the whole backtest are
  all `dip-to` legs** — across silver, NVDA, SPY and gold, so it is a down-barrier fact, not a WTI
  one. dip-to-80 is a p98.7 draw and dip-to-85 a p97 draw, nested on one contract over one selloff,
  i.e. ~1 observation not 2. On the trial rows themselves the concentration is total: the 12 touched
  rows pool to +0.1391 and the 20 untouched to **−0.0011**, no row outside ±0.004. So 08-02 should
  ask **"is this tail acceptable at our size, given how correlated the legs we hold are"** — a
  sizing/correlation question (`break-even-win-rate` q*/q/q⁻ + RoLC), not a Brier one. Two caveats
  stated in the write-up: the backtest is the sample the method was gated on, and `trend` is
  computed ex post and **must never become a filter** — that is `lifetime-volume-is-look-ahead`
  exactly.
- **Friday's power, corrected — and the two n≥30 questions disagree.** `dip-to-80` resolving YES
  took 6 rows out of the outstanding set including **2 jump-arm `feed_open=1` rows** the readiness
  table had counted as outstanding. Measured today: jump arm 25 → **37 rows / 19 markets**, ~49/~20
  after Thursday. **Clears 30 in rows; does not clear it in markets and will not by Friday** — and
  the readiness doc's own item 7 says markets is the honest unit. Today's 12 rows added only **4**
  new markets: the board universe is exhausted, so further runs buy repeats, not power. Escalated
  rather than silently picking the flattering unit.
- **Tape gate suppressed gold-weekly ↓3950** — 26 trades in 24h, but all at 0.32 or 0.61+, none
  within 5c of the 0.380 bid. It fires correctly **as written**, for a reason it was not designed
  for (the leg repriced; it is not an empty room). Applied unchanged and logged: retuning a gate
  mid-trial, after seeing which leg it catches, is the thing the prereg exists to prevent.
- Archive: 07-28 was captured partial by day-6 and the mtime rule refetched it today —
  WTIU6 3998B → 69796B, SPY 52B `no_data` → 20436B. Froze and verified `candles-2026-07-29`
  (19.2MB, supersedes 07-28) and `live-2026-07-29` (92KB).
- Escalation to CEO: (a) **pick the pricer-split unit, rows or markets, before Friday**;
  (b) **pick the RV/IV anchor, 01:1xZ or 12:00Z, blind**; (c) the equity/RTH scheduling decision
  is still open and 22 legs were suppressed again; (d) one row of discrepancy between my
  reconstruction (32 rows / 21 markets) and the CEO's scoring run (31 / 20), worth a minute before
  the numbers are quoted.

## 2026-07-28 — day 6: first run under the gate, and two silent data bugs (model: claude-opus-5, effort xhigh)

- **The gate ran, and it bit.** Feed status at 01:21Z: WTI/gold/silver **OPEN, 0.1h old**
  (Monday's session opened 07-27 22:00Z); SPY/NVDA **SHUT, 5.4h old, reopens in 12.1h**.
  All **22 equity legs suppressed**, not re-priced. **14 rows proposed** from 93 two-sided
  legs across 8 boards (`results/proposed-rows-2026-07-28.csv`, run_id `2026-07-28/daily`,
  15-column schema): WTI ↑100/↑95/↓75/↓80/↑90-from-jul-27, gold ↑4300/↓3900, silver ↓54,
  gold-weekly ↑4250/↑4200/↓4000/↓3950, silver-weekly ↑63/↓55. Suppressions: 22 stale-feed,
  34 mid<3c, 18 relative spread, 3 weekly/monthly de-dup, 2 tape gate. Epsilon clear
  (closest gold-weekly ↓4000 at 1.37%). Did not write predictions.csv.
- **The equity suppression is structural, and that is the finding.** The RTH feed trades
  13:30–20:00Z; the daily trigger fires ~01:07Z. There is no hour of the daily cadence at
  which an equity board is legally predictable. **The daily run can never emit an equity
  row.** Either equity gets a second run inside RTH or it leaves the trial — CEO's call. The
  model was −12c to −22c against the SPY down wing today, i.e. the ↓85 shape exactly, which
  is what the gate is for.
- **`cmd_candles` was silently keeping a partial yesterday — for four days.** Only `today`
  was force-refetched; yesterday's file, written mid-day by yesterday's run, reported
  "cached" forever after. Today's archive held WTIU6 2026-07-27 at **21.9KB against a true
  69.7KB**, and **52-byte `no_data`** files for SPY and NVDA — Monday's whole RTH session
  missing from the σ inputs. This is the same failure that made day-4 log RV14 48.8% against
  a true 51.7%, and it had been silently re-arming every day since. Fixed: a day-file is
  refetched unless it was written after that day ended. Archive re-frozen and verified
  (`candles-2026-07-28`, 18.9MB, supersedes 07-27).
- **`live` takes ONE comma-separated argument.** My first three invocations used spaces and
  each priced only the *first* board while writing a complete-looking prediction file;
  `cmd_live` also overwrites `data/out/predictions_<date>.csv` per call. Caught by noticing
  the last slug of every run was missing. Documented in STRATEGY.md — this is a silent
  wrong-answer path, not an error path.
- **Gamma's `closed` is a FILTER defaulting to `false`, not an override.** Measured:
  `?condition_ids=<cid>` returns an OPEN market and `[]` for a closed one; `&closed=true` is
  the exact reverse. **No single query finds a market in both states.** My first identity run
  came back 0/58 because of it. Friday's set will be *mixed* — boards close 21:00Z, UMA lags
  — so a one-form scorer silently drops half. Worse: **`?condition_id=` singular is ignored
  rather than rejected**, returning an arbitrary market (it handed back "New Rihanna Album
  before GTA VI?" for a WTI condition id). Re-run both ways: **identity 58/58 clean.**
- **Backfill for the CEO, measured rather than estimated**
  (`results/ledger-backfill-2026-07-28.csv`): `feed_age_h` / `feed_open` / `pricer_version`
  for all 132 pre-existing rows, computed per row from the frozen candle archive and the
  session calendars. Confirms 07-26 at 28.8h; **corrects 07-25 from "4.5h" to 4.9h**. New
  and uncomfortable: **days 1–2 emitted 20 equity rows on a shut RTH feed as well** (14.1h
  and 5.7h), so the true count is **68 of 132 rows — 52% — priced off a shut feed**, not
  64/95, and **every equity row we have ever emitted was stale.** They resolved NO and scored
  well; that was luck, not cleanliness.
- **Pricer version, and the confound that decides how to score it.** Outstanding rows split
  `touch-prob` 50 open / **45 shut** against `touch-prob-jump` **25 open / 0 shut** — every
  stale row is also an old-pricer row, so on the shut side the two factors cannot be
  separated at all. **The pricer comparison must be run within `feed_open=1` only: 50 vs 25.**
  The jump arm is below the n≥30 floor, so Wednesday's and Thursday's runs are load-bearing:
  skip either and the split cannot be decided on Friday.
- **RV/IV pre-registered, not switched** (`results/prereg-rv-iv-blend-2026-07-28.md`). The IV
  anchor sits **above** realized vol on **62 of 62** WTI/gold/silver legs (OVX 60.6 vs 49.7,
  GVZ 24.1 vs 20.0, VXSLV 47.6 vs 40.2). A higher σ raises every touch probability, and we
  are sell-only, so IV *removes* sell signals rather than adding them: on today's 27 fundable
  legs **RV 4, blend 3, IV 1**. Decision rule, blend weight (w=0.5, never tuned), power floor
  and a tradeability veto are all fixed before the outcome; my stated expectation is on
  record too. `cmd_live` now records `q_iv`, `q_blend`, `sigma_rv`, `sigma_iv` per leg so
  Friday scores it from the frozen archive. One fairness fix made first, before any outcome:
  `q_iv` used raw IV where `q_rv` used the gap-bumped σ.
- **Wiki**: folded my own correction into `checkpoint-artifact.md` — the leg-sum gate assumes
  a *partition*, a one-touch ladder is *nested*, and there `leg-sum ≈ 1` **cannot fail**, so
  it returns CLEAN however badly the book is priced. The general form is `Σmid` vs `Σwinner`
  (1.38 at creation, 1.11 window-open, 1.28 daily). The page had been telling a future
  researcher to run a check that passes for free.
- **Friday readiness** written up (`results/friday-2026-07-31-readiness.md`): 120 outstanding
  rows over 58 markets, identity 58/58; 104 resolve Fri 21:00Z, 16 BTC Sat 04:00Z; Gamma's
  `endDate` for the monthlies (Aug 1 03:59Z) is **not** the resolution window (07-31 21:00Z).
- Applications updated: WTI-weekly and metals-weekly flipped to `active = true` (books are
  real now); WTI-weekly still yields **zero** rows because every fundable barrier duplicates
  a live monthly leg. Silver-weekly ↑62/↑61 failed the tape gate despite an actively traded
  board — zero taker trades within 5c in 7 days.
- Escalation to CEO: (a) the equity/RTH scheduling decision; (b) apply the backfill before
  Friday or 95 rows aggregate as `unversioned`; (c) Wed+Thu runs are required for the pricer
  split. Nothing blocking today's rows.

## 2026-07-27 — day 5: the null check clears, and a loss traced to a shut feed (model: claude-opus-5, effort xhigh)

- **The leg-sum / null-model re-check, finally run** (`results/legsum-null-and-stale-feed-2026-07-27.md`).
  First the honest translation: a Hit Price ladder is **nested**, not mutually exclusive, so
  the wiki's `leg-sum ≈ 1` gate is vacuous here — the bucket masses sum to 1 by
  construction. The equivalent quantity that *can* be wrong is **Σmid = the market's
  expected number of YES legs, against Σwinner**. Measured on 46 fully-resolved boards /
  760 resolved legs: creation **1.38** (and **85% of legs quote a mid between 45c and 55c**),
  window-open 1.11, daily-12Z 1.28.
- **Verdict: not an artifact at the anchors we use.** Log-loss vs four nulls (uniform, flat
  base-rate in-sample, leave-one-board-out base rate, clairvoyant per-board rate):
  **at board creation the null WINS** (market 0.6630 vs base-rate 0.6524; gold, WTI, SPY and
  NVDA all lose individually). **At window-open (0.4226) and daily-12Z (0.2152) the market
  beats every null in every one of the seven assets**, and those are the only two anchors in
  the code — gate 1 reads `ws + 3h`, gate 2 walks daily 12:00Z inside the window. Checked in
  the source, not assumed. Clip sensitivity 1e-4…1e-2 changes nothing.
- **But the leg-sum gate costs us one claim, and it is gold's.** Gating board-snapshots at
  `avg_mid ≤ 0.40`, model-minus-market Brier at **window-open** for gold goes
  **−0.0189 (t −1.96) → −0.0078 (t −0.90) → −0.0001** at ≤0.30. That is the number day 3
  used to upgrade gold to tradeable, and it does not survive. Gold's **daily-checkpoint**
  edge does (−0.00541, t −3.55, n=1619), and that is the checkpoint the sell sim and our
  live rows use — so gold stays tradeable on different evidence and a smaller margin. WTI
  is gate-invariant (−0.00901, t −6.07). The **pooled** window-open edge reverses
  (−0.00505 → +0.00417): stop quoting it.
- **`will-wti-dip-to-85-in-july-2026` — the CEO asked why the model did not move when the
  market did. It could not.** Reproduced all four runs from the frozen archive: the 07-25
  and 07-26 runs read the **same spot (Fri 20:59Z close, 90.46), the same σ and the same
  five remaining sessions**. The WTI/metals session is 22:00Z→21:00Z Mon–Fri, so the feed
  was shut from Friday 20:59Z to Sunday 22:00Z — **28.8 hours stale at the 07-26 run** — and
  the Polymarket book moved **0.475 → 0.715 during exactly that closure** (it happened on
  Saturday 12:00–18:00Z and was stable at 0.71 by the time we quoted 0.365 against it). The
  model's only movement between the two runs was −2.8 points from the 14-day RV lookback
  sliding across two closed days. **Our pricer is a function of (spot, σ, τ); the calendar
  froze two of them and the third moved for a bookkeeping reason.**
- **No feed we hold saw it, and no vol model reaches it.** WTIU6, USOILSPOT and XAUUSD all
  printed **zero** times during the closure — there was no input we ignored. CLU6 then
  opened **−7.79%** (83.68) and printed 83.17 in the first minute, so the barrier was
  touched in the opening minute; `...-dip-to-90-...-from-july-25` opened *below* its barrier.
  Re-pricing the 07-26 run under every available fix: as-run 0.3928, OVX instead of RV
  **0.5156**, RV + a weekend-jump term 0.4445, OVX + jump 0.5432 — **none reaches 0.715**.
  Solving the market's quote for spot gives **87.3–88.0**: the market was pricing a *lower
  level*, not a wider distribution. That is information, and no σ recovers it.
- **Two code defects closed with one model.** `touch_prob_jump` = first-passage with an
  explicit initial jump, which covers both (a) the pre-window diffusion missing since 07-26
  and (b) the close-to-open gap a leg faces when priced on a shut feed. Plus
  `realized_vol_intraday` + `gap_sd` to split total RV into its smooth part and measured
  gaps, and a new `ladderrv gaps` subcommand. Measured gap sd (weekend / overnight):
  **USOILSPOT 3.78% / 0.35%**, WTIU6 4.25% / 0.40%, XAUUSD 0.74% / 0.13%, XAGUSD 1.20% /
  0.18%, SPY 0.74% / **0.59%**, NVDA 1.38% / **1.43%**, BTC 0 / 0. **A WTI weekend gap
  carries about as much variance as a whole trading session**; for the RTH-only equity feeds
  the *overnight* gap does, which we have been pricing at zero τ since day 1 and is a
  plausible cause of the model losing to the market on SPY/NVDA.
- **`selftest` earned its keep.** `jump_sd = 0` reproduces the closed form to 0.000000, and
  the jump-only limit converges to `N(−|ln(B/S)|/j)` — exactly **half** the reflection value,
  because reflection counts paths that touched and came back and a jump has no path. It also
  caught a live sign error: my first jump used the martingale-in-*price* convention
  `exp(jZ − j²/2)`, which injects a −j²/2 **log**-drift making every ↓ leg likelier and every
  ↑ leg less likely — a systematic tilt flattering a seller of the up wing. The inequality
  failed on L legs only, which is what exposed it. Removed.
- **Honest flag on the direction of my own fix.** On today's board (Mon→Fri, no weekend
  left) the change lowers q: WTI ↓75 0.177 → 0.100, ↑95 0.129 → 0.064, ↓80 0.571 → 0.491.
  Every move is toward "the touch is less likely than the market thinks", on the day the
  market was proved right against us. I believe it is correct — RV14 is currently inflated
  by Sunday's gap and the horizon has no weekend in it, and the new σ√τ (6.3%) matches the
  realised 4-session intraday move (5.9%) far better than the old (7.7%) — but I am not
  going to let that pass unremarked.
- **Daily archive freeze done** (skipped on day 4). `data/candles-2026-07-27.tar.gz.r2.json`
  and `data/live-2026-07-27.tar.gz.r2.json`, both uploaded and verified before the manifest
  was committed. 07-25/26/27 force-refetched for all 9 keys. It also surfaced a cost of the
  day-4 skip: **the day-4 run logged RV14 = 48.8%, and a complete archive gives 51.7%** with
  no truncation of Friday reproducing 48.8%. That candle store was incomplete in a way that
  can no longer be identified, because its inputs were never snapshotted — and the error ran
  in the direction that flattered us.
- **07-31 readiness.** Identity re-check: **51/51 markets clean** (95 outstanding ladder-rv
  rows), slug → same conditionId, token → `clobTokenIds[0]`, `&closed=true` throughout.
  Resolution-epsilon screen per leg from its own TRUE window start: nothing inside 0.2%
  (closest silver ↓54 1.44%, gold ↓3900 1.55%, WTI ↑95 1.59%, WTI ↓80 1.75%).
- **↓80 inverted and is worth staring at.** Day 4 proposed it as a tier-A sell, q 0.0738
  against a 0.405 mid. Today the same model says **0.4906 against a 0.490 mid**. The signal
  did not decay — it inverted, and the entire inversion is spot 90.46 → 83.82. It was never
  executed. A 33-point "edge" on this variant can be a 33-point spot move in disguise.
- **11 prediction rows** → `results/proposed-rows-2026-07-27.csv` (run_id `2026-07-27/daily`):
  WTI ↑95/↑100/↓75/↓80, gold ↑4300/↓3900, silver ↓54/↓52, gold-weekly ↑4250/↑4200/↓4000.
  All clear the two-sided book, the
  relative spread `≤ min(5c, ½·mid)`, mid ∈ [3c, 97c], the tape gate (bid-side flow over 7
  days: WTI ↑95 **$147.9k**, ↓80 $46.6k, ↑100 $54.3k, ↓75 $15.2k; gold $1.9–3.3k; silver
  $1.7–4.6k) and the epsilon screen. Dropped: silver ↑64 (6c spread) and ↑66 (2.8c on a 4.1c
  mid) on the relative spread gate; WTI ↑105/↓70 and gold ↑4400 on the 3c mid floor. Zero on
  August. Did not touch predictions.csv.
- **The week-of-Jul-27 boards woke up, and produced a new rule rather than a pile of rows.**
  WTI and gold are genuinely priced now (weekly avg_mid 0.070 and 0.193, no placeholder legs,
  WTI ↑95 quoting 0.140/0.160 on $168). But **their window ends 07-31 21:00Z, the same
  instant as the monthly**, and every gate-passing WTI weekly leg (↑95/↑100/↓75/↓80) carries
  a barrier that is also live and untouched on the monthly board — for an untouched barrier
  that is **the same event measured twice**. Adopted: *never emit a weekly leg whose barrier
  duplicates a live untouched monthly leg with the same window end.* Zero WTI weekly rows.
  Gold's weekly ladder has four barriers the monthly does not (↑4250/↑4200/↓4000/↓3850);
  three pass the tape gate (12/9/8 trades, $11/$71/$212 bid-side — thin, and recorded as
  thin), ↓3850 has zero trades and fails. **Three rows added, total 11.**
- **The new banner earned its keep within the hour.** SPY week-of-Jul-27 printed
  `feed SHUT, 59.7h old` (RTH closed since Friday 20:00Z) with the model −36c on ↑750 and
  +30c on ↓735 against a book that traded all weekend. That is the ↓85 shape exactly, and it
  is now visible at the top of the output instead of buried in a spread column.
- **Escalation to the CEO** (`roles/ceo/inbox/2026-07-27-ladder-rv-stale-feed-and-null-check.md`):
  (1) the null check clears, with the gold window-open correction; (2) a proposed **stale-feed
  gate** — **64 of our 95 outstanding rows were priced on a shut feed** (day 3 Saturday, day 4
  Sunday), so this is not one bad row; (3) RV-primary is now the method's weakest link, since
  the OVX anchor was closer on ↓85 and is above the market on every WTI leg today.
- Tooling note (CODING.md): the leg-sum/null tables and the divergence reproduction were done
  in throwaway Python against the frozen CSVs (fast, one-shot, not committed); everything
  that has to be reproducible went into the Rust crate — `ladderrv selftest` and
  `ladderrv gaps` are the artifacts.

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
