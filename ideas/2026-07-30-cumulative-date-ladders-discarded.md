---
date: 2026-07-30
slug: cumulative-date-ladders
status: discarded-idea
example_markets: ["gpt-6-released-by", "will-samuel-alito-announce-his-retirement-by", "next-mythos-class-model-released-by", "anthropic-ipo-by", "will-arc-launch-a-token-by"]
model: claude-opus-5 (effort max)
---

# Cumulative "by &lt;date&gt;" ladders — the hazard term structure

**Verdict: discarded at idea stage.** Two independent kills, either sufficient:

1. **Incumbent.** Kalshi runs the *identical* cumulative "Before &lt;date&gt;" ladder on these
   objects with 0.5–2.2M contracts per series, and where the two contracts genuinely match the
   venues agree to a **median |Δ| of 0.00pp** (Alito) and **1.50pp** (Mythos-class). The one
   object with a 16pp gap (GPT-6) has **materially different resolution rules in exactly the
   direction of the gap** — that is definitional basis, not mispricing, and it is not
   arbitrageable.
2. **Power.** The pre-registered shape test came back in the predicted direction —
   nearest rung **−11.12pp, t=−2.15**, decaying monotonically to −0.46pp at the farthest — and
   the mirror test did **not** fire. It dies anyway on the bound: the realised YES rate on the
   nearest rung is 5/29, **Wilson 95% upper 0.3455 against a break-even of 0.2786**, so at
   *zero fee* I cannot reject that the price is fair. Settling it needs **91 independent
   events**; the family produces **0.88 per month**, i.e. **5.9 years**.

**The one genuine positive, and it is worth more than the kill:** this is the **first family to
break the depth anti-correlation** of `depth-lives-where-the-edge-is-not.md`. A cumulative
ladder has **no mode** — price is monotone in date — so the unquoted legs are the
*already-decided* ones (p&lt;0.05, p&gt;0.95), not the ones a rule would trade. See the new page
`nested-ladders-trade-depth-for-power.md`: the same nesting that gives this family good depth is
what destroys its sample size, and that trade-off looks general.

---

## Thesis (as filed, before measurement)

A cumulative date ladder quotes P(event by d₁) ≤ P(event by d₂) ≤ … on one underlying news
event. The legs are **nested, not mutually exclusive**, so the board is a full implied hazard
curve, and the marginal bet is a conditional hazard over one interval.

**Claimed edge mechanism — a SHAPE claim, not a level claim.** The crowd's *total* probability
may be right while its *allocation across dates* is wrong: attention concentrates on the near
term ("it's coming any week now"), so the crowd overweights early intervals and underweights the
long tail. Precisely: mean(realised − price) &lt; 0 on the nearest rung, → 0 on the farthest.

**Who is on the wrong side.** Whoever buys the near rung because a release/announcement "feels
imminent" — the same participant who makes launch-date markets a byword for slippage.

**Why I picked this object against the funnel.** Rows 12 and 13 died to execution, and
`depth-lives-where-the-edge-is-not.md` explains why: on a *bucket* ladder depth concentrates at
the mode and mispricing lives in the wings, anti-correlated by construction. A **cumulative**
ladder has no mode. Its price is monotone in date, so there is no single "most likely bucket"
hoarding the depth, and the legs a hazard-shape rule trades are the mid-probability ones — where
the flow is. That was the structural reason to expect this family to clear wall 2, and **it
did.**

## Screen order — measured, in the order `market-selection.md` demands

### 0. Family size (Gamma, `/events` + `public-search` unioned over 30 wordings)

**219 live cumulative by-date ladders**, of which **90 non-war**; 96 fully settled. Excluding
war (blocked pending Felix's ruling) and quoted-price underlyings (Felix's standing
instruction — Bitcoin/gold/valuation "hit __ by" ladders), **63 settled boards** remain, of
which **29** have ≥3 legs alive at a common checkpoint.

Live non-war examples today: `gpt-6-released-by` ($779k), `anthropic-ipo-by` ($856k),
`will-samuel-alito-announce-his-retirement-by` ($3.22M), `next-mythos-class-model-released-by`
($84k), `will-arc-launch-a-token-by` ($377k).

### 1. Kalshi catalogue — 12,329 series today (12,298 on 07-29), one unauthenticated call

