# The leg-sum / null-model re-check, and why the model did not move when the market did

**Date:** 2026-07-27 · **Run by:** slot-1 researcher · **Model:** claude-opus-5 (effort xhigh)
**Inputs:** `data/backtest-metals-2026-07-25.tar.gz` (916 legs / 58 boards, 760 resolved),
`data/candles-2026-07-27.tar.gz` (frozen today, contains the Sunday open that decided this)
**Code:** `ladderrv selftest` / `ladderrv gaps` (`src/main.rs`)

---

## 0. The two answers, up front

**The null model does not beat the market at either checkpoint this trial uses.** At
window-open and at the daily 12:00Z checkpoints — the anchors behind every number we have
quoted — the market's log-loss beats uniform and beats a flat base-rate, in every asset,
by a wide margin. The trial's headline is not the artifact `checkpoint-artifact.md`
warns about. **But** a null *does* beat the market at the board-**creation** anchor
(0.6630 vs 0.6524), and a leg-sum gate **erases gold's window-open Brier edge** — the
number that earned gold its tradeable upgrade on day 3. Gold survives on its
daily-checkpoint evidence, not on the one we quoted. Details in §1–§2.

**`will-wti-dip-to-85-in-july-2026` was not a calibration failure. It was a stale-feed
failure.** On 2026-07-26 01:45Z our spot was **28.8 hours old** — the resolving feed had
been shut since Friday 21:00Z and does not reopen until Sunday 22:00Z. Over exactly that
closure the Polymarket book went **0.475 → 0.715**. Our pricer is a function of (spot, σ,
τ); two of those three were frozen by the calendar and the third moved only because a
rolling window slid. **The model could not move. It was not wrong about the distribution;
it was reading a price from the past.** CLU6 then opened **−7.79%** and printed 83.17
inside the first minute, through the barrier. §3.

---

## 1. Leg-sum

### 1.1 The literal gate does not apply, and here is the honest translation

`wiki/reference/checkpoint-artifact.md` says: in a **mutually exclusive** family the
de-vigged legs sum to ≈1.0, and a sum well above that means the book is not priced yet.

A Hit Price ladder is **not** mutually exclusive — its legs are **nested**
(`{M ≥ H₁} ⊇ {M ≥ H₂} ⊇ …`). The bucket masses `p(Hᵢ) − p(Hᵢ₊₁)` sum to 1 *identically*,
so "leg-sum = 1" is true by construction and tells us nothing. Reporting a number called
leg-sum here without saying that would be worse than not reporting one.

The structurally equivalent, checkable quantity is:

> **L(t) = Σᵢ midᵢ = the market's expected number of legs on this board that resolve YES**,
> against **Y = Σᵢ winnerᵢ**, the realised number.

That is a real forecast with a real answer key, it is the same arithmetic the wiki's gate
performs, and — unlike the nested bucket sum — it can be wrong. Two structural tells go
beside it: `avg_mid` (→ 0.5 when every leg quotes the 0.02/0.98 placeholder) and the
monotonicity violation mass along each side of the ladder.

### 1.2 Measured, 46 fully-resolved boards, 760 resolved legs

| checkpoint | boards | legs | Σ mid | Σ win | **ratio** | avg_mid | frac legs mid∈[.45,.55] | viol/leg |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **creation** (first print) | 43 | 603 | 297.0 | 216 | **1.38** | 0.493 | **0.852** | 0.0034 |
| creation + 6h | 43 | 603 | 283.0 | 216 | 1.31 | 0.469 | 0.695 | 0.0031 |
| creation + 24h | 43 | 603 | 260.0 | 216 | 1.20 | 0.431 | 0.478 | 0.0039 |
| **window-open** (gate 1) | 46 | 706 | 271.7 | 244 | **1.11** | 0.385 | 0.198 | 0.0040 |
| **daily 12:00Z** (gate 2) | 46 | 5927 | 798.7 | 625 | **1.28** | 0.135 | 0.045 | 0.0014 |

At **creation, 85% of legs quote a midpoint between 45c and 55c.** That is not a market;
that is `phantom-midpoints.md` at ladder scale. Worst boards at that anchor:
`will-wti-hit-week-of-july-13-2026` ratio 3.60 (avg_mid 0.515),
`will-xauusd-hit-week-of-may-18-2026` 3.37, `what-price-will-xauusd-hit-in-april-2026` 2.60.

By window-open it is largely gone (frac_half 0.198) and by the daily checkpoints it is
gone (0.045).

The daily-12Z ratio of **1.28** is not contamination — it is the strategy's own thesis
stated as an aggregate: across 5,927 board-snapshots the market's expected YES count runs
**28% above** the realised count. Systematic over-pricing of touch is exactly what we sell.

