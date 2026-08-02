# Decision log

Append-only. Every structural change to the firm gets a dated entry: **what changed, why,
who decided**. Newest first.

---

## 2026-08-02 (late) — Felix paused the daily trigger. The firm is idle, not stopped.

"Stop your routine trigger for now." Done: `trig_017vXv9HCCiTZXVUd3brFuD9` is **disabled**,
not deleted. The cron, the prompt and the self-bind session all survive, so re-enabling is one
call and the CEO keeps its accumulated context rather than starting cold.

**What stops.** Everything the daily run does: the market researcher, slot work, the resolution
sweep and scoring, the watchlist mirror at run start and close, health checks, book compaction,
run manifests.

**What does not stop.** The **snapshot worker** is a Cloudflare cron with no LLM in it and is
untouched — it keeps writing hourly book snapshots to R2. That is deliberate and worth stating,
because book history is the one dataset the firm cannot reconstruct after the fact: a market's
book at a past instant is gone if nobody recorded it. Leaving it running costs nothing and
preserves the option to resume with continuous data. The dashboard also stays up; it reads the
repo at request time, so it will simply show a firm whose last run was 08-02.

**What is left mid-flight, so resuming is not guesswork:**

1. **The trial review is unrun and due 08-03.** It slipped once this morning by the
   completeness gate. The form, the pre-registered rule, the four gates and the independent
   reviewer's brief are all written; the only outstanding input is
   `will-wti-reach-90-in-july-2026-from-july-27` (4 rows), still in UMA dispute. On resumption
   the 08-01 rule applies directly: still disputed ⇒ excluded, named, re-scored later, and only
   if the reviewer verifies the verdict does not turn on it.
2. **`barrier-touch/ladder-rv` stays `trial`.** Not promoted, not discarded. Its evidence is
   frozen at 163 rows over 82 markets, per market −0.0025, CI [−0.0078, +0.0027].
3. **The watchlist keeps 4 rows' worth of markets** it would otherwise drop once that dispute
   settles. Harmless; the tool is idempotent and will drop them on the next run.
4. Three open Felix items and the backlog (`ops/backlog.md`) are unchanged.

**A caution for whenever this resumes**, recorded now rather than rediscovered: the
pre-registrations do not expire. The decision rule, the completeness gate, the disputed-market
exception and the independent-evaluator rule were all written before the numbers existed, and a
pause is not a reason to revisit any of them. If anything, a gap makes them more valuable —
they are what lets a resumed review be the same review.

Recorded by the CEO (claude-opus-5).

---

## 2026-08-02 — The completeness gate fires, and the review slips to 08-03

The trial review was due today. It does not happen today.

`will-wti-reach-90-in-july-2026-from-july-27` is still **`umaResolutionStatus: disputed`**,
carrying 4 rows. Everything else settled: the 15 BTC legs resolved overnight, and the scored
evidence is now **163 rows over 82 markets, per market −0.0025, CI [−0.0078, +0.0027]**.

The gate, pre-registered on 07-30, says the review proceeds only when every outstanding row's
market is in `resolutions.csv`, else it slips **one day, once**. Today is the first review
date. So: **08-03**, and the 08-01 rule then applies — if the market is *still* disputed
tomorrow, it is excluded and the review proceeds, provided the reviewer verifies the verdict
does not turn on it.

**The uncomfortable part, said out loud.** 4 rows on 1 market out of 82 markets will almost
certainly not move any gate. I can see that, and I could argue it convincingly. That is
precisely the situation the gate was written for: a rule that only binds when it costs nothing
is not a rule, and "it obviously doesn't matter" is the argument I would also make if it did.
The cost of honouring it is one day. The cost of stepping over it is that every future
pre-registration in this firm becomes advisory.

I also note, because it will be true tomorrow too: I still do not know whether excluding those
4 rows helps or hurts. The market quotes ~0 for Yes and we are sell-touch, so I can guess. I
have not checked, and the reviewer will not be told to.

No other change. Slot 1 does not predict — its boards have settled and its trial is under
review; the market researcher runs as usual.

Recorded by the CEO (claude-opus-5).

---

## 2026-08-01 (late) — What to do if a UMA dispute blocks the review, decided while it is still hypothetical

16 markets / 19 rows are outstanding. Fifteen are BTC legs ending 03:59:59Z tonight, which will
be settled well before tomorrow's run. The sixteenth is not:

> `will-wti-reach-90-in-july-2026-from-july-27` — **`umaResolutionStatus: disputed`**, 4 rows,
> quoting 0.0005 / 0.9995.

A disputed market goes to UMA's DVM vote, which routinely takes days rather than hours. So it
may well be unresolved tomorrow, and possibly on Monday too — and the pre-registered
completeness gate is absolute: the review proceeds only when **every** outstanding row's market
is in `resolutions.csv`, else it slips **one day, once**, and a second slip must be argued here.

I am writing that argument now, before I need it, because in two days I will know things that
should not be allowed to shape it.

**The rule: a market still in UMA dispute at the second review date is EXCLUDED, and the review
proceeds.** Conditions, all of them binding:

1. **Name it in the review** — the market, its row count, our predictions, and the fact that it
   was excluded for dispute rather than for its content.
2. **The exclusion must be immaterial to the verdict, and that must be checked rather than
   asserted.** With 68 markets, one market is ~1.5% of the per-market sample. If the reviewer
   finds the verdict *would* flip on it, the review does not proceed — it waits, however long
   the DVM takes, because a decision that turns on a market nobody can score yet is not a
   decision.
3. **It gets appended and re-scored when it settles**, whatever the review already concluded.
   The record catches up even though the decision did not wait for it. If the verdict would have
   differed, that becomes its own entry.

**Why exclusion rather than indefinite delay.** The gate exists so a review is not read off a
half-settled batch. A single market held hostage by a third-party dispute process is a different
problem: waiting on it does not make the other 67 markets better evidence, it just moves the
decision to a date chosen by UMA voters.

**What I have deliberately not looked at:** whether excluding these 4 rows helps or hurts the
variant. The market quotes ~0 for Yes and we are sell-touch, so the direction is guessable, and
that is precisely why the rule is written now rather than on Monday with the number in front of
me. If it turns out the exclusion favours us, condition 2 catches it; if it disfavours us, the
rule still stands.

Recorded by the CEO (claude-opus-5), while the dispute is unresolved.

---

## 2026-08-01 — The July batch settles, the projection lands on its named alternative, and slot 1 does not run

**The settlement.** 44 markets resolved, **43 of them NO** — the side a sell-touch variant is
on. Scored evidence went 35 rows / 23 markets → **148 rows / 67 markets**.

| | 07-31 | 08-01 |
|---|---|---|
| per market | −0.0094 | **−0.0034** |
| 95% CI | [−0.0280, +0.0092] | **[−0.0097, +0.0030]** |
| tradeability | 15/35 (43%) | **93/148 (63%)** |
| mean `exec_edge` | +0.4647 | +0.2098 |

**The projection I recorded on 07-30 was testable and it resolved.** I wrote: 21→~90 markets
narrows the interval to about ±0.010; today's point estimate sits outside that; *so if the mean
holds, Friday excludes zero on the negative side and the rule says discard — and if the mean
moves toward zero, which is what an unlucky-draw tail would look like, it stays inconclusive.*

The interval came in at **±0.0064**, tighter than I guessed. The mean moved toward zero. It
landed on the branch I named as the alternative, and the interval still contains zero. Naming
both branches in advance is the only reason that sentence is worth anything today.

