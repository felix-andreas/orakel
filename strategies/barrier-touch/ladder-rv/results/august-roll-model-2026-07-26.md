# Pricing a WTI ladder that spans a CME roll — and why we still predict nothing on August

**Date:** 2026-07-26 · **Run by:** slot-1 researcher · **Model:** claude-opus-5 (effort xhigh)
**Code:** `ladderrv roll` / `ladderrv selftest` (`src/main.rs`, added today)

> In plain English: the August oil board asks whether "oil" touches a price, but the
> thing it actually measures swaps from one futures contract to a cheaper one halfway
> through the month. Today that swap is worth $4.58 a barrel — 5% — and it happens on a
> date everybody knows in advance. Our current model has no idea, and it is wrong in the
> most dangerous possible direction: it makes the downside bets look 40% cheaper than
> they are, which is exactly the trade we would have put on.

## 1. What the board actually resolves on

From `will-wti-reach-140-in-august-2026`'s own description (all August legs share it):

> …any 1-minute candle for the **Active Month** of WTI Crude Oil futures… The active
> month changes at the start of the second trading session prior to the nearest listed
> contract's last trading session… a contract's last trading day is three business days
> prior to the 25th calendar day of the month preceding the contract's delivery month
> (or four business days prior if the 25th calendar day is not a business day).

Applied:

| | |
|---|---|
| CLU6 (Sep delivery) LTD | 25 Aug 2026 is a **Tuesday** → 3 business days prior → **Thu 20 Aug** |
| Independent confirmation | Pyth names the feed "PYTH WTI **20 AUGUST 2026** / US DOLLAR" |
| Final three CLU6 sessions | Tue 18, Wed 19, Thu 20 Aug |
| ⇒ **CLV6 becomes the active month** | at the start of the session for **Tue 18 Aug** = **2026-08-17 22:00Z** |
| CLV6 (Oct delivery) LTD | 25 Sep is a Friday → **Tue 22 Sep** (Pyth: "22 SEPTEMBER 2026") ✓ |

So the August monthly board (window 2026-08-02 22:00Z → 2026-08-31 21:00Z, 21 sessions)
resolves on **CLU6 for its first 11 sessions (Aug 3–17) and CLV6 for its last 10
(Aug 18–31)**. Both feeds exist on Pyth today and both are in our archive.

### Correction to a previously recorded fact

`STRATEGY.md` and `memory/MEMORY.md` say *"CLU6 is the active month for every session
from Jul 1 through Aug 17 … The July monthly and the week-of-Jul-27 board therefore
resolve on CLU6 only, no roll."* **The second half of that is wrong.** Apply the same
rule to CLQ6 (Aug delivery): 25 Jul 2026 is a **Saturday**, so LTD = 4 business days
prior = **Tue 21 Jul**, and CLU6 became active at the start of the session for **Fri 17
Jul** (2026-07-16 22:00Z). This is literally the worked example in the market's own fine
print. Therefore:

- **The July monthly board also spans a roll** (CLQ6 → CLU6 at the Jul 17 session). Our
  gate-0 mirror used WTIU6 for the whole month, i.e. the wrong contract for Jul 1–16.
- Same for the WTI **weekly** week-of-Jul-13 board, whose "28/28 gate-0 from our own
  WTIU6 archive" covered four CLQ6 sessions.
- **It changed no answer** — see §5 — but it was luck, not method.
- `Commodities.WTIQ6/USD` is **already delisted** (`s=error`), so that half of July can
  never be reconstructed. Pyth carries exactly two contracts; **WTIX6 must be archived
  the day it appears** (~Aug 20), because the September monthly spans the CLV6 → CLX6
  roll at the session for Fri 18 Sep.

## 2. The model

Take the **deferred** contract as the primitive — it is the one that survives to the end
of the board. Let `V` be CLV6, driftless GBM in session time with vol `σ_v`. The front
contract is tied to it by the log calendar spread `k = ln(U/V)`, and that spread is not
constant: backwardation steepens when crude rallies. Model it affinely,

```
ln U(t) = ln V(t) + k0 + β·( ln V(t) − ln V0 )
```

so a barrier `B` quoted on the active-month series maps onto the V scale as

```
B_front = V0 · (B / U0)^(1/(1+β))
```

and the whole board becomes **one diffusion with a barrier that steps at a known instant**:
`B_front` while CLU6 is active, `B` afterwards. For an ↑ leg `B_front < B`, so the
pre-roll half is the easy half; for a ↓ leg `B_front < B` means the *post*-roll half is
the easy half, and any path sitting between the two levels at 2026-08-17 22:00Z is
absorbed **at the roll** — the resolving series jumps down onto the barrier. That atom is
the entire story of the August downside.

### Calibrating β and σ

CLU6/CLV6 daily closes, 2026-06-16 (first CLV6 print on Pyth) → 2026-07-24, n = 33 daily
log-returns:

| quantity | value |
|---|---|
| regression `Δ(lnU − lnV)` on `Δ lnV` | **β = 0.158** |
| daily vol ratio σ_U/σ_V | 1.168 → β = 0.168 |
| 14d realized (5-min closes, session-annualised) | σ_U **47.0%**, σ_V **41.7%** → ratio 1.127, β = 0.127 |
| corr(Δ lnU, Δ lnV) | **0.987** — the single-driver approximation is tight |

Three estimators land in 0.13–0.17; **β = 0.15** is the base case, 0.00 and 0.30 the
sensitivities. `k0 = ln(90.46/85.877) = 0.05199`, i.e. the spread is **+$4.58 (5.07%)**.
For scale, that spread was **+$0.04 on 1 Jul** and **+$5.51 on 23 Jul** — it tracked the
67 → 93 crude spike and its own daily changes have sd ≈ $0.85, so over the 16 sessions to
the roll its sd is ≈ **$3.4**. We cannot forecast it; we can only test whether the answer
survives it.

### Numerics

Absorbing heat kernel by images on a uniform log grid (`ROLL_DX = 0.0004`),
`p(y|x,τ) = φ(y−x) − φ(y+x−2b)`, composed over three phases: free diffusion to the window
open (5 sessions, τ = 0.019841), absorbing at `B_front` (11 sessions, τ = 0.043651),
kill-at-the-roll, absorbing at `B` (10 sessions, τ = 0.039683). `ladderrv selftest`
checks it against `2N(−|ln(B/S)|/(σ√τ))` and against splitting one phase in two:

```
  H105: grid 0.73004 closed 0.73010 diff -0.000052
  H110: grid 0.50084 closed 0.50035 diff +0.000496
  H130: grid 0.06353 closed 0.06357 diff -0.000036
  L95:  grid 0.71732 closed 0.71683 diff +0.000493
  L70:  grid 0.01164 closed 0.01167 diff -0.000029
  split 0.04+0.04 vs one 0.08: 0.19713 vs 0.19733 diff -0.000193
```

One trap worth recording: the grid **must extend to `2b − lo`** or the reflected image
source falls off it and every touch probability comes out *exactly half* the right
answer — a failure that looks plausible rather than broken.

## 3. What the roll is worth

U0 = 90.46, V0 = 85.877 (both 2026-07-24 21:00Z), σ_v = 41.7%, β = 0.15.
"naive" = what `ladderrv live` does today: one spot (CLU6), one σ, no jump.

| leg | mkt mid | naive | **roll-aware** | β=0 | β=0.30 | spread→0 | σ from OVX 68 |
|---|---:|---:|---:|---:|---:|---:|---:|
| ↓50 | 0.025 | 0.0000 | **0.0001** | 0.0001 | 0.0001 | 0.0000 | 0.0045 |
| ↓60 | 0.044 | 0.0025 | **0.0074** | 0.0073 | 0.0084 | 0.0025 | 0.0591 |
| ↓65 | 0.490 | 0.0148 | **0.0370** | 0.0364 | 0.0407 | 0.0158 | 0.1399 |
| ↓70 | 0.470 | 0.0588 | **0.1213** | 0.1185 | 0.1301 | 0.0649 | 0.2675 |
| ↓75 | 0.500 | 0.1672 | **0.2846** | 0.2763 | 0.2996 | 0.1844 | 0.4321 |
| ↓80 | 0.575 | 0.3651 | **0.5082** | 0.4940 | 0.5252 | 0.3892 | 0.6095 |
| ↓85 | 0.500 | 0.6463 | **0.7328** | 0.7217 | 0.7430 | 0.6377 | 0.7657 |
| ↑90 | 0.740 | **1.0000** | **0.8431** | 0.8443 | 0.8407 | 0.8690 | 0.8442 |
| ↑95 | 0.500 | 0.7182 | **0.6431** | 0.6163 | 0.6635 | 0.6909 | 0.7119 |
| ↑100 | 0.505 | 0.4599 | **0.4215** | 0.3722 | 0.4634 | 0.4806 | 0.5603 |
| ↑105 | 0.500 | 0.2720 | **0.2411** | 0.1931 | 0.2884 | 0.2962 | 0.4133 |
| ↑110 | 0.510 | 0.1495 | **0.1231** | 0.0900 | 0.1631 | 0.1652 | 0.2889 |
| ↑115 | 0.500 | 0.0769 | **0.0579** | 0.0385 | 0.0859 | 0.0849 | 0.1929 |
| ↑120 | 0.400 | 0.0373 | **0.0250** | 0.0155 | 0.0422 | 0.0407 | 0.1244 |
| ↑130 | 0.115 | 0.0075 | **0.0039** | 0.0022 | 0.0087 | 0.0078 | 0.0473 |
| ↑140 | 0.0345 | 0.0013 | **0.0005** | 0.0003 | 0.0015 | 0.0013 | 0.0164 |
| ↑150 | 0.028 | 0.0002 | **0.0001** | 0.0000 | 0.0002 | 0.0002 | 0.0053 |