**Neither anchor we use is creation-anchored.** Verified in code, not assumed: gate 1
reads `price_at(ws + 3h)` and gate 2 walks daily 12:00Z timestamps inside `[ws, we)`.

---

## 2. Null models against the market's own log-loss

Four nulls, all run through the same pipeline as the market and the model:

- **U** — uniform 0.5 (log-loss = ln 2 = 0.6931 by construction)
- **B** — flat base-rate, pooled **in-sample** (deliberately generous: it sees the answer key)
- **BX** — flat base-rate, leave-one-board-out (the honest version)
- **C** — each board's own realised YES fraction (clairvoyant; an upper bound on any null)

Log-loss, clipped at ε = 1e-3 (the whole table is stable from ε = 1e-4 to 1e-2 — wing legs
quote 0.0005, so this had to be checked rather than assumed):

| checkpoint | n | base | **market** | U | B | BX | C | verdict |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| **creation** | 603 | 0.358 | **0.6630** | 0.6931 | **0.6524** | 0.6549 | 0.5997 | **NULL WINS** |
| creation + 6h | 603 | 0.358 | 0.6091 | 0.6931 | 0.6524 | 0.6549 | 0.5997 | market (gold, spy still lose) |
| creation + 24h | 603 | 0.358 | 0.5368 | 0.6931 | 0.6524 | 0.6549 | 0.5997 | market, everywhere |
| **window-open** | 706 | 0.346 | **0.4226** | 0.6931 | 0.6447 | 0.6470 | 0.5947 | **market, every asset** |
| **daily 12:00Z** | 5927 | 0.105 | **0.2152** | 0.6931 | 0.3369 | 0.3394 | 0.2994 | **market, every asset** |

At the creation anchor the null wins overall and in **gold (0.6761 vs 0.6063), WTI (0.5060
vs 0.4597), SPY (0.6833 vs 0.5983), NVDA (0.6944 vs 0.6931)**. Silver is the only asset
where the market survives there, and only because silver's base rate happens to sit at 0.51.

**At the two anchors we actually use, no null comes close** — at daily-12Z the market's
0.2152 beats the best null by 0.084 nats, and it wins in all seven assets separately.
The model (q_rv) beats the market at both: 0.4143 vs 0.4226 at window-open, 0.2090 vs
0.2152 at daily-12Z.

> **So: the answer to the CEO's question is no. Our best result is not a null-model
> artifact.** Any future run that anchors on board creation would be one, and this is now
> the reason to keep the anchor where it is.

### 2.1 The leg-sum gate does bite — on gold's window-open number

Rule 1 of the wiki is to gate checkpoints on leg-sum. Applying it as `avg_mid ≤ 0.40`
(drop board-snapshots that are still substantially unpriced), model-minus-market Brier —
negative means our model is better:

**window-open**

| asset | ungated | ≤0.40 | ≤0.30 |
|---|---:|---:|---:|
| **ALL** | **−0.00505** (t −1.14) | **+0.00417** (t 0.97) | +0.00620 (t 1.42) |
| WTI | −0.01790 (t −1.59) | −0.01585 (t −1.55) | −0.01066 (t −0.84) |
| **gold** | **−0.01890 (t −1.96)** | **−0.00778 (t −0.90)** | **−0.00006 (t −0.01)** |
| silver | −0.00493 | +0.02464 | — |

**daily 12:00Z**

| asset | ungated | ≤0.40 | ≤0.30 |
|---|---:|---:|---:|
| ALL | −0.00162 (t −1.92) | −0.00075 (t −0.93) | −0.00144 (t −1.91) |
| **WTI** | **−0.00901 (t −6.07)** | **−0.00901 (t −6.07)** | **−0.00901 (t −6.07)** |
| **gold** | **−0.00657 (t −4.25)** | **−0.00541 (t −3.55)** | −0.00581 (t −3.94) |
| silver | −0.00303 | +0.00006 | −0.00019 |
| BTC | +0.00830 (t 7.30) | +0.00830 | +0.00830 |
| SPY / NVDA | +0.0089 / +0.0052 | unchanged | +0.0003 / +0.0003 |

Two things follow, and one of them costs us a claim:

1. **The pooled window-open edge reverses under the gate** (−0.00505 → +0.00417). The
   day-1 headline "model beats market Brier at window-open" is, pooled, a statement about
   boards that were not yet priced at window-open. It should not be quoted again.
