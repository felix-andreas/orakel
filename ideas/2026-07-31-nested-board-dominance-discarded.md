---
date: 2026-07-31
slug: nested-board-dominance
status: discarded-idea
example_markets: ["who-will-advance-from-the-alaska-governor-primary", "alaska-governor-election-winner", "which-candidates-will-advance-to-brazils-presidential-runoff", "next-french-presidential-election-who-will-advance-to-the-2nd-round", "alaska-at-large-primary-winners"]
model: claude-opus-5 (effort max)
---

# Same-venue nested-board dominance — coherence arbitrage between boards that never share a screen

**Verdict: discarded at idea stage.** No modelling was run and none was needed: this object
is a *dominance* claim, so it is read straight off the book. Three kills, in the order the
funnel now prescribes.

1. **Wall 3 (population), run first and free.** Across the **full 6,788-event open universe**
   there is exactly **one** rule-implied cross-board nesting (Alaska governor) and **four**
   boards carrying a hard "exactly N of these resolve Yes" constraint. 85 US races list both a
   primary and a general board, and **81 of the 85 general boards are two-leg *party* boards** —
   candidate nesting is structurally impossible on them.
2. **Wall 2 (depth), run before any modelling.** The one genuine violation I found (France,
   +23.90c at top of book) survives to **100 baskets and dies by 250**. Maximum extractable:
   **$8.88**.
3. **Wall 1 (incumbent).** Kalshi lists **11** twin series across these races. All eleven are
   **0-volume, 0-open-interest shells**. No incumbent — and the object died anyway, which is
   now the third time that has happened.

**And a fourth kill that subsumes the other three:** the one surviving arb returns **+0.35%
annualised** on capital locked to April 2027, against a ~4% short-term risk-free rate. It is
a **negative-carry** trade. A dominance arb needs n=1 to be true and still needs a hurdle rate
to be worth doing; the firm had never applied one because nothing had ever got this far.

**What graduated:** `wiki/reference/leg-sum-edge-scales-with-leg-count.md`. The mid-based
leg-sum statistic carries an artifact of exactly **K·s̄/2** (leg count × mean spread ÷ 2), so
the boards where a leg-sum edge looks biggest are precisely the boards where the fake part is
biggest. Verified to four decimals on all four boards.

---

## Thesis (as filed, before measurement)

Polymarket prices each *event* independently. Where two events are **logically nested** — the
outcome of one implies the outcome of the other — the pair carries a hard inequality that no
model is needed to evaluate:

- **Cross-board nesting.** A candidate cannot win a general election without first advancing
  from the primary, so `P(win general) ≤ P(advance from primary)`. The two boards live in
  separate events, are never shown together in the UI, and have different resolution dates.
- **Within-board Σ=N.** A board where exactly N of K legs resolve Yes must satisfy
  `Σ pᵢ = N`. Polymarket's **negRisk machinery mechanically enforces Σ=1**; it does nothing
  for **N > 1**, and all four Σ=N boards found are `negRisk: false`.

Who is on the wrong side: nobody in particular, which is the point. This is not a forecast —
it is an accounting identity the venue does not enforce. It is a **shape** claim in the sense
of `MEMORY.md`'s taxonomy, and the strongest possible form of one: it passes proxy-vs-primary
and glanceable-state by construction, and it needs **n=1**, not 91, because a dominance
violation is deterministic rather than statistical.

That last property is why this object was chosen. Wall 3 killed object 14 on draw count; a
dominance trade is immune to that wall by construction, so it was the natural next probe.

## The trade, stated exactly

**Cross-board.** Buy YES on the primary leg at ask `a_P`, buy NO on the general leg at
`1 − b_G`. Payoffs: loses primary → \$1; wins primary, loses general → \$2; wins both → \$1.
Minimum payout \$1, cost `a_P + 1 − b_G`. So the violation condition is exactly

    b_G > a_P      (general-leg best BID above primary-leg best ASK)

**Within-board Σ=N.** Buy one NO on each of K legs: cost `K − Σbᵢ`, payout `K − N`. Profit
`Σbᵢ − N`. (Payout is *at least* `K − N` when the listed set is incomplete, so the sell side
is the safe direction; buying the basket is unsafe because an unlisted winner makes the
payout smaller than N.)

## Wall 3 first — the population, counted before touching a book

Harvested the full open-event universe: **6,788 events** (offset paging caps at 2,000, so
this needed date-windowed paging — the volume-ranked 1,600-event scan finds only 4 of the 85
races, because Nov-2026 general boards are dormant and rank last on `volume24hr`).

| | count |
|---|---:|
| US races listing **both** a primary and a general board | **85** |
| …whose general board is a two-leg **party** board (no candidate legs) | **81** |
| …whose general board carries tradeable **named candidates** | **4** |
| …of those, a genuine rule-implied nesting | **1** (Alaska governor) |
| Boards venue-wide with a hard **Σ = N > 1** constraint | **4** |

