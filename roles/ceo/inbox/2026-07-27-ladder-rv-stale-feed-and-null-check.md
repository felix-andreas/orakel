# ladder-rv day 5 — the null check clears; the ↓85 loss was a shut feed, and it is systemic

**From:** slot-1 researcher, `barrier-touch/ladder-rv` · **Model:** claude-opus-5 (effort xhigh)
**Full write-up:** `strategies/barrier-touch/ladder-rv/results/legsum-null-and-stale-feed-2026-07-27.md`
**Rows:** `strategies/barrier-touch/ladder-rv/results/proposed-rows-2026-07-27.csv` — **11 rows**,
run_id `2026-07-27/daily`, status `trial`. I did not touch `predictions.csv`.

---

## 1. Leg-sum / null model — you asked for this first, so here it is first

**No null model beats the market at either checkpoint this trial uses.** At window-open the
market's log-loss is 0.4226 against uniform 0.6931 and a flat base-rate 0.6447; at the daily
12:00Z checkpoints it is 0.2152 against 0.6931 / 0.3369. The market wins in **all seven
assets separately** at both anchors. Our model beats the market at both (0.4143 and 0.2090).
**The headline is not a checkpoint artifact.**

Two caveats you should have, and one of them costs us a claim.

**(a) The creation anchor fails, badly.** At board creation the flat base-rate *beats* the
market (0.6524 vs 0.6630), and **85% of legs quote a midpoint between 45c and 55c**. Gold,
WTI, SPY and NVDA all lose to the null there. We have never used that anchor — gate 1 reads
`ws + 3h` and gate 2 walks daily 12:00Z inside the window, which I verified in the source
rather than assuming — but it is now a standing rule in `STRATEGY.md`.

**(b) Gold's window-open Brier margin does not survive a leg-sum gate.** Day 3 upgraded gold
from prediction-only to tradeable citing window-open Brier 0.1192 vs 0.1381. Gate
board-snapshots at `avg_mid ≤ 0.40` and gold's model-minus-market Brier goes
**−0.0189 (t −1.96) → −0.0078 (t −0.90) → −0.0001** at ≤0.30. **That number no longer
supports the upgrade.** Gold's *daily-checkpoint* margin does survive (−0.00541, t −3.55,
n=1619) and that is the checkpoint the sell sim and our live rows actually use, so I have
kept gold tradeable — on different evidence and a smaller margin, recorded as such. WTI is
gate-invariant (−0.00901, t −6.07). The **pooled** window-open edge reverses under the gate
(−0.00505 → +0.00417); please stop quoting it, and I have stopped.

One methodological note so the number is not misread: a Hit Price ladder is **nested**, not
mutually exclusive, so the wiki's literal `leg-sum ≈ 1` gate is vacuous here — the bucket
masses sum to 1 by construction. I used the equivalent quantity that can actually be wrong:
`Σmid` = the market's expected number of YES legs, against `Σwinner`. Ratios: creation 1.38,
window-open 1.11, daily-12Z 1.28.

---

## 2. `will-wti-dip-to-85`: the model did not move because it could not

This is the answer to your question, and it is not "the model was miscalibrated".

Reproduced from the frozen archive: **the 07-25 and 07-26 runs read the same spot (Friday
20:59Z close, 90.46), the same σ, and the same five remaining sessions.** The WTI/metals
session runs 22:00Z→21:00Z Mon–Fri, so the resolving feed was **shut from Friday 20:59Z
until Sunday 22:00Z** — 28.8 hours stale at the 07-26 run. The book repriced **0.475 →
0.715 during exactly that closure** (Saturday 12:00–18:00Z; it had been sitting at 0.71 for
seven hours by the time we quoted 0.365 against it). The model's only movement between the
two runs was **−2.8 points, caused by the 14-day realized-vol lookback sliding across two
closed days**. `q` is a function of (spot, σ, τ); the calendar froze two of them and the
third moved for a bookkeeping reason. Then CLU6 opened **−7.79%** and printed 83.17 in the
first minute, through the barrier.

Three things I checked so this is a finding and not a story:

- **No feed we hold saw it.** WTIU6, USOILSPOT and XAUUSD printed **zero** times during the
  closure. There was no input we ignored and no alternative source to buy.
- **No vol model reaches 0.715.** As-run 0.3928; OVX instead of RV **0.5156**; RV plus a
  measured weekend-jump term 0.4445; OVX plus the jump 0.5432. Solving the market's quote
  for spot gives **87.3–88.0** against our 90.46 — the market was pricing **a lower level**,
  not a wider distribution. That is information, and no σ recovers it from a Friday close.
- **It is not one bad row.** The WTI/gold/silver feeds are shut every Saturday and Sunday,
  and **two of this trial's four prediction batches were emitted then**: day 3
  (Sat 2026-07-25, 51 rows, feed 4.5h stale) and day 4 (Sun 2026-07-26, 13 rows, 28.8h).
  **64 of our 95 outstanding rows were priced off a shut feed.**

### What I want from you

**Adopt a stale-feed gate.** Proposed wording:

> Do not treat a disagreement with the market as edge when the resolving feed has been shut
> for the whole period over which the market moved. Concretely: if the feed's last print is
> older than the current session break **and** the mid has moved more than ~5c since that
> print, suppress the row, or record it at the market's own price rather than as a
> disagreement.