2. **Gold's window-open margin is the casualty.** Day 3 upgraded gold from prediction-only
   to tradeable citing window-open Brier 0.1192 vs 0.1381. Gate it and the margin goes to
   −0.0078 (t −0.90), then to zero. **That specific number no longer supports the upgrade.**
   Gold's *daily-checkpoint* edge does survive the gate at −0.00541, t −3.55, n=1619, and
   that is the checkpoint the sell simulation and our live predictions actually use — so
   gold stays tradeable, but on different evidence than we said, and with a smaller margin.
   WTI is unaffected (−0.00901, t −6.07, gate-invariant: no WTI daily snapshot has
   avg_mid > 0.40).

None of this is a fill claim. `executable-price-audit-2026-07-25.md` and
`book-and-tape-audit-2026-07-26.md` remain the binding constraint, and §3 below is a
reminder that being better-calibrated on average is compatible with being catastrophically
wrong on the one leg that trades.

---

## 3. `will-wti-dip-to-85-in-july-2026`: why the model did not move

### 3.1 The four runs, reproduced from the frozen archive

| run | spot | spot age | RV14 | OVX | sessions left | q_rv (replicated) | q logged | market |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 2026-07-23 01:30Z | 88.57 | 0.0h | 52.0% | 69 | 6.8 | 0.6309 | 0.4937 | 0.525 |
| 2026-07-24 01:29Z | 91.54 | 0.0h | 52.7% | 68 | 5.8 | 0.3562 | 0.3520 | 0.410 |
| 2026-07-25 01:29Z | 90.46 | **4.5h** | 51.7% | 68 | 5.0 | 0.3928 | 0.3928 | 0.415 |
| **2026-07-26 01:45Z** | **90.46** | **28.8h** | 51.7% | 68 | **5.0** | **0.3928** | **0.3650** | **0.715** |

Read the last two rows. **Same spot, same σ, same number of remaining sessions.** The
07-26 run had no new information of any kind: the WTI session runs 22:00Z→21:00Z Mon–Fri,
so the feed closed Friday 20:59Z and did not reopen until Sunday 22:00Z. Between the two
runs the market moved **+30 points** and the model moved **−2.8 points**, and the −2.8 came
from the 14-day realized-vol lookback sliding across two closed days. That is the whole
answer to "why did the model not move": **it is a pure function of a feed that was shut.**

Verified rather than assumed — prints available to us between Friday 21:00Z and Sunday
22:00Z: **WTIU6 zero, USOILSPOT zero, XAUUSD zero.** We do not hold, and Pyth does not
offer, any feed that traded during that window. There was no alternative input we ignored.

The Polymarket tape shows the repricing happened on **Saturday 2026-07-25**: 0.475 at
11:00Z → 0.555 at 13:00Z → 0.665 at 14:00Z → 0.715 by 18:00Z, and it sat at 0.705–0.72 all
through Sunday, i.e. it was already there and stable when we quoted 0.365 against it.

### 3.2 What happened at the open

| | |
|---|---|
| Fri 2026-07-24 20:59Z close | CLU6 **90.46** |
| Sun 2026-07-26 22:00Z open | CLU6 **83.68** — gap **−7.79%** |
| low in the first 60 minutes | **83.17** (−8.40% vs Friday) |
| barrier 85 first touched | **2026-07-26 22:00Z**, the opening minute |

`will-wti-dip-to-90-in-july-2026-from-july-25` opened *below* its barrier, so it was YES
before a single minute of its window had elapsed. Our 0.9409 vs the market's 0.9470 on
that leg is noise; the leg was decided by the same gap.

### 3.3 The one-cent question: could a better model have got there?

Re-pricing the 07-26 run under every fix available:

| model | q | market 0.715, outcome YES |
|---|---:|---|
| as run (RV14, no gap term) | 0.3928 | |
| IV anchor instead of RV (OVX 68) | **0.5156** | RV-primary cost us 12 points here |
| RV14 + weekend-jump term (sd 3.78%) | 0.4445 | |
| OVX + weekend-jump term | 0.5432 | |

**None of them reaches 0.715, and that is the important result.** Solve for the spot that
reproduces the market's quote: at RV14 the market was pricing **87.29**, at OVX **88.03** —
2.7–3.5% below our 90.46. The market was not pricing a wider distribution around Friday's
close. It was pricing **a lower level**. It knew something about the weekend that no vol
model recovers from a Friday close. (It did not know how much: the actual open was 83.68,
below what even the market implied.)

So the fix is not a better σ. **The fix is to stop treating a disagreement with the market
as edge when the market has been trading and our feed has not.**

### 3.4 The gap variance, measured — a real defect regardless