**The incumbent screen fired, and I nearly missed it.** The object-specific tickers are all
**0-market shells**: `KXGPT5RELEASE`, `KXGEMINI3`, `KXMYTHOS`, `KXCLAUDE4`, `KXCLAUDE5`,
`KXO3RELEASE`, `KXDEEPSEEKV4RELEASE`, `KXDEEPSEEKR2RELEASE`, `KXGROK4`. Stopping there would
have produced "no venue incumbent on AI release dates" — **false by 3.3M contracts.** The
volume is on the **vendor-generic** tickers:

| Kalshi series | title | markets | live | volume (contracts) |
|---|---|---:|---:|---:|
| `KXCLAUDE` | Claude Model Release | 24 | 10 | **2,158,541** |
| `KXIPOOPENAI` | When will OpenAI announce IPO? | 13 | 11 | **1,146,482** |
| `KXGPT` | ChatGPT Release Date | 21 | 5 | **1,052,027** |
| `KXALITOANNOUNCERETIRE` | Alito announces retirement | 7 | 4 | **492,347** (317,434 OI) |
| `KXGEMINI` | Gemini release date | 8 | 3 | 93,271 |
| `KXSTRIPEIPO`, `KXIPOBLOOMBERG`, `KXIPOSPACEX`, `KXALIENS` | date ladders | 13/7/13/9 | 11/7/0/8 | — |

Generalised into `sharp-line-screen.md`: **a shell under the object-specific ticker is not
evidence of no incumbent — check the vendor-generic ticker.**

Other candidates measured: **Manifold** covers the family heavily (767k volume / 495 traders on
one Claude-release market). **Metaculus' API is now authenticated-only** ("Permission Error: the
API is only available to authenticated users") — recorded as a measured limitation, not an
unmeasured incumbent; the verdict does not turn on it, since it could only add to the kill.

### 2. Cross-venue matched rungs — the deciding incumbent number

Rungs matched on the **calendar instant**, never the label (Kalshi "before D" ≡ Polymarket "by
D−1", both 11:59pm ET), and both sides required to have a real two-sided book (*an unpriced leg
does not vote*).

| object | rungs | mean Δ (Poly − Kalshi) | median \|Δ\| | rules match? |
|---|---:|---:|---:|---|
| Samuel Alito retirement | 3 | **−0.50pp** | **0.00pp** | yes |
| Next Mythos-class model | 3 | **+0.05pp** | **1.50pp** | yes |
| GPT-6 release | 4 | **+10.31pp** | 12.25pp | **no** |

Overall: **6 of 10 matched rungs within 3pp**, and all four exceptions are GPT-6.

Alito, rung by rung: 0.045/0.045, 0.135/0.150, 0.445/0.445. Two venues, disjoint crowds,
317k contracts of Kalshi open interest — **the same hazard curve to a cent and a half.** There
is no term-structure mispricing here to harvest.

**And the one big gap is a different contract, not a mispriced one.** GPT-6:

| deadline | Polymarket mid | Kalshi mid | Δ |
|---|---:|---:|---:|
| 2026-07-31 | 0.003 | 0.005 | −0.25pp |
| 2026-08-31 | 0.320 | 0.235 | **+8.50pp** |
| 2026-09-30 | 0.710 | 0.540 | **+17.00pp** |
| 2026-12-31 | 0.885 | 0.725 | **+16.00pp** |

Kalshi: *"If OpenAI releases a model **called GPT-6 or greater**."* Polymarket: *"a product
explicitly named GPT-6 … **or one that is recognized as a successor to GPT-5**. Products labeled
GPT-5.5 or similar will not count."* Polymarket's clause is strictly broader — it admits a
successor shipped under a different name — and OpenAI is currently on **GPT-5.6**, so a
non-"GPT-6" flagship is entirely live. Third opinion: **Manifold's** separately-worded "marketed
as GPT-6 by Aug 31" prices **0.219**, sitting next to Kalshi's 0.235, not Polymarket's 0.320.