**I am not calling gates from this.** Gate 2 now looks comfortable and gate 1 still straddles,
and it would be easy to write both down as settled. They go to tomorrow's independent reviewer
like everything else, because a rule I apply selectively when the answer looks obvious is not a
rule.

**Slot 1 does not run today, and that is a decision rather than an omission.** Its trial is
reviewed tomorrow; the July boards it predicts on have settled; the stale-feed gate suppresses
WTI, gold and silver on weekend runs anyway; and any row emitted now would land on a market
already predicted many times, adding rows and not markets, on the eve of a per-market judgement.
There is no version of today where slot 1 emitting changes what tomorrow can conclude, and one
where it looks like padding the sample the night before the verdict. The predicting phase of
this trial is over.

The market researcher **does** run. If ladder-rv is discarded tomorrow the firm has zero live
strategies and a backlog of one blocked idea, so the pipeline is the thing that matters now —
and the answer to that is more objects worked, not a lower bar.

Recorded by the CEO (claude-opus-5).

---

## 2026-07-31 (late) — The independence rule gets tested on its first day, and holds

Slot 1 closed its sizing work this evening and reported, in its own words, that **"08-02 is
decidable on the sizing doc — gate 3 fails at the executable price on nominal n alone."**

The analysis behind it is strong and it is the *decisive* number: the nominal margin is
**+0.73pp** and the median half-spread on the 65 gate-passing legs is **1.00c**, so selling at
the bid gives `q* 0.8316` against `q⁻ 0.8289` — **a failure of 0.27pp at nominal n, at zero
fee, before any correlation argument is made at all.** Break-even half-spread is 0.73c against a
median book 2.0c wide. Separately, between-family ρ is now measured and `n_eff ∈ [118, 173]`,
failing across the whole range, and Kelly at the 95% lower bound is negative (−6.8% / −14.9%),
which answers "at our size" without needing the bankroll figure Felix has not set.

**I am not recording that as a verdict, and the reason matters more than this trial.**

This morning I ruled that slot 1 supplies analysis and never verdicts. Twelve hours later slot 1
handed me a verdict — and it is a verdict *against its own variant*, which makes it credible and
makes accepting it feel free. That is exactly the shape of the test.

If I accept an unfavourable verdict from the researcher today, I have established that the
researcher gives verdicts, and the next one may not be unfavourable. **The asymmetry is the
bias**: taking a failing number at face value while I would have sent a passing one to review is
not caution, it is the same discretion pointed in a direction I happen to like. So the sizing
document goes to the independent reviewer with everything else, and the reviewer grades gate 3.

I expect it to agree. That is not the point.

One thing does change: the reviewer must be told that **gate 3 may be decidable without Friday's
resolutions at all**, since an edge smaller than the spread does not depend on how the boards
settled. If so, that is worth saying plainly — it would mean the trial's outcome was determined
by the book rather than by the forecast, and nine days of prediction were measuring the wrong
question.

Recorded by the CEO (claude-opus-5).

---

## 2026-07-31 — The trial review is evaluated by someone who is not me

Written on the last predicting day, before a single July board had closed.

The 08-02 rule is pre-registered and gate 1's interval is now computed by the tool rather than
by a person. Both were about removing discretion from the moment of judgement. There is one
piece left, and it is the largest: **I would have been the one applying the rule.**

Three things make me the wrong reader of a marginal number here, and none of them are visible
from the inside:

- I have run this trial for nine days.
- I wrote the rule.
- The firm has one trial and **zero** live strategies, so a discard leaves nothing.

I already caught myself twice this week asserting things I had not measured — the exposure
hypothesis slot 1 refuted on 633 legs, and an OVX comparison against the wrong series. Both
were caught by someone else. This is the same failure mode with more at stake.

**So the gate evaluation goes to an independent agent** that has not worked on the variant,
given the pre-registered rule, the scored data and slot 1's sizing work, and **no context about
what the firm would prefer**. It returns a verdict per gate. I then make the promote / discard /
extend call on that evaluation, and if I disagree with any verdict I record the disagreement
explicitly rather than quietly substituting a different number.

**Slot 1 may supply analysis. Slot 1 may not supply verdicts.** It has done the sizing work
honestly, including reporting the number that fails its own variant — but a researcher grading
its own trial is the arrangement, not the person, and the arrangement is what I can fix.

The form the reviewer fills is `ops/reviews/2026-08-02-ladder-rv.md`, written today with every
threshold copied verbatim and every value blank. Each gate takes `PASS` / `FAIL` /
`UNEVALUABLE`; prose in place of a verdict is itself a failure of the review. Thresholds may not
be edited in that file — changing one is a decision entry of its own, dated after the numbers
and argued in the open.

Recorded by the CEO (claude-opus-5), 2026-07-31, before any July board closed.

---

## 2026-07-30 — The 08-02 trial decision rule, written before Friday's numbers exist

I have pre-registered three secondary things for Friday and had not pre-registered the
decision the whole week is for. That is the gap that matters: on Sunday I would be choosing a
threshold with the result already in front of me, which is exactly what I have spent two runs
telling researchers not to do.

Written now, while nothing about Friday is known. **Current state, for the record:** 32 scored
rows / 21 markets, per row **−0.0452**, per market **−0.0127**, 18 of 21 markets positive and
3 negative, fillable 13/32, mean `exec_edge` +0.4989. Friday adds ~104 rows.

And stated plainly because it is the pressure the rule exists to resist: **the firm has one
trial and no live strategies.** If ladder-rv is discarded we have zero. "We would have nothing
left" must not become an argument on Sunday, so it is disqualified here, in advance.

### The unit

**Per market, always.** Rows are not independent — we predict the same market every morning,
and one barrier touch is scored once per day the market was open. The row number is reported
beside it and is never the deciding number. This is the same choice already made for the
pricer split, for the same reason.

### The four gates. Promotion needs all four

Inherited from the founding guideline (≥15 scored predictions across ≥3 markets beating the
market baseline) and extended by what the firm has since measured:

1. **Calibration.** Per-market mean paired improvement **> 0**, with a cluster-robust 95%
   interval that excludes zero. Beating the market on average is not enough if the interval
   contains it.
2. **Tradeability.** A majority of scored rows fillable (**> 50%**, currently 41%) *and* a
   positive mean `exec_edge` on the fillable subset. An edge on rows nobody would fill is a
   research result, not a business.
3. **Fundability.** The **95% lower bound** of the realised win rate above the break-even
   rate `q*` at measured spreads and fees — `wiki/reference/break-even-win-rate.md`. A band
   that went 16/16 with t=+10.3 failed this, so it is not a formality.
4. **Tail at size.** The 8 worst legs of 633 are all `dip-to`, and our two losses were nested
   on one contract — roughly one draw, not two. So: the effective independent sample after
   accounting for correlation among concurrently held legs, and the loss at a realistic clip
   under that correlation. **A variant that clears 1–3 and fails 4 is not promotable**; it is
   a sizing problem, and the honest response is `parked` with the sizing work named.

### The three outcomes

- **Promote to `live`** — all four gates pass.
- **Discard** — the per-market interval lies **entirely below zero**. Reliably worse than the
  market is a finished question, and five day-1 kills have already shown that a clean kill is
  worth more than a hedge.
- **Extend** — only if the *sign* is right and only the *precision* is missing, **and** there
  is a named new source of evidence. This is a high bar on purpose: **the July board universe
  is exhausted.** Further runs on it buy repeats, not power. So an extension necessarily means
  the August cohort, which is a different regime — that is a *new trial* with a new
  pre-registration, not a continuation, and it must be called that.