`realized_vol` walks consecutive in-session 5-minute closes, so a close-to-open return *is*
inside its sum of squares — but it is charged 5 minutes of the denominator, and `tau`
then re-spreads it smoothly across session time. For a first-passage question that is the
wrong shape: a gap is a lump landing at a known instant, and a barrier can be jumped clean
over. `ladderrv gaps`, over 2026-04-01 … 2026-07-27:

| feed | overnight gap sd | **weekend gap sd** | intraday RV14 |
|---|---:|---:|---:|
| USOILSPOT | 0.35% | **3.78%** | 53.8% |
| WTIU6 | 0.40% | **4.25%** | 49.8% |
| WTIV6 | 0.25% | 3.24% | 43.0% |
| XAUUSD | 0.13% | 0.74% | 20.3% |
| XAGUSD | 0.18% | 1.20% | 40.6% |
| SPY | **0.59%** | 0.74% | 8.7% |
| NVDA | **1.43%** | 1.38% | 29.8% |
| BTCUSDT | 0.00% | 0.00% | 29.1% |

Two facts worth keeping:

- **A WTI weekend gap carries about as much variance as an entire trading session**
  (weekend rms 3.78% vs intraday session rms 3.56%), while its *overnight* gap is a tenth
  of that — CME crude only pauses an hour. The 2026-07-26 gap was 2.1 rms events.
- **For the RTH-only equity feeds the overnight gap is comparable to the whole session**
  (SPY 0.59% vs 0.66% intraday). Our equity ladders have been pricing 17.5 hours of daily
  risk at zero τ since day 1. That is a plausible part of why the model loses to the
  market on SPY/NVDA Brier (+0.029 / +0.020) while beating it on WTI and gold.

---

## 4. Method and code changes made today

1. **`touch_prob_jump`** — first-passage with an explicit initial jump before the diffusion
   starts. It closes **both** open defects with one model: (a) the leg whose window opens
   later now gets its pre-window diffusion (logged unfixed on 07-26), and (b) a leg priced
   while the feed is shut gets the coming close-to-open gap as a lump. A path that gaps
   past the barrier touches **at the open** — the venue reads the candle.
2. **`realized_vol_intraday` + `gap_sd`** — total RV is split into its smooth part and
   measured close-to-open gaps, so a weekend-free horizon is not charged weekend variance
   and a weekend-spanning one is. On today's board (Mon→Fri, no weekend) this takes WTI's
   effective σ from 61.1% to 49.8%+, which checks out against the realised 4-session
   intraday move (5.9% vs the model's 6.3%; the old model said 7.7%).
   **Direction noted honestly: on this particular board the fix makes us more confident,
   i.e. it flatters a seller, on the day after a gap cost us. It is right for the right
   reason — RV14 is currently inflated by Sunday's gap and there is no weekend left in the
   window — but it wants watching.**
3. **`cmd_live` reports staleness**: feed age, open/shut, and a loud `STALE FEED` banner;
   every row now carries `feed_age_h`, `feed_open`, `jump_sd`.
4. **Proposed standing gate — the stale-feed gate** (for the CEO; not yet adopted):
   > Do not treat a disagreement with the market as edge when the resolving feed has been
   > shut for the whole period over which the market moved. Concretely: if the feed's last
   > print is older than the current session break **and** the mid has moved more than
   > ~5c since that print, suppress the row or record it at the market's own price.

   It would have suppressed `dip-to-85` on 07-26 (28.8h stale, +30c of market move). It
   also flags a systemic exposure: **the WTI/gold/silver feeds are shut every Saturday and
   Sunday, and two of this trial's four prediction batches were emitted then** — day 3
   (2026-07-25, Saturday, 51 rows, feed 4.5h stale) and day 4 (2026-07-26, Sunday, 13 rows,
   feed 28.8h stale). Sixty-four of our 95 outstanding rows were priced off a shut feed.
5. **`selftest` extended.** `jump_sd = 0` reproduces the closed form to 0.000000; the
   jump-only limit converges to `N(−|ln(B/S)|/j)`, exactly **half** the reflection-principle
   value, because reflection counts paths that touched and came back and a jump has no path.
   The first implementation used the martingale-in-price convention `exp(jZ − j²/2)`, which
   injects a `−j²/2` **log**-drift that makes every ↓ leg likelier and every ↑ leg less
   likely — a systematic tilt favouring a seller of the up wing. The selftest caught it as
   an inequality violation on L legs only. Removed; the jump is now driftless in log price,
   consistent with `touch_prob`.

### A data-integrity note that the skipped freeze caused

The day-4 run recorded RV14 = 48.8%. **From a complete archive the correct value is
51.7%**, and no truncation of Friday's session reproduces 48.8% (the estimator sits at
51.2–51.9% for every cut). The day-4 candle store was therefore incomplete in some way we
can no longer identify, **because day 4 skipped the daily freeze and its inputs were never
snapshotted.** The error ran in the flattering direction: lower σ → lower q → larger
apparent edge against a market that was already 30 points away. Frozen today:
`data/candles-2026-07-27.tar.gz.r2.json` (uploaded and verified before this file was
committed) and `data/live-2026-07-27.tar.gz.r2.json`.

