# 08-02 as a sizing question: the break-even bound and the correlation of what we'd hold

**2026-07-30 (day 8), model claude-opus-5 effort xhigh.**

Framing set by `ops/decisions.md` (2026-07-29): the "structurally short downside touch"
hypothesis is **refuted** on 633 resolved legs, the model is well calibrated on the evidence
we have, and what is real is a one-sided tail whose eight worst legs are all `dip-to` legs
nested on few contracts — *"roughly one draw, not two"*. So 08-02 asks **"is that tail
acceptable at our size, given how correlated the legs we hold are?"** Not another calibration
table.

This document answers the two things that question needs: the **break-even bound** and the
**effective sample / tail exposure of a book we would actually hold concurrently**.

> **Caveat, stated up front and applying to every number below.** The 633-leg sample is the
> sample the method was gated on. These numbers are the right *shape* of answer and must not
> be read as out-of-sample. Friday's 131 rows are the out-of-sample test and are far too
> small to redo this on. Where the two disagree, neither wins on its own.

---

## 1. Our trade is a favourite-side buy, and the wiki already says what that costs

We are **sell-only**. Selling YES at mid `p` is **buying NO at cost `c = 1 − p`**. In
`break-even-win-rate`'s language:

- `q*` (break-even win rate) `= c = 1 − p` (+ fee)
- `q` = observed fraction of legs that did **not** touch
- `q⁻` = Wilson 95% lower bound on `q`
- **trade only when `q⁻ > q*`** — never `q > q*`

This reframing is the first useful thing. Our fundable band is `p ∈ [3c, 50c]`, so
`c ∈ [50c, 97c]`: **every trade this variant is allowed to make is a favourite-side buy at
50–97 cents.** Selling a 3-cent wing is buying the favourite at 97, needing to be right 97
times in 100. That is precisely the regime the wiki page calls uninvestable, and it is where
the variant's thesis ("fade the overpriced wing lottery legs") puts most of its trades.

## 2. The break-even table, per band, with the look-ahead removed

Population: the 633 resolved backtest legs, daily in-window checkpoints, `zone_excl = 0`.
One trade per leg, entered at the **first** checkpoint at which the rule fires.

> **The entry rule has to be the first qualifying checkpoint, not the last.** My first cut
> took the last qualifying checkpoint and produced a −66% RoLC in the 20–35c band. That was
> look-ahead, not a result: for a leg that eventually touches, the mid **rises toward 1** as
> the touch becomes likely, so "the last checkpoint still priced under 50c" selects the
> moment just before the loss. Same error family as
> `lifetime-volume-is-look-ahead`. Corrected below.

**Sell signal fires (`q_rv < mid − 2c`; the live rule also needs `− spread`, unavailable in
this file, so this is marginally generous):**

| band (sell price `p`) | n | `q*` = c | `q` | `q⁻` | verdict | EV/trade | RoLC |
|---|---:|---:|---:|---:|---|---:|---:|
| [0.03, 0.10) | 166 | 0.948 | 0.988 | **0.957** | **clears** (+0.9pp) | +0.040 | +4.2% |
| [0.10, 0.20) | 62 | 0.857 | 0.839 | 0.728 | fails | −0.019 | −2.2% |
| [0.20, 0.35) | 63 | 0.731 | 0.730 | 0.610 | fails | −0.001 | −0.2% |
| [0.35, 0.50) | 65 | 0.552 | 0.723 | **0.604** | **clears** (+5.2pp) | +0.171 | +30.9% |
| **pooled 3–50c** | **356** | **0.822** | **0.868** | **0.829** | **clears (+0.7pp)** | +0.046 | +5.6% |

For contrast, without the signal filter (every fundable leg): pooled `q⁻` 0.764 vs `q*` 0.789
— **fails**. So the model's disagreement filter is doing real work; that is a genuine result
in its favour and the mean-edge story from the backtest survives.

**Two things this table says that a Brier number cannot:**

1. **The edge lives at both ends and not in the middle.** 10–35c has no edge at all
   (EV −0.019 and −0.001). Pooling hides it, which is exactly the wiki's rule 1.
2. **The pooled bound clears by 0.7 percentage points.** Under one point of margin on the
   *lower bound* is not a promotion case at size; it is a coin-flip about whether the sample
   is big enough.

## 3. The number that decides it: effective n is 173, not 356

A one-touch **down**-ladder on one underlying is a set of legs that are **monotone functions
of a single random variable** — the running minimum of that underlying over that window. If
the minimum reaches 80, every leg with a barrier above 80 has already lost. There is no
diversification between them at all; there is one draw with a staircase payoff.

Measured on the 356 sell-signal legs:

| | |
|---|---:|
| legs | **356** |
| distinct boards (asset × window) | **46** |
| distinct monotone families (board × direction) | **84** |
| distinct assets | 7 |
| mean legs per family | 4.24 (max 12) |
| **intraclass correlation of the loss indicator within a family** | **ρ = 0.325** |
| design effect `1 + (k̄ − 1)·ρ` | **2.05** |
| **effective n** | **173** |

Re-running the bound at the effective n:

| | n | `q*` | `q` | `q⁻` | verdict |
|---|---:|---:|---:|---:|---|
| nominal | 356 | 0.822 | 0.868 | 0.829 | **clears** (+0.73pp) |
| **effective** | **173** | 0.822 | 0.868 | **0.808** | **FAILS (−1.32pp)** |

