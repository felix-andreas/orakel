# PRE-REGISTERED: RV-primary vs IV-primary vs a fixed blend

**Written 2026-07-28 ~01:30Z, three days before the boards it will be scored on resolve
(2026-07-31 21:00Z). Model: claude-opus-5, effort xhigh.**

> This document exists so that the answer cannot be chosen after the fact. Everything
> below — the pricers, the row set, the metric, the thresholds, the veto, and the
> "do nothing" branch — is fixed **now**, while the outcome is unknown. **The live pricer
> is NOT changed.** `cmd_live`'s trade signal still reads `q_rv` and only `q_rv`.

## Why this is a comparison and not a switch

Day-5 flagged the RV/IV question as the method's weakest link: on the leg we lost
(`will-wti-dip-to-85-in-july-2026`) the OVX-anchored number was closer than the realized-vol
one (0.5156 vs 0.3928 against a market that went to 0.715). The tempting inference is
"switch to IV". It is the wrong inference, and the reason is mechanical rather than
statistical:

**The IV anchor sits above realized vol in all three assets, on every leg, today.**

| asset | σ_rv (effective, in use) | σ_iv (anchor) | legs where q_iv > q_rv |
|---|---:|---:|---:|
| WTI | 49.7% | **60.6%** (OVX) | 21 / 21 |
| gold | 20.0% | **24.1%** (GVZ) | 19 / 19 |
| silver | 40.2% | **47.6%** (VXSLV) | 22 / 22 |

A higher σ raises the touch probability of every barrier in both directions. We are
**sell-only**: a sell signal needs `q < mid − (spread + 2c)`. So raising q *mechanically
destroys sell signals and manufactures buy signals*, and buys are disabled because they
lose after fees. Measured on today's 27 fundable legs (mid ∈ [3c, 50c]):

| pricer | sell signals today |
|---|---:|
| **A — RV-primary (live)** | **4** |
| B — IV-primary | **1** |
| C — 50/50 σ blend | **3** |

A naive switch to IV would have removed three-quarters of the only trades we are allowed
to take. That is the finding to keep in view when the Brier numbers arrive on Friday: **an
IV win on calibration is not automatically a win for this variant**, because this variant
can only monetise one side.

## The three pricers

Identical in every respect except σ. All use `touch_prob_jump` with the same spot, the same
τ, and the same jump term; all read the same frozen candle/vol archive.

- **A — RV-primary.** σ = `bump(realized_vol_intraday(14d))`. The live pricer.
- **B — IV-primary.** σ = `bump(IV anchor)` — OVX (WTI), GVZ (gold), VXSLV (silver).
- **C — blend.** σ = `0.5·σ_A + 0.5·σ_B`. **w = 0.5, fixed now, never tuned.** A weight
  fitted after seeing Friday's outcome would make this whole document worthless.

**One fairness fix made today, before any outcome was seen:** `q_iv` previously used the
raw IV while `q_rv` used the gap-bumped σ, so A carried in-window session-break variance
and B did not. B now gets the same `bump`. The effect is small (≈0.2 vol points) and it
does not change any signal count above, but a comparison whose two arms treat variance
differently is not a comparison. Recorded here because it is a change to B made by the
person who will score B.

## What will be scored, and on what

**Row set.** Every ladder-rv leg outstanding on the WTI, gold and silver **July monthlies**
and **week-of-Jul-27 weeklies**, all of which resolve at 2026-07-31 21:00Z. A, B and C are
recorded per leg by `cmd_live` in `data/out/predictions_<date>.csv` (columns `probability`,
`q_iv`, `q_blend`, plus `sigma_rv` / `sigma_iv`), so no re-derivation is needed on Friday —
the numbers are already frozen in the daily archive.

**Metric.** Paired Brier against the realised outcome, model minus market midpoint, at the
**daily 12:00Z in-window checkpoint** — not window-open, which does not survive the leg-sum
gate for gold. Reported per asset and pooled, each with its `Σmid / Σwinner` count beside
it, and each with its fillable count (`midpoint-is-not-a-fill`).

**Power floor.** The comparison decides nothing below **n ≥ 30 resolved legs**. Below that
it is reported as underpowered and the live pricer stands.

## The decision rule, stated before the outcome

A change to the live pricer requires **all four** of the following. Any one failing means
no change.

1. **Margin.** C (or B) beats A on paired Brier by **≥ 0.005 absolute**, pooled.
2. **Consistency.** The sign of that improvement holds **separately in WTI and in gold**.
   Silver is prediction-only and underpowered; it is reported but **does not vote**.
3. **Tradeability veto.** The winning pricer must not reduce the count of sell signals in
   the fundable 3–50c zone. *A calibration win paid for with buy-side signals is not a
   promotion case for a sell-only variant.* Today's baseline is A=4, B=1, C=3, and on that
   evidence B is already vetoed before Friday — which is the point of writing it down now.
4. **Power.** n ≥ 30.

**If 1–2 pass and 3 fails**, the recorded conclusion is: *"IV/blend is better calibrated
than RV, and is not usable by a sell-only variant."* That is a real result, it goes in the
wiki, and it changes nothing in `cmd_live`. It would also be an argument for revisiting the
buy-side ban on a class-gated basis — a **separate** pre-registration, not this one.

**If everything passes**, the change is proposed to the CEO after the 08-02 trial review,
with the `q*` / `q` / `q⁻` table required by `break-even-win-rate.md`. **Not before.**

## What I expect (recorded so it can be wrong)

RV-primary is currently the *low* vol estimate in all three assets, and the market sits
between RV and IV on most fundable legs. My expectation is therefore that **B and C beat A
on Brier** (the market agrees with them more) **while both fail the tradeability veto** —
i.e. outcome "1–2 pass, 3 fails". If instead A wins outright on Brier, that is evidence the
14-day realized-vol anchor is fine and the ↓85 loss really was purely the stale feed rather
than a vol-anchor problem.

The one thing that would genuinely surprise me is C beating A on Brier **and** producing at
least as many sell signals. That cannot happen by the mechanism above unless the market
mids move a long way toward the model between now and Friday.