### The case the rule has to survive

The likeliest Friday outcome, on today's shape, is **negative overall but concentrated**: most
markets at or above zero, a small number of `dip-to` legs carrying the loss. That is gate 1
failing while the mechanism looks fine, and the temptation will be to exclude the tail as
unrepresentative.

**The tail is not excludable.** It is the same barrier structure, the same pricer, and the same
legs we would have held; the two losses were 100% fillable while the flat majority was 2/19.
Excluding the rows we could actually trade and keeping the ones nobody would fill inverts the
entire point of `wiki/reference/midpoint-is-not-a-fill.md`. If the loss is concentrated in the
tradeable tail, that is the result.

`parked` remains available and is the right answer when a mechanism is validated but
untradeable — it is what `arena-rank/favourite-shrinkage` got. It is **not** available as a way
to avoid recording a negative outcome.

### What the rule says about *today*, and what that projects

Gate 1 was unevaluable when I wrote the rule, because `scoring/` reported the per-market mean
and no interval — the reviewer would have hand-computed the statistic that judges the variant.
It now emits `n_markets`, `mean_improvement_market`, `ci_lo`, `ci_hi`, clustering on market.

Run against today's evidence, **every level contains zero**:

| level | n | markets | per-market | 95% CI |
|---|---|---|---|---|
| variant / overall | 32 | 21 | **−0.0127** | [−0.0325, +0.0072] |
| horizon 0–1d | 19 | 19 | −0.0001 | [−0.0023, +0.0021] |
| horizon 1–3d | 8 | 6 | −0.0704 | [−0.1891, +0.0483] |
| horizon 3–7d | 5 | 2 | −0.1077 | [−0.8245, +0.6091] |
| pricer 07-23 | 30 | 21 | −0.0165 | [−0.0432, +0.0103] |

So today the variant is **neither promotable nor discardable** — genuinely undetermined at 21
markets, which is an honest place to be and not a reason to soften anything.

**The projection, stated before the fact.** Friday takes us from 21 markets to roughly 90. If
the dispersion of market means holds, the interval narrows by about √(21/90) ≈ 0.48 — call it
±0.010 against today's ±0.020. Today's point estimate of **−0.0127 sits outside that projected
band**. So: **if the per-market mean holds where it is, Friday's interval excludes zero on the
negative side and the rule says discard.** If the mean moves toward zero — which is what one
would see if the two `dip-to` losses were an unlucky draw rather than the central tendency —
it stays inconclusive and the extend/park question becomes live.

I would rather have written that down while it is a projection than explain on Sunday why the
threshold moved. **Discard is the likely outcome on current shape, and that is fine**: five
day-1 kills have already produced more durable knowledge than any promote would have.

Recorded by the CEO (claude-opus-5), 2026-07-30, before any 07-31 resolution.

---

## 2026-07-29 (late) — Three calls made blind, and the exposure hypothesis I proposed was wrong

All three are recorded **before Friday's outcomes exist**. Slot 1 raised each of them
pre-outcome precisely so the choice could be made without knowing which way it pays.

### 1. The pricer split will be reported INCONCLUSIVE, in the unit that says so

Within `feed_open=1` the comparison reaches **37 rows / 19 markets** today and ~49 / ~20 on
Thursday. That clears the n≥30 floor **in rows and not in markets** — and the readiness
document's own item 7, written before any of this, says markets is the honest unit. Rows are
not independent: we predict the same market every morning, and one barrier touch is scored
once per day it was open. It is the reason the firm reports a per-market level at all.

The board universe is exhausted, so further runs buy repeats rather than power. There is no
version of the schedule that reaches 30 markets by Friday.

**So: the 07-31 pricer comparison is inconclusive, and will be reported that way.** The
row-unit number is reported beside it as descriptive only and is never the deciding number.
**The 08-02 trial decision may not rest on the pricer split.** Switching to the unit that
happens to clear would be choosing the answer, and the threshold was set in markets by us,
in advance, for reasons that have not changed.

### 2. The RV/IV comparison is anchored where we actually trade — and is probably underpowered

The pre-registration fixes the checkpoint at **12:00Z**; `cmd_live` fires at **~01:1xZ**. We
have never emitted a row at 12:00Z, so scoring there measures a decision the variant does not
make (`wiki/reference/checkpoint-artifact.md`, `delayed-execution-test.md`).

**Primary anchor is the emission time (~01:1xZ). 12:00Z is reported as a robustness check.**
Both are fixed now, blind.

Two things also weaken it and neither changes the rule:

- Slot 1 found that the prereg's claim that `q_iv`/`q_blend` are already frozen is **false** —
  the 07-28 file predates those columns by nine minutes. The comparison is scorable from
  **07-29 and 07-30 only**. It should be expected to come back underpowered, and it will be
  reported that way rather than rescued.
- ~~**OVX (57.1) has fallen below RV14 (62.7) for the first time**, which softens the
  premise~~ — **WITHDRAWN 2026-07-30. This was wrong and it was my error to publish.** The
  comparison used the wrong series: the pricer runs on *intraday* realized vol, not RV14.
  Measured against the σ actually in use, **σ_iv sat above it on every asset on both scorable
  days**. The premise never softened, so the argument for not switching stands undamaged.
  Slot 1 caught it. The lesson is the one this entry was itself about: I compared two numbers
  without checking they were the same quantity, in the same breath as telling a researcher not
  to re-specify a test.

  Recorded blind on 07-30, before any outcome: arms B and C both *reduce* sell signals on both
  scorable days (4/1/1, then 2/1/1), so **both fail the tradeability veto ahead of the
  outcome** — the pre-registration's "everything passes" branch is effectively unreachable.

Slot 1 explicitly declined to re-derive the missing columns after seeing today's numbers,
which was right: that is a change to the comparison made by the person who scores it.

### 3. My "structurally short downside touch" hypothesis is REFUTED

I proposed on 07-29 that the two losing markets meant a sell-touch variant is short downside
touch by construction, and asked for it to be measured rather than reasoned about. Measured on
**633 resolved legs / 5,927 checkpoints** with the same pricer as the losing rows, the model
**beats** the market on touched legs (−0.01152, t −1.99), and WTI down legs trending ≥5% into
the barrier are its **best** bucket (−0.01259, t −4.66). The story is wrong.

What is real is a one-sided **tail**: the 8 worst legs of all 633 are `dip-to` legs — across
silver, NVDA, SPY and gold, so a down-barrier property rather than a WTI one. dip-to-80 sits at
p98.7 and dip-to-85 at p97, and they are nested on one contract, so they are roughly **one
draw, not two**.

**That reframes 08-02.** The question is not "is this well calibrated" — it is, on the evidence
we have. It is **"is that tail acceptable at our size, given how correlated the legs we hold
are?"** A sizing question, not a Brier question, and it needs the break-even bound and the
correlation of concurrent positions rather than another calibration table.

Recorded by the CEO (claude-opus-5).

---

## 2026-07-29 — WTI dipped to $80; and a completeness gate on the trial review, pre-registered

Two things, and the order matters: the second is written down *today*, while Friday's rows do
not yet exist, precisely so that using it later cannot be a reaction to what they say.

**The headline got worse, and its shape got clearer.** `will-wti-dip-to-80-in-july-2026`
resolved YES. Per row the trial is **−0.0172 over 25 → −0.0466 over 31**; per market
−0.0051 over 19 → **−0.0133 over 20**. But the loss is not spread across the book:

| split | reading |
|---|---|
| by market | dip-to-80 (−0.1690, **6/6 fillable**) and dip-to-85 (−0.1127, **4/4 fillable**) carry all of it; **18 of 20 markets sit at or above zero** |
| by horizon | 0–1d **−0.0001** over 19 rows, **2/19** fillable — flat and unreachable. 1–3d **−0.1211** (5/7 fillable), 3–7d **−0.1190** (5/5 fillable) |
| by pricer | old −0.0492 (29 rows) vs the 07-27 jump revision −0.0089 (2 rows) — n=2, not evidence, but the column is now doing its job |

Both losing markets are **WTI downside touches** in a week when WTI fell through two barriers.
That poses a question the 08-02 review has to answer and which no Brier number answers on its
own: **is this miscalibration, or is a sell-touch variant simply short downside touch by
construction?** Those have different remedies — one is a model fix, the other is a statement
about which regimes the variant may trade in. Slot 1 has been asked to test it against the
frozen archive rather than reason about it.

**The completeness gate.** 114 rows over 57 markets are outstanding; ~104 resolve Friday
21:00Z and 16 BTC Saturday 04:00Z, and UMA settles a batch over hours, not at once — so the
Sunday review could otherwise read a half-settled batch and call it the trial's evidence.
Recorded in `ops/state.toml`: the review proceeds only when every outstanding row's market is
in `resolutions.csv`; otherwise it slips **one day, once**. A second slip has to be argued
here rather than taken automatically, and the gate may only delay a review — never reopen a
completed one. Written now, with the numbers unknown, for the same reason we pre-register a
pricer comparison: a rule chosen after seeing the result is not a rule.

Recorded by the CEO (claude-opus-5).

---

## 2026-07-28 — The idea funnel becomes a file, and my diagnosis of the slot problem was wrong

Yesterday I filed "seven of nine kills are one failure" as a hypothesis under test. Today I
put the same eleven objects in a table with two columns that the prose was missing — **the
screen that decided it** and **the counterparty it named** — and the table says something the
prose did not. `ops/idea-funnel.md`, maintained from now on, one row per object on the day it
is decided.

Two things fall out.

**The survivors are exactly the objects where no incumbent was found.** All three, no
exceptions in either direction: hit-price ladders, arena-rank, frontline. Every one of the
eight kills named a specific counterparty and measured it; the ninth named the market itself.
Nothing has yet died of a modelling failure or a data failure alone. Either that is the
cleanest screen the firm owns, or it is so dominant that it is the only thing we are really
testing — both readings say run it first and stop paying for the modelling behind a failed one.

**I have been diagnosing the empty slots wrong since 07-25.** I wrote "idea SUPPLY is the
binding constraint on slots" into memory three runs ago and have been repeating it. The rate is
~2.2 objects worked per day and ~0.6 reach a slot; that fills five slots in under two weeks.
The real scarcity is narrower: **1 of 11 objects has reached a trial that can trade today.**
Arena-rank passed its pre-registered test and has no board until ~08-10. Frontline is blocked
on Felix's ruling. So the constraint is objects arriving *with a live tradeable board
attached* — a property of the calendar and of a pending domain ruling, not of researcher
throughput. The practical consequence: **spawning a second market researcher would not have
fixed it**, and I would have spent the tokens to learn that.

`slots_total = 5` is annotated in `ops/state.toml` as a **ceiling, not a target**. An empty
slot costs nothing. A slot filled to look busy cost us `tomatometer/arrival-drift`, promoted
on a described gate 0 and dead the day it was filed.

No change to slot count, no change to cadence. Revisit the "is this the right pond" question
at ~20 objects, which is Felix's to answer and is not ripe yet.

Recorded by the CEO (claude-opus-5).

---

## 2026-07-26 (late) — Gate 0 becomes a regression test: Kalshi's whole catalogue is one call

The market-researcher cycle run under Felix's "no already-efficient markets" directive filed
**no idea**, which is the correct outcome and the one the playbook now asks for. It worked up
the strongest candidate the scan produced — Polymarket's shipping-chokepoint boards resolving
on IMF PortWatch, ~$36M open, 19 resolved weekly ladders, a pure counting process on a free
government feed with zero taker fee — and killed it on three measured gates.

The board passed **every liquidity screen we own**: 1–2c spreads, 174/125 distinct wallets,
~$28k of seven-day taker flow on *both* sides of the leg we would trade, leg-sum 1.019. Worth
saying plainly, because it is the good news buried in a kill: **our liquidity gates are not
the binding constraint any more.** The three ways a quoted price lies are now screened for,
and boards clear them.

What killed it:

1. **Gate 0, measured rather than described** (the rule I wrote this morning after promoting
   an idea on a description). Kalshi's `KXHORMUZWEEKLY` declares **our exact PortWatch
   resolution URL**, trades 156k–446k contracts a week at 1c spreads, and is unbiased for the
   realised settlement: mean error +2.63, se 6.19, **t = 0.42** over n=9.
2. **The cross-venue fallback is also dead.** Polymarket priced the realised winner +4.6pp
   *higher* than Kalshi (se 3.8, t ≈ 1.2). If anything we are the sharper venue here — there
   is no spread to harvest in either direction.
3. **The one that generalises: the feed is not a fixed number.** PortWatch restates settled
   weeks by **−9% to +247%**. Rebuilding all 19 resolved boards from the live API reproduces
   the **wrong winning bucket on 7 of them (37%)**. For the week of 11–17 May, Kalshi settled
   15, Polymarket resolved 40–59 two days later, and the feed reads 52 today. No vintage
   archive exists. So the family is **unbacktestable**, which is a different and worse
   category than "efficient" — we could not have measured our own edge even if one existed.

**The capability this produced is worth more than the idea would have been.** Kalshi's entire
catalogue — **12,186 series with declared `settlement_sources` and `expiration_value`, the
exact settled integer** — comes back from **one unauthenticated call**. Gate 0 stops being an
argument and becomes a regression test: before any modelling, ask whether Kalshi already
declares the same resolution source, and if so compare their line to the settlement. Promoted
to `wiki/reference/sharp-line-screen.md`. `wiki/reference/first-print-vintages.md` gains a
mandatory companion gate: **rebuild ≥3 settled instances from the live feed and check they
match what the venue actually paid**, before modelling anything.

Screening those 12,186 series against everything we have considered found Kalshi covering
nearly all of it — Rotten Tomatoes (244 series), Netflix ranks, MrBeast views, GPU prices,
home values, reality-TV eliminations, chess, earthquakes, Emmys. **The one clean hole is
domestic box office**, against a deep Polymarket family ($17.1M on the Avatar opener, live
boards at $261k and $204k) resolving on The Numbers' *final* figures, explicitly "not studio
estimates". Recorded as a lead, explicitly unverified: its own gate 0 is BoxOfficePro's
forecast, which 403s us live but is archived in Wayback. Half a day, and it runs before a slot
is spent — not after, which is the mistake I made this morning.

**Two process failures, both mine to own.**

I sent the `/execution` route-reassignment message to the **wrong agent** — it reached the
market researcher, whose brief forbids touching `dashboard/`, and which correctly refused to
act on it and told me. The backtest agent therefore never got the instruction and added the
redirect I had tried to prevent; I removed it by hand. Agent ids returned from a parallel
spawn are not ordered the way the calls were written, and I assumed they were.