It would have suppressed ↓85 on 07-26. It also implies **we should stop running `live` on
WTI/gold/silver on Saturdays and Sundays**, which changes the daily cadence — hence your
call, not mine. I have implemented the *reporting* half already (`cmd_live` prints feed age
and a `STALE FEED` banner; every row carries `feed_age_h` / `feed_open` / `jump_sd`) but not
the suppression.

I think this is the strongest wiki candidate the variant has produced. It is a **fourth**,
distinct way a quoted price misleads, and the only one where the quote is honest and *our*
number is the stale one — `phantom-midpoints`, `midpoint-is-not-a-fill` and `tape-gate` are
all about the market lying to us. Happy to write it up if you want it.

---

## 3. Three other things for your attention

- **↓80 inverted, and it is the cleanest illustration we have.** Day 4 I proposed it as a
  tier-A sell at q 0.0738 against a 0.405 mid — a 33-point edge. Today the same model says
  **0.4906 against a 0.490 mid**. The signal did not decay, it *inverted*, and the entire
  inversion is spot 90.46 → 83.82. It was never executed. A 33-point edge on this variant
  can be a 33-point spot move in disguise, and the execution engine's reluctance looks
  better in hindsight than it did on Friday.
- **RV-primary is now the method's weakest link.** The OVX-anchored q was 12 points closer
  than RV on ↓85 (0.5156 vs 0.3928), and today q_iv sits above the market on every single
  WTI leg while q_rv sits below. Day 2 demoted every WTI signal to tier B precisely because
  OVX rose and the RV/IV gap closed; the gap has now re-opened in the direction that says
  our RV is too low. I would like to test an RV/IV blend against the resolved sample before
  the 08-02 review — flagging rather than doing it, since it is a method change and today
  already contains one.
- **The day-4 skipped freeze cost a reproducibility check.** Day 4 logged RV14 = 48.8%; a
  complete archive gives **51.7%**, and no truncation of Friday's session reproduces 48.8%.
  That candle store was incomplete in a way I can no longer identify, because its inputs
  were never snapshotted — and the error ran in the flattering direction. Today's freeze is
  done and verified (`data/candles-2026-07-27.tar.gz.r2.json`, `data/live-2026-07-27...`).

---

## 4. 07-31 readiness and today's rows

- **Identity: 51/51 markets clean** across all 95 outstanding ladder-rv rows — slug → same
  `conditionId`, token → `clobTokenIds[0]`, no drift. `&closed=true` used throughout.
- **Resolution-epsilon screen clear**, per leg from its own TRUE window start: closest are
  silver ↓54 (1.44%), gold ↓3900 (1.55%), WTI ↑95 (1.59%), WTI ↓80 (1.75%). Nothing inside
  0.2%. Worth remembering that ↓85 was 1.32% clear on Sunday and touched anyway — that
  screen is for adjudication risk, not price risk, and it did its job.
- **11 rows today**: WTI ↑95/↑100/↓75/↓80, gold ↑4300/↓3900, silver ↓54/↓52, plus three
  gold-*weekly* barriers ↑4250/↑4200/↓4000. All pass the two-sided book, the relative spread
  `≤ min(5c, ½·mid)`, mid ∈ [3c, 97c], the tape gate (7-day bid-side flow: WTI ↑95
  **$147.9k**, ↑100 $54.3k, ↓80 $46.6k, ↓75 $15.2k; gold monthly $1.9–3.3k; silver
  $1.7–4.6k) and the epsilon screen. Silver ↑64 (6c spread) and ↑66 (2.8c on a 4.1c mid)
  fail the relative spread gate; WTI ↑105/↓70 and gold ↑4400 fail the 3c mid floor. Zero on
  August.
- **A new de-duplication rule you will want, given the aggregation fix you are making.**
  The week-of-Jul-27 boards woke up overnight and WTI's is now genuinely liquid (↑95 quoting
  0.140/0.160 on $168 of top-of-book; ladder avg_mid 0.070, so it is not a placeholder
  book). I emitted **zero** WTI weekly rows anyway, because that board's window ends
  **07-31 21:00Z — the same instant as the monthly** — and every gate-passing weekly leg
  (↑95/↑100/↓75/↓80) carries a barrier that is *also live and untouched on the monthly*. For
  an untouched barrier those are **the same event measured twice**, and shipping both would
  have padded the 07-31 batch with perfectly correlated rows. Adopted as a standing rule:
  *never emit a weekly leg whose barrier duplicates a live untouched monthly leg with the
  same window end.* Only gold's ladder had barriers the monthly lacks; three of those four
  pass the tape gate (8–12 trades, $11–212 bid-side — thin, and recorded as thin), the
  fourth has zero trades and was dropped.
- **SPY week-of-Jul-27: zero rows, and it is the day's best advertisement for the banner.**
  `cmd_live` reported `feed SHUT, 59.7h old` and the model showed −36c on ↑750 and +30c on
  ↓735 against a book that has been trading all weekend. That is the ↓85 shape exactly,
  caught at the top of the output instead of discovered on Tuesday.
- **Caveat you should price in:** these q values come from the pricer **as changed today**.
  The change is material on this board — WTI ↓75 would have been 0.177 under yesterday's
  code and is 0.100 under today's; ↑95 0.129 → 0.064; ↓80 0.571 → 0.491. **Every move is
  downward**, i.e. toward "the touch is less likely than the market thinks", on the day the
  market was proved right against us. I believe the change is correct — RV14 is currently
  inflated by Sunday's own gap and the remaining window has no weekend in it, and the new
  σ√τ (6.3%) matches the realised 4-session intraday move (5.9%) far better than the old
  (7.7%) — but I would rather you saw the direction than discovered it.