Read the ↓ column first. **The naive model under-prices every downside leg by 40–110%
in relative terms** — ↓75 by 11.7pp (0.167 → 0.285), ↓80 by 14.3pp (0.365 → 0.508), ↓70
by 6.3pp. Those are sell-side legs quoted between 4c and 58c: a naive run would have
flagged them as fat, overpriced wings and sold them, and it would have been wrong by
roughly double. On the ↑ side the naive model is too *generous*: ↑90 reads a certainty
(spot 90.46 ≥ 90) where the honest answer is 0.843, because CLU6 has five sessions to
fall under 90 before the window even opens and the post-roll series starts $4.58 lower.

The naive model also has two defects that are nothing to do with the roll, both found by
reading the code today:

1. **`SessionCal::build` stopped at 2026-08-20.** Any board running past that date had
   its post-Aug-20 session minutes silently dropped: the August board would have been
   priced on **14 of its 21 sessions**, making σ√τ **18% too small** (√(14/21) = 0.816).
   Fixed today (calendar now runs to 2026-10-31; Labor Day added; Columbus Day
   deliberately not — CME energy and NYSE both trade it).
2. **`cmd_live` starts the diffusion at *today's* spot for a window that opens later.**
   τ is correctly counted from the window start (`now.max(l.ws)`), but the level is not
   diffused over the 5 sessions in between (σ√τ = 5.9% of spot), so every leg on a
   not-yet-started board is under-priced. Not fixed yet — `roll` handles it via the
   `tau_pre` free-diffusion phase; `live` still needs it.

## 4. Verdict on August: model yes, predictions no

The model exists and is validated. We still emit **zero August rows**, for a reason that
has nothing to do with the model:

- **14 of the 20 legs have no book.** Spreads of 46c–98c (↓65 quotes 0.03/0.95, ↑95
  quotes 0.01/0.99). Per `wiki/reference/phantom-midpoints.md` those 0.50 "midpoints" are
  the absence of a price. They fail the book gate outright.
- **The legs where the roll actually bites are exactly the unquoted ones.** Every leg
  whose naive-vs-roll gap exceeds 2pp (↓65…↓85, ↑90…↑115) has a spread ≥ 46c. The
  differential claim this model makes is, today, untradeable and unrecordable.
- **The 6 legs that pass the 5c spread gate are wings where the roll is worth nothing.**
  ↓20/↓30/↓40/↓50 and ↑150 all quote under 3c — outside the fundable band and precisely
  the category the executable-price audit demolished. The only gate-passing leg inside
  the fundable band is **↑140 at a 3.45c mid on a 2.0/4.9 book** — and its own spread
  (2.9c) plus the method's 2c buffer exceeds the mid, so the sell rule
  `q < mid − (spread + 2c)` cannot fire at any q ≥ 0. Its top-of-book bid is $10.
- **The 5c spread gate is vacuous below a ~5c mid.** ↓20 quotes 0.003/0.019 — a 1.6c
  spread that "passes", on a book whose midpoint is **3.8× its bid**. The gate must
  become relative (e.g. `spread ≤ min(5c, ½·mid)`), otherwise it waves through exactly
  the wings that the fill audit showed are unsellable. Recorded as a method change.
- **August resolves 2026-08-31 — after the 2026-08-02 trial review.** No August row can
  contribute to the promotion case even if it were fillable.
- The gold and silver August boards (no roll — continuous Pyth spot) are worse: **all 28
  legs fail the 5c gate**, 24 of them quoting 0.01/0.99 or 0.02/0.98.

So: August is now *priceable* and is not *predictable*. Re-read the book when the crowd
arrives — the near-money ↓ legs are where our new model has something to say, and it says
the opposite of what the old one would have.

## 5. Does the July roll break the 07-31 scoring?

No, and here is why rather than an assertion. The July board's first half resolved on
CLQ6, which we cannot reconstruct. But over the CLQ6 sessions (Jul 1–16) CLU6 ranged
**67.12 – 80.53**, and CLQ6 sat within about +$1.00 of that (the U6−V6 spread over the
same window was +$0.04 … +$1.20, and Q6−U6 is the same kind of one-month spread). So the
CLQ6 half of July spanned roughly **67.2 – 81.5**, against live barriers of ≤65 on the
downside and ≥95 on the upside. Nothing is close. Every live July leg's touch/no-touch is
decided inside the CLU6 half, which we do hold, minute by minute, in our own archive.

## Reproducing

```sh
cd strategies/barrier-touch/ladder-rv && cargo build --release
./target/release/ladderrv selftest data
./target/release/ladderrv roll data what-price-will-wti-hit-in-august-2026 90.46 85.877 0.417 0.15
```