The backtest agent then ran a repo-wide `git add` and swept the researcher's six files into
its own commit, despite an explicit instruction in its brief not to. Same failure as
2026-07-25. Content is intact on `main` and verified file-by-file; only the commit message
lies. The researcher declined to rewrite shared history with another agent live, which was the
right call — the fix is more dangerous than the defect. Since briefs demonstrably do not
prevent this, the rule has been moved into **`AGENTS.md`**, which every agent reads before
anything else.

---

## 2026-07-27 — Eleven ideas, one live trial, and the kills all say the same thing

Worth stating plainly while it is still a hypothesis rather than a conclusion, because it
bears on how slots get allocated and on whether the firm's premise holds.

Five days, eleven ideas. One in an active trial (and its headline flipped negative today).
One blocked on a domain ruling. **Nine dead, and the causes are not varied:**

| idea | why it died |
|---|---|
| `runningmax` | mispricing clears in 0–3 minutes; bots own it |
| `gistemp-era5` | the crowd runs GISTEMP's primary inputs; σ 0.015 vs our proxy floor 0.038 |
| `bo3-derivatives` | Pinnacle prices it; the residual "edge" was phantom midpoints |
| `satellites` | the market already prices rank persistence |
| `quake-etas` | market's implied Fano 1.362 vs empirical 1.358 |
| `arrival-drift` | Kalshi is the primary venue, with an unbiased line |
| `box-office` | a former trade analyst publishes it free, weekly, in a Substack PNG |

Seven of nine are one failure: **somebody already prices it, and prices it well.** Three of
those (golf/DataGolf, MLB/FanGraphs, box office) were not venues at all but specialists
publishing for free, which is why `wiki/reference/implied-sigma-names-the-incumbent.md` now
exists — it catches the ones no catalogue scan can.

Set that beside today's other result. On the one variant that did survive to scoring, the
ledger splits cleanly: **where there was liquidity we were wrong (6/25 reachable rows, and the
reachable ones are the losses); where we were right there was no liquidity.**

The honest reading is that the screens are working and are telling us something about the
opportunity rather than about our execution. The markets we can reach are mostly efficient,
and the exceptions are mostly untradeable. That is a real finding — five day-1 kills each
produced a durable wiki page, and a firm that discovers its premise is narrower than hoped has
learned something worth more than a fabricated edge.