---

## 5. 07-31 readiness

- **Identity: 51/51 clean.** Every outstanding market (95 ladder-rv rows across 51 markets)
  re-resolves to the same `conditionId` and the same `clobTokenIds[0]`. No slug drift, no
  token drift. `&closed=true` used throughout — the hazard flagged on 07-26 is real and
  affects exactly the markets that will have resolved.
- **Resolution-epsilon screen, per leg from its own TRUE window start**, against today's
  archive: nothing inside 0.2%. Closest are silver ↓54 (1.44%), gold ↓3900 (1.55%),
  WTI ↑95 (1.59%), WTI ↓80 (1.75%). All clear.
- **↓80 is the leg to watch.** Spot 83.82, barrier 80, four sessions left, running window
  low 81.397. Yesterday we proposed it as a **tier-A sell at q 0.0738 against a 0.405 mid**;
  today the same model says **0.4906 against a 0.490 mid**. The signal did not decay, it
  inverted, and the whole inversion is the spot moving 90.46 → 83.82. It was never
  executed. It is the clearest possible illustration that a 33-point "edge" on this variant
  can be a 33-point spot move in disguise.

---

## 6. Today's rows — 8, all July monthlies

Gates applied uniformly and stated in advance: two-sided CLOB book; **relative** spread
`≤ min(5c, ½·mid)`; mid ∈ [3c, 97c]; **tape gate** (≥1 taker trade on the bid side within
5c of the quote in the last 7 days); resolution-epsilon screen.

Tape, last 7 days, bid-side (YES-equivalent, folded as `tools/fillcheck` does):

| leg | mid | trades 7d | bid-side ≤5c | bid $ 7d | bid $ 24h | gate |
|---|---:|---:|---:|---:|---:|---|
| WTI ↑95 | 0.0915 | 3182 | 51 | **$147,857** | $10,747 | PASS |
| WTI ↑100 | 0.0370 | 1907 | 176 | $54,308 | $2,792 | PASS |
| WTI ↓75 | 0.1600 | 869 | 192 | $15,187 | $1,744 | PASS |
| WTI ↓80 | 0.4900 | 1499 | 102 | $46,590 | $11,385 | PASS |
| gold ↑4300 | 0.0590 | 394 | 116 | $1,856 | $64 | PASS |
| gold ↓3900 | 0.0785 | 329 | 49 | $3,324 | $491 | PASS |
| silver ↓54 | 0.0750 | 266 | 21 | $4,624 | $282 | PASS |
| silver ↓52 | 0.0335 | 138 | 33 | $1,715 | $220 | PASS |

Dropped, with reasons: WTI ↑105 / ↓70 and gold ↑4400 on the 3c mid floor; **silver ↑64
(0.140/0.200, 6c spread) and ↑66 (0.027/0.055, 2.8c on a 4.1c mid)** on the relative
spread gate; the whole sub-3c wing on both floors. Zero rows on August (no book) and zero
on the five week-of-Jul-27 boards.

`results/proposed-rows-2026-07-27.csv`, run_id `2026-07-27/daily`, model `claude-opus-5`.

**Caveat the CEO should price in:** these q values come from the pricer as changed *today*.
The change is material on this board — WTI ↓75 would have been 0.177 under yesterday's
code and is 0.100 under today's; ↑95 0.129 → 0.064; ↓80 0.571 → 0.491. Every one of those
moves is downward, i.e. toward "the touch is less likely than the market thinks", on a day
when the market has just been proved right against us. I believe the change is correct
(§4.2) and I am flagging the direction rather than burying it.

## Reproducing

```sh
cd strategies/barrier-touch/ladder-rv && cargo build --release
./target/release/ladderrv selftest data     # jump pricer identities
./target/release/ladderrv gaps data         # the close-to-open gap table
./target/release/ladderrv live data what-price-will-wti-hit-in-july-2026,...
```

The leg-sum / null tables come from `out/gate0.csv`, `out/gate1_open.csv`,
`out/gate2_checkpoints.csv` and `clob60/` inside `data/backtest-metals-2026-07-25.tar.gz`.