The 4 candidate-level exceptions are AK-gov (22 tradeable named legs), AK-sen (5), MI-gov (1),
RI-gov (1). MI/RI are lone independents with no matching primary board. **AK-sen's pair is
spurious** and the rule text is why: the primary board resolves on *"the winner of most votes
in the final reported round"*, which under a top-4 + RCV system is **not** implied by winning
the general. So: **one race, 19 matched legs**.

Alaska is the only one because Alaska is the only state with a **top-4 jungle primary**, which
is what makes its general board multi-candidate instead of R-vs-D. That configuration recurs
on the 4-year gubernatorial cycle. Arrival rate ≈ **1 nestable race per election cycle**.

*A trap worth recording:* the party boards **do** carry legs that look like candidates —
`Will A win the FL-06 House seat?`, `Will B…` through `Will E…`. Single capital letters, not
`Person A`, so the documented placeholder filter misses them. Filtering on
`volumeNum == 0 && liquidityNum == 0` (per `recipes/polymarket-api.md`) catches them
correctly; filtering on the name pattern does not. My first census said "0 party-only boards"
and was wrong by 81.

This did **not** kill the idea on its own — n=1 suffices for a dominance trade — but it capped
the family at 5 live instances before any book was fetched.

## Wall 2 — the executable measurement

### Cross-board: Alaska governor, 19 matched legs, live books

`b_G − a_P`, sorted:

| candidate | b_G | a_P | b_G − a_P |
|---|---:|---:|---:|
| James Parkin | 0.0020 | 0.0020 | **+0.0000** |
| Bruce Walden | 0.0010 | 0.0060 | −0.0050 |
| Nancy Dahlstrom | 0.0050 | 0.0110 | −0.0060 |
| … | | | |
| Treg Taylor | 0.1270 | 0.1900 | −0.0630 |
| Bernadette Wilson | 0.2800 | 0.8400 | −0.5600 |
| Tom Begich | 0.3200 | 0.9710 | −0.6510 |

**0 violations of 19.** Median −0.0300, best exactly 0.0000, worst −0.6510. The inequality
holds everywhere, with room. General-leg bid depth is also dust — \$2–\$1,021 per leg,
median \$46 — so even a violation would have been unfillable.

### Within-board Σ=N: all four boards, both directions

| board | K | N | Σ bid | Σ mid | Σ ask | buy-all | sell-all |
|---|--:|--:|--:|--:|--:|--:|--:|
| AK Governor primary (top-4) | 20 | 4 | 3.6850 | 4.0825 | 4.4800 | **−0.4800** | **−0.3150** |
| AK At-Large primary (top-4) | 6 | 4 | 3.1650 | 3.5915 | 4.0180 | **−0.0180** | **−0.8350** |
| Brazil presidential runoff (top-2) | 9 | 2 | 1.9540 | 2.0280 | 2.1020 | **−0.1020** | **−0.0460** |
| France 2nd round (top-2) | 37 | 2 | 2.2410 | 2.6140 | 2.9870 | −0.9870 | **+0.2410** |

Three of four: **both directions lose simultaneously**, which per
`reference/midpoint-is-not-a-fill.md` means the thing being measured is the spread. On the AK
governor board Σask − Σbid = 0.795 over 20 legs = a **3.98c mean spread**, and the apparent
+8.25c overround at mid is exactly half of it.

The fourth is the interesting one, and it is the first executable dominance violation the firm
has ever measured: **selling the 36-leg French basket at top of book yields +23.90c on a
\$33.76 basket, guaranteed.** (Rule-checked: "resolve Yes if the listed candidate advances to
the second round **or wins outright in the first round**", and if no election occurs by
2027-12-31 every leg resolves No — a tail that pays the *seller* more, not less.)

### The depth walk that kills it

Buying one NO on each of the 36 named legs, VWAP through the book, fee = 0.04·p(1−p) per leg
(politics, taker, paid once on a hold to resolution):

| baskets | cost/basket | fee | net profit/basket | capital | return | annualised (9mo) |
|--:|--:|--:|--:|--:|--:|--:|
| 100 | 33.8661 | 0.0452 | **+0.0888** | \$3,387 | +0.262% | **+0.35%** |
| 250 | 33.9657 | 0.0423 | −0.0080 | \$8,491 | −0.024% | −0.03% |
| 500 | 34.0787 | 0.0393 | −0.1180 | \$17,039 | −0.346% | −0.46% |
| 1,000 | 34.3149 | 0.0335 | −0.3484 | \$34,315 | −1.015% | −1.35% |

The binding legs are the small ones — Glucksmann 1,244 shares at the NO ask, Tondelier 1,308,
Attal 1,352 — not the \$75k Le Pen leg. Same anti-correlation as
`depth-lives-where-the-edge-is-not.md`, in a new place: on a basket trade the **thinnest** leg
sets the size for **all 36**.

**Total extractable: \$8.88.**

## Wall 1 — the incumbent screen

Kalshi catalogue in one unauthenticated call: **12,355 series** (12,329 on 07-30, 12,298 on
07-29 — the +26/day drift continues). Searched by *jurisdiction*, not by object name, per my
own 07-30 near-miss, and then re-checked the vendor-generic tickers underneath:

| ticker | title | markets | volume | OI |
|---|---|--:|--:|--:|
| `KXGOVAK` | Alaska governor | 16 | **0** | **0** |
| `KXGOVPARTYAK` | Alaska Governor (party) | 2 | 0 | 0 |
| `KXBRPRESADVANCE` | Who will advance in the Brazilian presidential election? | 6 | 0 | 0 |
| `KXBRPRES` / `KXBRBALLOT` / `KXBRPRES1R` / `KXBRPRES1MOV` / `KXBRDEP` | Brazil presidential/ballot/deputies | 57 | 0 | 0 |
| `KXAKSENATE` / `SENATEAK` / `KXAKMOV` / `KXAKSENGOVCOMBO` | Alaska senate/MOV/combo | 24 | 0 | 0 |

**Every one is a shell**, and their `close_time`s are a full year *after* the elections
(2027-11-03 for a 2026-11-03 race), which is the signature of auto-listing. This is the exact
mirror of 07-30, where object-name search found empty stubs while vendor-generic series
carried 3.3M contracts: today the generic tickers are empty too. **No incumbent found** — the
fourth time (objects 1, 4, 9, 12) — and the third time the object died anyway.

The real counterparty is the one object 6 named: **the market itself**. Three of four boards
are coherent to within their own spread, and the fourth is coherent to within its own depth.

## The kill that actually decides it: carry

Even taking the +\$8.88 at face value, the capital is locked until the French first round
(April 2027, and to 2027-12-31 in the no-election branch) — **~9 months for +0.262%**, i.e.
**+0.35% annualised**. Against a ~4% short-term risk-free rate that is roughly **−3.7pp/yr**.

This is a new gate and it belongs in the sequence. Every previous object was a *statistical*
claim, where the question was "is the edge real", and a hurdle rate never came up. A dominance
arb is real by construction, so the only remaining questions are size and time — and those
are the two the funnel had never had to ask. **A guaranteed profit is not an edge until it
clears the risk-free rate on the capital it locks.**

*(Checked and withdrawn: I assumed Polymarket's `Earn 4%` tag meant a venue-paid yield on idle
USDC, which would have made the comparison internal. It does not — those tags are
`Rewards Automation …`, the maker liquidity-rewards programme. The comparator is external.)*

## Two standing constraints, checked explicitly

- **Session calendar (kill condition, not a caveat).** None of these boards resolves on a feed
  with trading hours. Alaska: *"the Associated Press, Fox News, and NBC… else official
  certification"*. Brazil: the **TSE** (`dadosabertos.tse.jus.br`). France: the **Ministère de
  l'Intérieur**. Election returns are published once, on a known date, with no session
  calendar — so `stale-feed-gate.md` does not bite and neither does the Pyth-RTH overlap
  problem. The underlyings are elections, not quoted numbers, so Felix's standing instruction
  is satisfied too.
- **No lifetime-volume filter.** `lifetime-volume-is-look-ahead.md` forbids gating on realised
  volume. Nothing here does: `volumeNum == 0 && liquidityNum == 0` is used **only** to identify
  legs that have no book at all — the documented neg-risk placeholder filter from
  `recipes/polymarket-api.md` — and this screen reads a **live** book with no settled outcome
  in sight, so look-ahead is not available to it in either direction.

## Falsification sketch (what would have kept it alive)

Pre-registered before the books were pulled:

- **Kept alive if** ≥1 cross-board pair showed `b_G > a_P` by more than the two-leg fee, at
  ≥\$500 of walkable depth. → **0 of 19**, best +0.0000.
- **Kept alive if** any Σ=N board's executable basket cleared **`|Σmid − N| > K·s̄/2 + fees`**
  by a margin that annualised above the risk-free rate over the lock-up. → **1 of 4 cleared
  the arithmetic, 0 of 4 cleared the hurdle.**
- **Killed if** the population of nested pairs could not exceed ~5 live instances, since a
  scan-and-take rule needs opportunities, not evidence. → **5 instances; 1 cross-board
  nesting, 4 Σ=N boards.**

## What to reuse, and what not to re-derive

- **Do not re-propose** primary/general dominance pairs, Σ=N basket coherence, or cross-board
  leg-sum arbitrage on Polymarket. The population is one race and four boards, and the venue
  is coherent to within its own spread on all of them.
- **Reusable, and it is the day's real product:** `leg-sum-edge-scales-with-leg-count.md`. The
  gate is arithmetic — `real edge = |Σmid − N| − K·s̄/2` — and it needs no backtest, no
  history, and one book fetch. It reproduced the executable number to four decimals on four
  independent boards.
- **Reusable:** the placeholder trap. `Will A win…` / `Will B win…` are placeholders that the
  `Person [A-Z]` filter misses; gate on `volumeNum == 0 && liquidityNum == 0`, never on the
  name pattern.
- **Reusable:** dormant boards rank last by `volume24hr`, so a volume-ranked scan finds 4 of
  85 races. Any *population* count must page the full universe — and offset paging caps at
  2,000, so date-windowed paging is mandatory (`recipes/polymarket-api.md`).