**No action today.** Two things must land before this becomes a decision rather than an
observation: the 07-31 resolution (~48 rows, the trial's actual evidence) and Felix's ruling on
the war-market domain, which is the one live idea whose fill evidence *passed* — bid-side
notional exceeding ask on all seven legs measured, 1c spreads, fee-free category. If that one
also dies, the question stops being "which idea next" and becomes "is this the right pond",
and that is Felix's to answer, not mine.

Recorded by the CEO (claude-opus-5) as a hypothesis under test, not a conclusion.

---

## 2026-07-27 — WTI touched $85 and the headline flipped: ladder-rv now LOSES to the market

Two markets resolved overnight, both YES, and we were wrong on both.

`will-wti-dip-to-85-in-july-2026` **touched**. We predicted no-touch four mornings running —
0.4937, 0.3520, 0.3928, 0.3650 — while the market went 0.525 → 0.410 → 0.415 → **0.715**. The
second, `will-wti-dip-to-90-...-from-july-25`, also touched, on a leg where we broadly agreed
with the market (0.9409 vs 0.9470).

**The variant's headline inverts: mean paired improvement −0.0172 over 25 rows, from +0.000945
over 21.** `dip-to-85` alone contributes **−0.4510**, which is larger than the total loss of
−0.4312 — every other row still nets +0.0198.

Three things follow, and the order matters because only the first is a defence and it is a
weak one.

**1. Scoring now aggregates per MARKET as well as per row, because rows are not independent.**
We predict the same market every morning, so one barrier touch is scored once per day it was
open. Those four `dip-to-85` rows are one event. Counted per market: 19 markets, mean
**−0.0051**. Counted per row: 25 rows, mean **−0.0173** — 3.4× worse, entirely because we
happened to predict the losing market four times. Both are negative; the flip is real either
way. `scoring/` gained a `market` level so the number of *events* a conclusion rests on is
visible next to the number of rows, and this cuts both ways — it would have deflated the
21/21 headline too.

**2. The 07-26 row is the research finding and it is not about counting.** That morning the
market had repriced to 0.715 and we stayed at 0.365 — a 35-point disagreement, and the market
was right. On 07-23 we were within 3 points of it. So the model did not merely have a
different view; **it failed to move when the market did.** Something priced that move in and
our pricer did not see it. Slot 1 has been told to explain that specifically, today.

**3. Reachability makes this worse, not better.** Tradeability went 2/21 → **6/25**, because
mid-board WTI legs at 0.4–0.7 are exactly the reachable ones. So the rows we could actually
have traded are the rows we lost on, and the wings we "beat the market" on remain the ones
nobody would trade with us. The two halves of the ledger are now cleanly separated: where
there is liquidity we were wrong, and where we were right there was no liquidity.

This is the caveat I have been repeating since the first scoring run — "21/21 was easy OTM
wings, one week, one regime" — arriving on schedule. It is also why the 07-31 resolution
matters more than ever: ~48 further rows on the July commodity boards, four days out, and the
trial review is 08-02. No slot decision today; the evidence that decides it lands on Friday.

Recorded by the CEO (claude-opus-5).

---

## 2026-07-26 (late) — Felix: don't research already-efficient markets; "Execution" is renamed to Backtest

Two directives, both correcting something the firm was getting wrong.

**1. "we shouldn't pick markets that a already efficient (like where will price of NVDA be)."**
This is the failure that has cost us the most, and our own data now names it. Four of our six
dead variants died because a professional already priced the object: `bo3-derivatives` against
Pinnacle, `satellites` against the market's own rank persistence, `quake-etas` against an
implied Fano factor of 1.362 versus an empirical 1.358, and `arrival-drift` today against
Kalshi — whose line is *unbiased for the realised settlement* on boards 3–300× the size of
Polymarket's. In that last case the underlying observation was true and replicated at 8×
sample; it was simply already in the price.

The rule going forward: **a liquid, heavily-traded board on an object professionals price is
not a research target.** Anything shaped like "where will <liquid asset> be on <date>" is out
unless a specific structural reason the crowd is wrong survives our own screens. What we want
instead are boards where the counterparty is *structurally* unable to be sharp — where no
professional cares, where the barrier is work rather than information, or where the crowd
reasons by narrative and the answer is arithmetic. That is also where our only durable
advantage (high-performance Rust, no deadline) actually applies.

**2. "execution would be real execution. i think we rather want a backtest page."** He is
right and the point is substantive, not cosmetic. `CONSTITUTION.md` makes real trading a hard
line, so a surface called "Execution" claims something the firm does not do; everything on it
is a replay of stored signals against stored prices. The dashboard route becomes `/backtest`
(old paths redirect). The **repo directory `execution/` does not move** — it is referenced
from `ARCHITECTURE.md`, this log, several `strategy.toml` success guidelines and the CEO
playbook, and the rename is a presentation change only. He also called the page overloaded and
asked for a full rework, which is dispatched.

Recorded by the CEO (claude-opus-5). The selection rule graduates to `wiki/market-selection.md`
once the market-researcher cycle currently editing that file has finished.

---

## 2026-07-26 — tomatometer/arrival-drift killed on day 1 by the gate I should have run before promoting it

I promoted the Tomatometer idea into slot 2 within three hours of it being filed, on the
strength of its own description of gate 0: that Kalshi runs 233 Rotten Tomatoes series but is
"another retail crowd reading the same page". The day-1 researcher measured that description
and it was wrong on the facts. Kalshi is the **primary** venue for this object — 19 resolved
boards at $58k–$7.19M against Polymarket's $25k median, a 10–29 rung ladder against 3–9, a 1c
median spread where Polymarket's live `90+` leg quotes 0.650/0.830, and The Odyssey traded
$7.19M there against $41k here. Its implied score is **unbiased for the realised settlement**
at every checkpoint from T−96h. The thesis requires the displayed score to sit ~2 points above
settlement; Kalshi therefore already sits ~2 points below the displayed number, which is
verbatim the kill the idea had written for itself.

Two independent confirmations, either sufficient alone. **Gate 3:** on 68 resolved ladder
boards with per-leg ground truth, `price − realised` runs +0.010 (t=+0.23) at T−96h to
−0.171 (t=−3.34) at T−6h — the level claim is falsified in *direction*, and the expensive
half is under-priced by 10.5–29.5pp, which is favourite-longshot bias pointing the opposite
way to this trade. **Gate 5:** the natural form of the trade needs `q* = 0.192` and won 1 of
30, and the idea's liquidity table does not reproduce — board totals match exactly, but the
final-72h in-band split is $8,952 against a claimed $48,846, with median single-leg in-band
flow of $238 over 72h.

`slots_active` back to 1. Variant retired, folder kept as the post-mortem.

**The process failure is mine, and the fix is cheap.** Naming a candidate incumbent and
characterising it is not running the sharp-incumbent screen — it is deferring it, and the
deferral cost a slot-day. Added to `roles/market-researcher/PLAYBOOK.md`: if an idea names
any venue, model or public tool that might already price the object, the measured comparison
goes **in the idea file**, and if the data cannot be got the idea is filed as `needs-gate-0`
rather than `backlog`. I will not spend a slot on an unmeasured incumbent again. An idea filed
honestly as unverified is worth more than one filed confidently as clear.

Worth being clear that the day was not wasted, because this is what day-1 kills are for — six
of our eight variants have now died on day 1 and every one produced something durable. This
one produced four things, and the first is significant beyond the variant:

1. **Kalshi publishes a free hourly bid/ask history** (`candlesticks`). That is the historical
   order book `wiki/reference/midpoint-is-not-a-fill.md` says we have been missing — our
   fillcheck reachability numbers are a *lower bound* precisely because a resting bid nobody
   hit leaves no trace in a trade feed. A real quote history would replace the bound with a
   measurement. Top wiki item for the next run.
2. A **favourite-longshot replication in an unrelated family** — `arena-rank/favourite-shrinkage`'s
   mechanism appearing in film-score ladders, with one band clearing `q⁻ > q*` while a 15-for-15
   band still fails. `wiki/reference/break-even-win-rate.md` proving itself on fresh data the
   day after it was written.
3. `how-to-make-a-killing` resolved **incoherently** — `≥56` NO and `≥57` YES, with $190k on the
   broken leg. A venue-integrity data point.
4. `endDate` is **not** the resolution instant in this family; up to 15h of checkpoint drift.

The researcher also flagged, unprompted, that it never audited the founding −2.23-point drift
measurement because the Wayback harvest was still running — and that its Rust crate, though
verified against a 10⁶-draw sampler to TV 0.0057, was never fitted to a conclusion. Saying so
plainly instead of dressing a dead thesis in a backtest is exactly the behaviour I want.

Decided by the CEO (claude-opus-5); analysis by the slot-2 day-1 researcher (claude-opus-5, max).

---

## 2026-07-26 — New status `parked`; arena-rank/favourite-shrinkage passes its kill test and loses its slot anyway

Slot 2 ran its pre-registered day-3 band test a day early — correctly, since the cohort
checks 07-31 and any trade had to go on now. **It passed decisively.** The favourite-longshot
gain concentrates exactly where the variant committed it must, in the fundable 0.60–0.90
band: +16.8pp over n=74 across 10 months, t=+5.94, +15.2% return on locked capital, 95%
lower bound on the win rate 0.846 against a 0.829 break-even. It survives a leg-sum gate and
a 10-fold month jackknife.

And it proposed **zero rows**, because the mechanism has no expression in the cohort it was
handed. Six of seven July boards sit at 0.935–0.983 with four quoting an ask of 0.990 — pay
99c to win 1c, where one loss per hundred wipes the band out. The seventh is in band and
fails a screen the variant did not have this morning: at a 0–3 leaderboard margin with the
crowd backing the incumbent, the crowd is already right (n=5, market 0.800 → realised 0.800,
our rule 0.951 — the largest model error in the sample). August and September are listed but
**unpriced**, leg-sums 6.5–12.5, i.e. phantom ~0.5 on empty books. Nothing to trade for
roughly two weeks.

**Introduced `parked` as a variant status** (`trial | live | parked | retired`;
`strategies/README.md` documents it). `retired` means a gate killed the thesis and the folder
is a post-mortem. `parked` means the thesis held and has no expression: the boards it needs
are unlisted, unpriced, or outside the band it committed to. A parked variant releases its
slot and stops counting as active work — a slot that cannot trade for two weeks is a slot the
firm is pretending to use, and `ops/state.toml` claiming `slots_active = 2` would have been a
lie to the only human reading it. It keeps its trial clock and its evidence, and
`reopen_when` names the observable condition (an Aug/Sep board with leg-sum ≤ 1.05, favourite
in 0.60–0.90, passing the margin screen, from ~08-10) so reopening is checkable rather than
remembered. All 182 legs stay in the watchlist, so the snapshot worker accumulates the
evidence to reopen on whether or not anyone is watching.

`slots_active` 2 → 1. This is the first variant to leave a slot without being wrong.

Two durable pages written from it, both of which generalise well past this variant:

- **`wiki/reference/break-even-win-rate.md`** — the best artifact this firm has produced. A
  band that went 16/16 with t=+10.3 is uninvestable because it needs a 97.2% win rate and
  2.83 losses per 100 trades take it to zero. Report `q*` (break-even), `q`, and the 95%
  lower bound; refuse when the bound is below `q*`. This is now the standard promotion gate
  for any favourite-side trade, and it retires cents-per-trade as a ranking metric: cents
  ranked the bands 4:1, RoLC 5.2:1, and the bound ranked them tradeable / not / not.
- **`wiki/reference/sharpen-only-what-persists.md`** — a favourite-longshot correction inside
  a recurring ranking cohort is conditional on the ranking persisting; measure persistence on
  the resolution variable's own archive, at the granularity the board resolves on. Includes
  the pooled-statistic trap: the losing application cited a 0.976–0.982 persistence figure
  that was real but computed on established 50k-vote rows, quoted for a pair in a
  6.5-sd sub-population where it is 0.846. Sibling of `published-ci-vs-printed.md`.

Decided by the CEO (claude-opus-5); analysis by the slot-2 researcher (claude-opus-5, xhigh).

---

## 2026-07-25 — Dashboard reads are pinned to a commit SHA and issued concurrently

Felix: "it feels much slower now. it was fast before." He was right, and the first
diagnosis was wrong. The cause was not the fallback removal — it was that the
`GITHUB_TOKEN` worker secret landed **today**. Before that, `token(env)` returned `None`
and every read short-circuited straight to the compiled-in pack: twenty in-memory string
scans, zero I/O. The moment the token existed, those same twenty reads became twenty
*sequential* HTTPS round trips.

Measured before the fix: latency scaled linearly with the number of reads — `/decisions`
(1 read) 0.24s, `/strategies` (~10) 0.62s, `/` (~20) 0.87s warm and **2.9s** whenever the
60-second cache clock lapsed.

Two changes, both in the read layer:

1. **Pin content reads to a commit SHA.** `live::head()` resolves `main`'s SHA (memoised
   per isolate for 60s — that TTL is now the dashboard's only freshness knob) and every
   file and tree read goes to `?ref=<sha>`. The URL then names an immutable blob, so it
   caches for a day instead of a minute. A push produces new URLs; old entries just go
   unused. The every-60-seconds cliff is gone by construction, not by tuning.
2. **Issue independent reads concurrently** (`data::read_all`, `futures::join!`). A page's
   reads never decide each other — only the tree must land before we know which variant
   and run manifests to fetch — so every page is two waves rather than N steps. The
   per-variant and per-run loops were the worst offenders and are now single batches.

Result: `/` **0.87s → 0.41s**, every other page 0.21–0.38s, and latency no longer scales
with read count. All 17 routes verified 200 with correct content afterwards.

Not done, deliberately: fetching the whole repo as one tarball, or mirroring it to R2 and
reading through the binding. Both would shave another ~0.15s and both add a moving part;
at 0.4s the page is no longer the bottleneck. Revisit only if the repo grows enough that
the tree read itself gets slow. Decided by Felix, implemented by the CEO (claude-opus-5).

---

## 2026-07-25 — Dashboard has no fallback copy of the repo: it reads `main` or shows an error

The Worker compiled every renderable repo file into its own binary (~170 files, ~1.0 MiB)
and served that whenever a GitHub read failed. Felix called it: we don't need it, show an
error instead. Removed.

The reason it had to go is not the megabyte, it is the failure mode. A dashboard that
silently swaps in a build-time copy during an outage does not look broken — it looks like
a working dashboard showing numbers that happen to be old. Every number on it is a claim
about the firm's current state, so an invisible fallback is a machine for producing
confident wrong answers. A visible gap is strictly better than a plausible stale one, and
that is the same principle as the fillable-count decision above: prefer the honest hole.

- `main` at request time is the only source of truth. A failed read yields empty text.
- The top bar reads **`cannot read repo`** instead of a timestamp, and a red banner names
  the cause: no token, 401 (token revoked), 403 (rate limit or scope), 5xx (upstream).
- The banner is rendered once in `render::layout` from the freshness state, not by each
  page — a page cannot forget it. That deleted 15 per-page banner call sites.
- Transient failures (network, 5xx) are retried once. A retry costs one subrequest and no
  staleness, unlike serving an old copy; 401/403/404 are definitive and never retried.
- Bundle: 2390 KiB → 1306 KiB raw, 781 KiB → 521 KiB gzipped (−45%). Repo edits also no
  longer trigger a Worker rebuild, since nothing outside `src/` is compiled in.

Verified with the token removed: every route still returns 200, renders its empty state
plus the error banner, and leaks **zero** repo content (grep for `barrier-touch` on the
no-token render: 0 hits). With the token, 76 loads across every route showed one failure —
the first request against a freshly deployed version. It is rare, it is now visible by
design, and a reload clears it. Decided by Felix, implemented by the CEO (claude-opus-5).

Worth being precise about one thing, since it came up: we do **not** mirror the repo to
R2. The bucket holds hourly market book snapshots written by `workers/snapshot`, and
`tools/r2data` blobs. The thing just deleted was a copy compiled into the Worker binary.

---

## 2026-07-25 — Scoring reports tradeability next to calibration; the first batch was 2/21 fillable

Our first scored batch was 21 predictions, all `barrier-touch/ladder-rv`, and all 21 beat the
market on paired Brier. That headline was reported without checking the one thing that makes
it mean anything: `market_price` is a CLOB **midpoint**, and a midpoint on a wing leg is the
average of a near-zero bid and a fat ask. It is not necessarily a price anyone will give you.

Built `tools/fillcheck` (Rust, `attohttpc` behind the agent proxy like `r2data`), which
replays Polymarket's public trade feed for every market we predicted on and records the best
price a counterparty was demonstrably reachable at on each side, in windows of 1h / 24h /
life. `scoring/` now joins the result and prints `n_fillable` and `exec_edge` on every
aggregate.

The answer: **21/21 beat the market, 2/21 were reachable, 1/21 within the first hour.** The
one liquid row (`will-wti-dip-to-90`, $34k volume) is the row where we had essentially no
edge — 0.8263 against a market at 0.82 — and it contributed 11% of the batch's improvement.
The other 89% sits in SPY/NVDA weekly wings where `will-spy-reach-760` was scored at a 2.55c
midpoint against a best-ever bid of 0.12c.

What changed, and why:

- **`scoring/` will no longer print a Brier improvement without a fillable count beside it.**
  Reporting calibration as if it were money is the single easiest way for this firm to fool
  itself, and it already happened once.
- **Promotion decisions turn on `exec_edge`, not `improvement`.** `ladder-rv`'s
  `success_guideline` is amended; its 2026-08-02 review uses the executable number.
- **Weekly equity ladders are demoted to research-only** for `ladder-rv` — still predicted
  on, never counted in a headline without their own fill evidence. The monthly WTI/gold/
  silver boards resolving 2026-07-31 are the trial's real evidence.
- Durable rule: `wiki/reference/midpoint-is-not-a-fill.md`. Evidence:
  `strategies/barrier-touch/ladder-rv/results/executable-price-audit-2026-07-25.md`.

Honest limit, recorded so nobody over-reads it: `fillcheck` sees trades, not orders, so a
resting bid nobody hit is invisible to it. 2/21 is a lower bound. The real fix is recording
the book at prediction time (`bid`/`ask`/`depth_usd` columns, sourced from the snapshot
worker), which is now the top infrastructure item.

This independently corroborates the execution engine from the other direction: on
`orakel-live` signals, seven of eight execution policies took zero trades. Two methods, one
conclusion — this variant's demonstrated edge lives where the liquidity isn't. Decided by
the CEO (claude-opus-5).

---

## 2026-07-25 — Dashboard switched from build-time snapshot to live repo reads

Felix provisioned `GITHUB_TOKEN` in the environment, so it was set as the `orakel-dashboard`
Worker secret (`wrangler secret put`) and the Worker redeployed on the current `main`. Every
page now reads `main` at request time instead of the pack embedded at build time; the
"snapshot" banner is gone and the top bar says `live`.

Verified rather than assumed: the freshness stamp on the deployed dashboard tracked a commit
pushed **after** the running build's timestamp — only possible via a request-time read — and
all eleven routes return 200 with content, including the tree-driven listings (`/strategies`
lists all four variants), which exercise the Trees API and not just Contents.

Why it matters operationally: agents and Felix now see the firm's actual state, not the
state as of the last deploy. A stale dashboard was a real risk — the previous build was 5
minutes old and already missing a code change and three commits.

One caveat filed to Felix (`roles/felix/inbox/2026-07-25-github-token-scope.md`): the
provisioned token is a classic PAT with `repo` write scope on all his repositories, where
the dashboard needs only read-only Contents on this one. Working as-is; worth narrowing.

**Decided by:** Felix (asked for the redeploy); executed and verified in-session.

---

## 2026-07-25 — Venue fees found, verified, and priced into every policy (v2)

The market researcher discovered Polymarket charges real taker fees, undocumented in our
wiki: `fee = shares × rate × p × (1−p)`. The execution engine had `fee_bps = 0`
everywhere, so the matrix I had just reported was too generous. Corrected same day.

Verified three independent ways (not taken on the wiki's word): the published docs; each
market's own `feeSchedule` on Gamma across 600 markets; and a fit against ~2,300 real
executed fills, which additionally **ruled out** the plausible `min(p, 1−p)` form.
Established facts: charged **per taker fill, on entry and on exit, never at resolution**
(so a held position pays once, a round trip twice); **makers pay zero**; and — the piece
nobody had right — **gold, silver, WTI, SPY and NVDA are `finance` at 0.04**, read off
each market rather than guessed. Sports was 0.03 before 2026-07-10, which matters for
any future sports backtest.

Eight `-v2` policies were created rather than edited (DESIGN.md §5), and v1 re-runs
bit-identical — the proof that fees are the only difference.

**All three conclusions I reported survive**, with two changes worth stating: fees take
8–25% of gross PnL; the sell/buy split sharpens from (+7.75c / +0.47c) to
(**+7.20c / −0.22c**), i.e. the naive buy book is now an outright money-loser, which
independently vindicates ladder-rv's decision to disable buys; and while `harvest` keeps
the top rank, its lead over `sniper` collapses by 74% (213pp → 54pp) because it pays the
fee twice. A ranking that survives is not the same as a conclusion that is unchanged.

---

## 2026-07-25 — Execution layer built; watchlist mirroring moved to run start

Built the execution simulator (`execution/`): eight named policies, two signal sets, the
capital-lockup accounting rule (annualized return on locked capital, not cents/trade),
conservative fills (never at mid), and a refusal to name winners below n=30.

First matrix (details in `execution/results/SUMMARY.md`) produced three findings:
**(a)** filtering is the single biggest lever — `mirror`→`gate` roughly doubles
annualized return on strictly fewer trades; **(b)** the sell-side house finding
replicates independently (sells +7.75c/trade vs buys +0.47c on the naive policy);
**(c)** the two headline metrics genuinely disagree — `sniper` wins cents/trade,
`harvest` wins annualized return because it holds 3 days instead of 10 — which is the
design's own argument reproduced on data.

**And one sobering result: on our OWN live predictions, seven of eight policies take
zero trades.** After a 3c spread our 21 scored predictions had under 5c of *executable*
edge: they were 2–7c wings whose "edge" was measured against a midpoint that is not a
tradeable price. Being right 21/21 and having nothing to trade are compatible states,
and the firm now measures both.

Operational change (CEO playbook step 3): the R2 watchlist is now rebuilt from **active
applications at the START of every run**, not from predictions at the end. Root cause of
the missing books: the watchlist grew 18→40 markets 26 minutes *after* the run that
produced 18 of the 21 signals, so the hourly snapshot worker had never seen them. Fixing
the order makes future signal sets book-complete at zero cost.

Also corrected a 10× arithmetic error in DESIGN.md §3's worked example (the formula and
the engine were always right; the prose was not) — caught by the implementing agent.

---

## 2026-07-25 — arena-rank: thesis killed, mechanism kept (variant split)

`arena-rank/satellites` day-1 falsification killed its founding thesis on gate 2: the
anchor-calibrated order-statistic simulation lost to the satellite crowds (log-loss
1.244 vs 0.504, better in 1/10 cohort-months), and the portfolio-correlation effect
calibrated to zero. Root cause is now a wiki rule: the leaderboard publishes CIs about
LATENT skill (±5.9) while the market resolves on the PRINTED rank, whose realised 7-day
sd is 1.23 — using published bars as σ over-disperses and fades favourites.

But one mechanism survived with better statistics than the original claim: the crowds
are **underconfident in their own favourite** (+9.2pp vs de-vigged price at T−7d, se
1.9pp, t=4.77, 9/10 months), and sharpening their distribution gains +0.111 log-loss
OOS (t=+2.63; at T−7d t=+7.49, 10/10 months).

Decision per our taxonomy (different approach → new variant, not a version bump):
retire `satellites` with its post-mortem, create `arena-rank/favourite-shrinkage`
(`supersedes = "satellites"`) carrying only the surviving evidence. The slot clock is
NOT reset — day 1 is spent. **A kill test is pre-registered for day 3**: the
favourite-longshot gain must concentrate in a fundable 0.60–0.90 band; if it exists only
on 0.93–0.99 favourites, return on locked capital after spread cannot justify a slot and
the variant retires. The retired simulation's forward prediction rows were deliberately
NOT logged — we do not put a dead mechanism's calls into the track record; day 2
produces shrinkage-based rows for the same cohort, still ahead of the 07-31 resolution.

---

## 2026-07-24 — Model routing: Opus 5 everywhere (Felix)

Opus 5 released; Felix directs: use it wherever Fable was used, at **max** effort, and
**xhigh/high** for the roles that already ran Opus. Rationale carried over from the
original split — idea generation and day-1 falsification are the highest-leverage
decisions (each bad call burns a slot), so they get the deepest thinking; recurring
daily research and execution are more mechanical. Fable is retired from routing. Note:
prediction rows and worklogs must now record `opus-5` (+ effort) as the producing model
— the model column keeps separating method-edge from model-edge.

---

## 2026-07-22 — CEO instantiated (Felix's instruction)

The scaffolding session is promoted to the CEO: it becomes the CEO's persistent session,
woken daily by a self-bind trigger at 01:07 UTC (03:07 German summer time — inside the
working window year-round). Felix chose self-bind over fresh-session mode because it
keeps all MCP connectors (verified live earlier today; fresh sessions from agent-created
triggers lose them). Model routing per constitution §4: subagents on Fable run at high
effort only; Opus subagents may run extra-high. First CEO run starts immediately:
market researcher scan → first strategy idea → fill research slot 1.

## 2026-07-22 — Founding (Felix + scaffolding session)

The firm is founded as the successor of `poly`, redesigned around lessons from its ~2-week
run. Founding decisions, agreed between Felix and the scaffolding agent:

- **Research unit = strategy variant**, not market. poly's per-market research (3
  researchers/market) produced correlated one-shot papers, n=2-3 per method, and its
  `strategies/` promotion path never fired once.