The apparent trade — buy NO on Polymarket at 0.300, buy YES on Kalshi at 0.550, outlay 0.850 for
a package "paying" 1.00 — is **not an arb**. In the state *"a GPT-5 successor ships under a
non-GPT-6 name"* both legs lose. So the 15c is the market's price for that state, not a free
lunch. This is object 12's signature in a new guise, and it is now a wiki page: **if both sides
can lose, you measured the definition.** Every prior cross-venue screen we ran (chokepoint
transit counts, Tomatometer scores, tennis games, post counts) compared contracts settling on a
**shared external scalar**; a news-resolved date market has no such anchor.

Also structural: a cross-venue expression can never be *our* strategy — we hold no Kalshi
account and `CONSTITUTION.md` §5 forbids execution. Cross-venue prices are only ever *evidence*
about Polymarket's line, and here that evidence is contaminated.

### 3. Leg-level depth walk — run BEFORE the modelling, per the 07-29 rule

32 legs across five live non-war boards, one `/book` call each, walking the book at $100/$500/$2,000:

| leg | mid | $ at ask | $2,000 ask VWAP |
|---|---:|---:|---:|
| Anthropic IPO by Sep 30 | 0.060 | 1,238 | 0.073 (+1.3c) |
| Anthropic IPO by Dec 31 | 0.705 | 50 | 0.720 (+1.0c) |
| Anthropic IPO by Oct 31 | 0.355 | 325 | 0.373 (+1.3c) |
| GPT-6 by Dec 31 | 0.885 | 1,107 | 0.894 (+0.4c) |
| GPT-6 by Sep 30 | 0.710 | 613 | 0.740 (+2.0c) |
| Alito by Dec 31 | 0.135 | 701 | 0.146 (+0.6c) |

11 of 32 legs have no two-sided book — and **those are the already-decided legs**
(GPT-6 by Mar 31, Arc by Dec 2025, Alito by Feb/Mar) plus the sub-1c rungs. In the tradeable
band (mid 0.15–0.85, n=10): **median spread 2.0c, median $47 at the best ask, median $264 at the
best bid**, max $613 at the ask and ~$3,000 at the bid.

Two honest qualifications. (a) A median of $47 at the ask is *better* than the post-count wings'
**$7** but is not itself generous — call this a **partial** pass. (b) The rule here **shorts the
near rung**, i.e. buys NO, which consumes the **YES bid** side: median **$264**, which is the
number that binds. The structural claim — dead legs are the decided ones, not the edge ones — is
what holds cleanly, and it is the reusable part.

### 4. Pre-registered backtest — is the crowd's hazard shape wrong?

Both venues agreeing does not make either right (funnel row 6 needed the empirical check; row 4
found the crowd wrong by 16.8pp). So this had to be measured, and it is the test that could have
flipped the verdict.

Two feed facts were **measured, not assumed**, and either would have produced a wrong verdict:

- The per-leg deadline is Gamma **`endDate`**, not the question text. Many settled boards omit the
  year; a wrong year silently moves the checkpoint outside the board's life.
- CLOB `prices-history` returns an **empty array — not a truncated one — for windows wider than
  ~14 days** (168h→168 pts, 336h→336 pts, **504h→0 pts**, at every fidelity). My first pass
  scored **0 of 125 legs** and looked exactly like "this family has no price history, therefore
  unbacktestable." That would have been a *wrong kill*. Chunk at ≤14 days.

**Design (pre-registered in code before running).** Cumulative ladders share a birth and a
death — every leg is born with the board and they **all resolve at the instant the event
happens** — so there is no per-leg "H days before its own deadline"; the checkpoint must be one
common instant per event. t₀ = earliest leg `startDate` + 7d; score legs with `startDate ≤ t₀`
and `closedTime > t₀`; SE **clustered by event**; mirror test at mid ± 0.5c (the measured median
half-spread); kill on the mirror firing, on the bound failing break-even, or on &lt;20 events.

**Result — 125 legs over 29 events. The shape test came back in the predicted direction.**

| rung | n | mean mid | realised YES | mean(outcome − mid) | t |
|---|---:|---:|---:|---:|---:|
| **rank 1 (nearest)** | 29 | 0.284 | **5/29** | **−11.12pp** | **−2.15** |
| rank 2 | 29 | 0.477 | 11/29 | −9.79pp | −1.31 |
| rank 3 | 29 | 0.582 | 16/29 | −3.04pp | −0.44 |
| last (farthest) | 29 | 0.625 | 18/29 | −0.46pp | −0.06 |
| all legs | 125 | 0.481 | 50/125 | −6.67pp | −1.61 |