**This is the answer to the CEO's question.** The pooled sell-side edge clears its break-even
bound on the sample size we *appear* to have and fails it on the sample size we *actually*
have. The two losses being "~1 draw, not 2" is not a curiosity about those two legs — it is
the general structure of the book, quantified: **ρ = 0.325 across 84 families cuts our
evidence in half.**

It also explains the day-7 finding without any new mechanism: the 8 worst legs of 633 sharing
a direction is what a ρ of 0.325 on down-families *looks* like.

## 4. Tail exposure at size: a cliff, not a tail

The outstanding book as Friday will hold it (131 rows incl. today's 5, 1 unit sold per row),
grouped into correlated families:

| family (asset, direction, window) | legs | premium collected | loss if that extreme is fully hit |
|---|---:|---:|---:|
| wti, UP, in-july | 34 | 5.29 | 28.71 |
| wti, DOWN, in-july | 21 | 1.15 | 19.85 |
| xagusd, DOWN, in-july | 15 | 1.46 | 13.54 |
| xauusd, DOWN, in-july | 13 | 1.07 | 11.93 |
| bitcoin, DOWN, in-july | 9 | 0.78 | 8.22 |
| **whole book** | **131** | **14.50** | **116.50 (8.0×)** |

The worst **single** family's full loss, 28.71, is **1.98× the premium collected on the
entire book**. Twelve families is not twelve bets.

But "fully hit" is the extreme corner, so here is the honest version — the **staircase**, for
the WTI down-ladder, against the move the trial actually produced (CLU6 90.46 Fri 07-24 →
83.68 Sunday open → **77.80** low Tuesday, −14.0%):

| if the running minimum reaches | rows lost | loss | net vs the 1.15 premium |
|---:|---:|---:|---:|
| 85 | 0 | 0.00 | +1.15 |
| 80 | 1 | 0.66 | +0.49 |
| **77.80 — what actually happened** | **1** | **0.66** | **+0.49** |
| 75 | 8 | 6.96 | **−5.81** |
| 70 | 11 | 9.90 | −8.75 |
| 65 | 14 | 12.88 | −11.73 |
| 55 | 20 | 18.85 | −17.70 |

**The marginal cost of the next 5% down was +6.30 — 548% of that family's entire premium.**
A −14% move left the family net positive. A −18% move takes it to −5.8× its premium. The
premium is not distributed along the ladder: **90% of it sits on the two barriers nearest
spot** (↓75 carries 0.695 across 7 rows, ↓80 carries 0.345 on 1), and those are the first two
a continuing move removes. The deep wings ↓45–↓65 contribute **0.054 of premium — 4.7%** —
while adding **5.92 of loss exposure**.

So the wing-lottery thesis is exactly right in the mean and exactly wrong in the tail: those
legs are nearly free to sell **and nearly free to lose**. They pay ~1 cent each and are
carried away by the same move that takes the legs we actually get paid for.

## 5. What this recommends for 08-02 — and what it does not

**It does not say the variant is bad.** The mean edge is real, the model beats the market on
touched legs, and the sell-signal filter clears where the unfiltered population fails. All of
that stands.

What it says is that the promotion question has a specific shape:

1. **Do not promote on the pooled number.** It clears by 0.7pp nominal and fails at effective
   n. Any promotion case must quote the bound **at effective n**, with ρ stated.
2. **Size by family, not by leg.** The unit of risk is (asset, direction, window), and we hold
   ~4 legs per unit. A per-leg limit lets one contract's running minimum carry 12 positions.
3. **The 10–35c band has no measured edge.** It is a candidate for exclusion — but that is a
   *new* pre-registration, decided before seeing Friday, not a filter fitted now.
4. **The deep wings are the cheap part of the tail.** Selling ↓45 for 0.2 cents to add 1.0 of
   loss exposure is not a trade; it is a lottery ticket sold to us. Worth a minimum-premium
   floor per leg — again, pre-registered, not fitted.
5. **Buys stay off.** Nothing here touches that.

## 6. What is missing, and would need its own day

Scoped honestly rather than half-done:

- **ρ is estimated on the gated sample.** Friday's 131 rows across 12 families cannot
  re-estimate it (12 clusters is too few for a stable ICC). The right fix is more *boards*,
  not more days — and the board universe is exhausted until the August cohort prices ~08-10.
- **ρ across families is not measured.** I treated the 84 families as independent. They are
  not: WTI-DOWN-july and WTI-DOWN-week-of-jul-27 share an underlying, and the whole book
  shares a macro factor. **The true effective n is below 173, so the failure at effective n is
  a lower bound on the problem, not an upper one.** Measuring the between-family correlation
  needs a common-factor decomposition across assets and is a day of work.
- **Fees and the real fill are not in `q*`.** `q* = c` ignores fees and assumes the scored
  midpoint is reachable. `midpoint-is-not-a-fill` puts reachability at 82–99% for the
  commodity monthlies and **38%** for equity weeklies, which pushes `q*` up and every verdict
  above toward "fails". Folding `fills.csv` into `q*` per board family is the single highest
  value follow-up and should happen with Friday's regenerated fills.
- **No capital model.** RoLC per trade is here; a Kelly-style or drawdown-constrained size
  given ρ = 0.325 and the staircase in §4 is not. That is the actual "at our size" answer and
  it needs a stated bankroll, which the firm has not set.