- **Family → variant → application** taxonomy with the params-plus-small-local-changes
  membership rule; split variants rather than over-generalize. Versions = name postfix +
  `supersedes` field.
- **5 research slots**, ≥10-day trials judged on scored evidence (guideline: ≥15 scored
  predictions across ≥3 markets beating the market baseline + backtests on resolved
  markets). CEO decides promote/discard/extend.
- **Roles with own memory + inboxes**: CEO (orchestrates, never researches), market
  researcher (daily scan → one idea/day), researchers (per slot), executors (per live
  variant). Felix is a role with an inbox.
- **One daily CEO trigger owned by Felix**; CEO spawns everything else and may create
  further triggers inside the working window (weekdays 02:00–15:00, weekends 02:00–08:00
  Europe/Berlin).
- **No hard token cap initially**; spend logged per run. Model routing: Fable
  (high/xhigh) for market research + initial research, Opus for recurring research +
  execution.
- **Git = index, R2 = bytes** (poly committed 70 MB of snapshots into git). Upload-before-
  commit, immutable content-addressed keys.
- **Execution layer from day one** (paper only): versioned execution policies with signal
  combination folded in, PnL-backtestable. Real trading stays a Felix-only decision.
- **Dashboard**: dynamic Rust app on Cloudflare Workers, private via Cloudflare Access,
  htmx + ECharts, deployed from agent sessions via wrangler.
- **Wiki seeded** with a curated handful of durable poly insights (market selection,
  favorite-longshot bias, thin-market price reading, crowd calibration, wash-trade
  detection, Polymarket API recipes); everything else clean slate.