A monotone gradient, largest at the near rung, vanishing at the far one — exactly the
pre-registered prediction. **The mirror test does not fire on rank 1**: buy YES −11.62pp, buy NO
**+10.62pp**. Only one side loses, so this is not the spread. Unlike 07-28, the number survived
that check.

### 5. …and dies on the bound, which is what actually decides it

`break-even-win-rate.md` exists for exactly this. The trade is *buy NO on the nearest rung*, so
break-even needs the true YES rate below mid − half-spread = **0.2786**. Realised **5/29 =
0.1724**, but the **Wilson 95% interval is [0.0760, 0.3455]**:

**Wilson upper 0.3455 > break-even 0.2786 — at zero fee.** With 29 independent events I cannot
reject that 28.4c is the fair price. Every rank fails identically (rank 2: 0.5600 vs 0.4723;
last: 0.7731 vs 0.6203; pooled: 0.4876 vs 0.4757). The apparent +10.62pp is a point estimate
whose confidence interval comfortably contains zero edge.

Two supporting decompositions, per *"always decompose a surviving band by leg TYPE"*:

- **Sub-family cells are tiny and inconsistent.** rocket launch/flight test n=8 (−15.4pp), AI
  model release n=7 (−7.1pp), political personnel n=4 (−17.6pp), **IPO n=2 (+24.3pp, opposite
  sign)**, token launch n=2, wildfire n=1.
- **Leave-one-family-out is not robust.** Dropping rocket launches takes t from −2.15 to
  **−1.45** — and "launch dates slip" is the single most glanceable fact in that domain, so the
  largest contributor is also the least likely to be un-priced.

### 6. Why more data will not arrive

The 29 usable events settled between **2023-10-25 and 2026-07-29** — 33.1 months, **0.88 usable
independent events per month**. To reach the 91 events an 11pp deficit at p=0.28 needs
(one-sided α=0.05, 80% power; 117 two-sided) is **70 more months ≈ 5.9 years**.

This is the real finding, and it is *not* a liquidity or an incumbent problem. **A 12-leg
cumulative ladder is one observation, not twelve** — the legs are perfectly nested and they
resolve simultaneously. 219 live boards and 96 settled ones present as a large family and yield
29 independent draws.

## Falsification sketch — what was pre-registered, and what fired

| pre-registered kill | fired? |
|---|---|
| A venue runs the identical ladder with a line we cannot beat | **YES** — Kalshi, 0.00pp / 1.50pp median \|Δ\| on matched contracts |
| Mirror test: both directions lose at executable prices | no — buy NO +10.62pp on rank 1 |
| Shape gradient absent or reversed | no — monotone, in the predicted direction |
| Wilson bound on the realised rate fails break-even | **YES** — 0.3455 vs 0.2786, at zero fee |
| Fewer than 20 independent settled events | no — 29 (but 91 needed) |
| Leg-level depth cannot absorb the intended size | no — **first family to pass**, partially |
| Resolution feed has session hours | no — UMA/news, no session calendar |

**What would revive it.** Only a construction that recovers independence — one event, one
observation, so the object would have to be a *high-frequency recurring* by-date ladder (the
UpOnly-podcast-episode board is the only instance in the universe with that shape, and it is a
$711k one-off). Not the AI-release boards, whose whole appeal was volume.

## For the funnel (CEO to append — I do not touch `ops/`)

| # | date | object | outcome | screen that decided it | counterparty |
|---|---|---|---|---|---|
| 14 | 07-30 | Cumulative "by &lt;date&gt;" ladders (hazard term structure) | discarded at idea stage | **two:** matched-rung cross-venue agreement (median \|Δ\| 0.00/1.50pp) **and** Wilson upper 0.3455 vs break-even 0.2786 on n=29 | **Kalshi `KXGPT`/`KXCLAUDE`/`KXALITOANNOUNCERETIRE`** (0.5–2.2M contracts/series), plus Manifold |

Note for the funnel's wall analysis: this row is the **first to clear wall 2 (execution) and
die at a third wall — statistical power**. And the two are structurally linked, which is the
generalisation worth carrying: see `wiki/reference/nested-ladders-trade-depth-for-power.md`.
