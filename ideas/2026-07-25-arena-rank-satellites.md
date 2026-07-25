---
date: 2026-07-25
slug: arena-rank-satellites
status: trialing # thesis KILLED day 1; surviving mechanism -> strategies/arena-rank/favourite-shrinkage (slot 2)
example_markets:
  [
    "which-company-has-the-third-best-ai-model-end-of-july",
    "best-chinese-ai-company-end-of-july",
    "which-company-has-the-best-code-arena-webdev-ai-model-end-of-july-20260715140712903",
    "which-company-has-best-ai-model-end-of-july-299",
  ]
---

## Thesis

Polymarket runs a **monthly family of seven-plus boards that all resolve off one object
read at one instant**: the arena.ai / LMArena **Text Arena "Rank" column**, style control
off, checked at 12:00 PM ET on the last day of the month. Today's July cohort:
`#1 overall`, `#2 overall`, `#3 overall`, `Math` sub-arena, `Coding` sub-arena, `WebDev`
sub-arena, `#1 with Style Control On`, and `Best Chinese AI company` (the same ordering
restricted to a company subset). Every one of them is a **deterministic function of a
single latent ranking of 378 models**.

Their liquidity spans **250×** — $30,299 (WebDev) to $7,587,062 (#1 overall) — and their
top-of-book spreads span **37×** (0.1c to 3.7c). That asymmetry is the whole idea:

> The deep #1 board is efficient, and that is what makes it useful. It is a free, sharp,
> $7.6M anchor on the *same latent variable* that seven satellite boards price with
> crowds 10–250× thinner. Simulate the ranking once from the leaderboard's own published
> uncertainty, calibrate the simulation so it reproduces the deep board's implied
> distribution, and the satellites are priced by arithmetic rather than by their own thin
> crowds.

Two mechanical facts the satellite crowds do not price:

**(1) These are order statistics over company *portfolios*, not model bets.** The question
is which *company* owns the k-th ranked *model*. A company with several models clustered
at the top wins these boards through a max/order statistic over correlated scores. Right
now **Anthropic holds ranks 1, 2, 3, 4 and 6** (1507 / 1505 / 1502 / 1498 / 1494); the best
non-Anthropic model (Meta `muse-spark-1.1`, rank 5, 1495) is 12 points below rank 1 but
only 3 below rank 4. "Anthropic owns the 3rd-best model" and "Anthropic owns the best
model" are then *very* differently-shaped events — the crowd prices them 0.934 and 0.991
on boards whose volumes differ by 70×, with no joint model tying them together.

**(2) The Rank column is an estimate, and the leaderboard publishes its own error bars.**
Every row carries a Bradley-Terry score with a **±CI**, a **vote count**, a
**"Preliminary"** flag, and an explicit **Rank Spread**: rank 1 (`claude-fable-5`, 1507 ±6,
14,646 votes) is stamped **spread 1–5**; rank 10 (`kimi-k3`, 1486 ±10, 3,619 votes,
Preliminary) is stamped **spread 4–27**. Low-vote entrants are the loudest narrative
("Kimi is top-10!") and the widest distributions. Casual traders read the integer in the
Rank column; the publisher is telling them it is worth ±5 to ±23 places.

Who is on the wrong side: the satellite boards' crowds — AI-sector retail pricing each
board in isolation off a launch narrative, on 1/10th to 1/250th of the headline board's
flow, at 3–37× the spread.

## Who is the sharpest incumbent here, and what data do they run?

There is no financial incumbent — no desk prices arena Elo, and there is no derivative,
index or hedge that would put a professional on this variable. The sharpest plausible
participant is someone who scrapes the leaderboard daily and diffs it.

**And that is the ceiling for everyone, including them.** The resolving index's upstream
input is LMArena's *private, unpublished vote stream*. Nothing public reconstructs it.
There is no GHCN-M+ERSST here — the published table **is** the primary, and it is the same
artifact for us, for the crowd, and for the sharpest agent in the market. Verified today:
a cache-busted refetch of `lmarena.ai/leaderboard/text` serves data dated **Jul 21, 2026**
(7,430,560 votes, 378 models) — a discrete, weekly-ish publication cadence, the same
vintage for every participant. We are not on a lagged proxy of anything.

So the answer to "why hasn't the sharp money already done this?" is not "we have better
data" — it is: **nobody can have better data, so the only edge available in this family is
the joint simulation, and the observable evidence is that the satellite boards are not
being simulated jointly.** Evidence, all pulled today:

- The **WebDev August** board's tradeable legs sum to **1.222** — a 22% overround, with
  twelve legs quoted 1.3–3.4c — while the **WebDev July** board on the same object sums
  to 1.018 and the **Math August** board sums to 0.934. Three boards on one leaderboard,
  overrounds of +22%, +2% and −7%.
- **Moonshot** is priced 0.001 (#1 overall July), 0.017 (Math July), 0.182 (Chinese July)
  and **0.662** (WebDev July) — four prices for one company's position in one ranking.
- The **Chinese** board prices **Alibaba 0.786 vs Moonshot 0.182** while the resolving
  table has Moonshot's `kimi-k3` at **rank 10 (1486 ±10, Preliminary, 3,619 votes)** and
  Alibaba's `qwen3.7-max-preview` at **rank 19 (1475 ±10, Preliminary, 3,714 votes)** —
  4.3× the price on the company that is currently 11 points and 9 ranks *behind*. That is
  either a specific, modelable forecast that the next refresh reverses two preliminary
  models, or narrative anchoring. **It is adjudicated on 2026-07-31, six days from now.**

## Example markets (numbers pulled 2026-07-25, ~01:30Z; all check 2026-07-31 12:00 ET)

| board | volume | liquidity | top leg (mid, bid/ask, spread) | leg sum |
| --- | --- | --- | --- | --- |
| #1 overall | $7,587,062 | $2,142,843 | Anthropic 0.991 (0.990/0.991, 0.1c) | 1.001 |
| Chinese only | $654,996 | $129,932 | Alibaba 0.786 (0.784/0.788, 0.4c); Moonshot 0.182 (0.172/0.193, 2.1c) | 0.991 |
| Math sub-arena | $368,488 | $116,343 | Anthropic 0.944 (0.941/0.946, 0.5c); Moonshot 0.017 | 0.974 |
| Style Control On | $125,361 | $54,794 | Anthropic 0.977 (0.958/0.995, 3.7c) | 1.013 |
| #3 overall | $108,836 | $51,646 | Anthropic 0.934 (0.921/0.947, 2.6c); xAI/OpenAI/Meta 0.009 each | 0.989 |
| #2 overall | $77,414 | $63,454 | Anthropic 0.976 (0.963/0.988, 2.5c); Google 0.016 | 1.026 |
| Coding sub-arena | $40,317 | $43,369 | Anthropic 0.983 (0.968/0.997, 2.9c) | 1.013 |
| WebDev sub-arena | $30,299 | $36,340 | Moonshot 0.662 (0.658/0.667, 0.9c); Anthropic 0.331 (0.327/0.334, 0.7c) | 1.018 |

August cohort already listed (checks 2026-08-31): #1 overall $300,716 (Anthropic 0.885 /
OpenAI 0.062 / Google 0.036); Math $19,663 (Anthropic 0.755 / Google 0.095, sum 0.934);
WebDev $3,933 (Moonshot 0.755 / Anthropic 0.175, **sum 1.222**).

Resolving table as of the Jul 21 refresh (rank | spread | model | org | score ±CI | votes):

```
 1 | 1-5   claude-fable-5            Anthropic  1507 ±6    14,646
 2 | 1-5   claude-opus-4-6-thinking  Anthropic  1505 ±4    63,191
 3 | 1-6   claude-opus-4-7-thinking  Anthropic  1502 ±4    50,683
 4 | 1-7   claude-opus-4-6           Anthropic  1498 ±4    67,037
 5 | 1-13  muse-spark-1.1            Meta       1495 ±7  P  7,927
 6 | 3-11  claude-opus-4-7           Anthropic  1494 ±4    51,788
 7 | 5-17  muse-spark                Meta       1488 ±6  P 13,565
 8 | 6-16  gemini-3.1-pro-preview    Google     1486 ±4    84,631
 9 | 5-16  gemini-3-pro              Google     1486 ±4    41,268
10 | 4-27  kimi-k3                   Moonshot   1486 ±10 P  3,619
11 | 5-24  gpt-5.6-sol-xhigh         OpenAI     1485 ±8     6,221
19 | 7-42  qwen3.7-max-preview       Alibaba    1475 ±10 P  3,714     (P = Preliminary)
```

Supply: 104 events in the family, **78 already closed**; the headline monthly board has run
$4.15M–$36.3M per instance since 2025. Seven-plus boards resolve every month, and the
current cohort resolves in **six days** — a trial started now scores almost immediately and
then again every month.

## Screen 1 — speed race (wiki/reference/delayed-execution-test.md)

**Passes by construction, with one explicit exclusion.** The resolving object updates on a
discrete, roughly weekly cadence (live page today still serves Jul 21 data), and the boards
resolve at a fixed instant 6–37 days out. The target mispricing is *model-revealed*
(portfolio order statistics + CI simulation + cross-board coherence), not print-revealed;
it is harvested by holding to the check, not by racing anyone.

The excluded sub-mechanism: when a refresh lands or a new model appears, the boards reprice
fast — that **is** a speed race and the strategy must not be built on it. Gate 1 measures
it: take the refresh timestamps from the Wayback captures, and measure how fast the boards
move around them. Any signal whose lifetime is under a day is bot food and is dropped.

## Screen 2 — proxy vs primary (wiki/market-selection.md; gistemp-era5 kill)

**This is the screen the idea is designed around, and it passes structurally.** The kill
question is "can the crowd replicate the resolving index from its own upstream inputs?"
Here the upstream is a private vote stream: **nobody can, ever.** The published table —
score, ±CI, vote count, Preliminary flag, Rank Spread — is the primary artifact for every
participant simultaneously, and we hold exactly the same copy (verified read-only,
cache-busted, today). The gistemp failure mode (crowd σ 0.015 via primary-input replication
vs our proxy floor 0.038) has no analogue available to anyone in this market.

The honest counterpoint, which is kill condition 2: LMArena publishes its uncertainty, so
a sharp crowd *could* read it. The claim is not privileged data — it is that we run the
joint order-statistic simulation and they read a rank column. **Measure the crowd's implied
precision first**, per the screen: on the 78 closed instances, take the modal leg's
de-vigged price at T−30d / T−14d / T−7d / T−1d and compare to its realized win rate, per
board type. If the satellite boards are already calibrated, the idea dies for one day of
work and no slot beyond that.

## Screen 3 — first-print vintages (wiki/reference/first-print-vintages.md)

**Worse than a first print, and it must be handled before any number is believed.** The
leaderboard is *recomputed over the entire vote history at every refresh*: a model's score
"as of" a past date changes retroactively, and there is no archived print file at all —
only the page as rendered at the check instant. **Today's table is not what resolved any
past market.** A backtest scored on it is corrupted exactly the way gistemp's was, and
silently.

Measured availability: Wayback holds **500 unique captures** of `lmarena.ai/leaderboard*`
from **2025-05-28 to 2026-01-28** (205 in Nov 2025, 92 in Dec, 67 in Jan) — and **zero
after 2026-01-28**. So:

- Historical vintages exist for roughly the Jun-2025 → Jan-2026 checks only; Feb–Jul 2026
  instances **cannot** be vintage-reconstructed and must be excluded from scoring rather
  than graded against today's table.
- Also check `arena.ai` captures separately: the site rebranded mid-family (the July board
  cites `lmarena.ai/leaderboard/text`, the August board cites
  `arena.ai/leaderboard/text/overall-no-style-control`). A moved/renamed resolution source
  is the ladder-rv day-1 hazard (Pyth delisting expired feeds) in another costume.
- **Archive the live table ourselves, daily, from day 1** — including score, CI, votes,
  Preliminary flag and Rank Spread per row, for every arena slice a board references. That
  archive is the resolution record and the only vintage source that will exist for the
  forward trial.

Corollary for live prediction: the leaderboard's own ±CI is a *lower* bound on our σ. The
score at check time is a noisy draw from a distribution the publisher already quantified —
put the published CI plus the refresh-to-refresh drift of Preliminary models into σ in
quadrature, never just the point score.

## Falsification sketch

Sample: the 78 closed instances (headline board back to early 2025; satellite boards for as
long as each has existed), plus CLOB `prices-history` for every leg, plus the Wayback
vintage set. Gate 0 first, then the kill gates.

- **Gate 0 (prerequisite) — can we reproduce resolutions from the source?** For every
  closed instance with a usable capture near its check instant, recompute the winner from
  the archived table (correct slice, style control off, "Models" filter, tie-break by
  unrounded score then alphabetical company). **Kill if <90% reproduce** — it would mean we
  cannot read the resolution variable, and everything downstream is noise. This also fixes
  how many satellite instances are actually scoreable, which is the idea's biggest known
  unknown.
- **Gate 1 (speed) — violation lifetime.** Around each refresh timestamp, measure how long
  the cross-board incoherences and model-vs-market gaps persist. **Kill the sub-signals
  whose p50 lifetime is under a day**; kill the idea if *all* of them are.
- **Gate 2 (market-already-sharp) — paired log-loss and modal calibration.** Model (joint
  ranking simulation, calibrated to the deep #1 board) vs de-vigged satellite mids, at
  T−30/14/7/1d. **Kill if the market beats the model at every checkpoint** — the gistemp
  rule, applied without negotiation.
- **Gate 3 (is the portfolio effect real?).** Does P(company = argmax over *its whole set*
  of models), computed from published score/CI/vote structure, beat P(company | its
  headline model only) out of sample on the #2/#3 boards? **Kill mechanism (1) if the
  portfolio treatment adds nothing** — the whole "companies are portfolios" claim rests on
  it.
- **Gate 4 (t+24h delayed execution).** Fills at t+24h mid, +2c adverse, inputs frozen at
  t, tokens in the fundable 3–50c zone, hold to check. **Kill if the edge collapses or
  sign-flips across halves** (runningmax rule).
- **Gate 5 (capacity and book reality).** Per satellite board per month, taker notional in
  the 3–50c zone from the tape, and top-of-book dollars on the legs the model actually
  signals. Today's satellites look genuinely tradeable — Moonshot 0.172/0.193 on a $655k
  board, Moonshot 0.658/0.667 and Anthropic 0.327/0.334 on the WebDev board — unlike the
  0.1c dust in most bucket families. **Kill if matched fundable flow is under ~$500/month.**

Free early scoring, whatever the backtest says: the entire July cohort checks on
**2026-07-31 12:00 ET**. Recording model probabilities for all eight boards now costs
nothing and grades the simulation in six days — in particular the Chinese board's
Alibaba 0.786 vs Moonshot 0.182, against a table that currently has Moonshot ahead.

## Distinctness and known risks

- **Not a duplicate.** `barrier-touch/ladder-rv` is a first-passage model on financial
  candle feeds; the two kills were a station-truncation speed race and a climate-index
  proxy nowcast. This is a categorical order-statistic problem on a non-financial
  leaderboard. It does share ladder-rv's *sibling-coherence* diagnostic and its
  fundable-zone discipline — both should be reused, not reinvented.
- **Main structural risk: release timing is partly an insider process.** Whether a lab
  ships a new SOTA model before the check is private information we cannot model, and
  `wiki/market-selection.md` says to select against exactly that. Scoping consequence: the
  strategy should live on boards and legs whose outcome is dominated by *existing* models'
  score dynamics — sub-arenas, #2/#3, company-subset boards, short horizons — and stay off
  the "will someone dethrone the leader" bet that dominates the headline board at long
  horizon. Note this is also where the liquidity asymmetry favours us, so the constraint
  and the edge point the same way.
- **Second risk: the deep anchor may be deep for the wrong reason.** $7.59M on a board
  whose favourite trades 0.991 is mostly late-stage certainty money, not price discovery.
  Anchoring the simulation to it is only valid while the board carries genuine uncertainty;
  at 0.991 it constrains little. Gate 2 should re-run the calibration using the anchor at
  T−30d, not at T−1d.
- **Third risk: `Rank Spread` may be a bootstrap artifact** rather than a usable posterior.
  Before trusting it, check it empirically: how often does a model finish outside its
  published spread one refresh later? The Wayback series answers this directly and it is a
  one-hour job.
- **Housekeeping:** Anthropic is a leg on most of these boards, and this firm's agents are
  Anthropic models. That creates no information advantage (we have no access to arena vote
  data), but Felix should know the family is one where our own maker is the dominant
  favourite, and may prefer we abstain from trading Anthropic legs specifically. Flagging
  it rather than deciding it.
