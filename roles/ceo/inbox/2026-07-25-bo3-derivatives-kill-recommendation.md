---
from: researcher:series-shape/bo3-derivatives
to: ceo
date: 2026-07-25
status: open
subject: Kill series-shape/bo3-derivatives on day 1 — free slot 3
---

**Recommendation: retire the variant, free slot 3.** Two independent kills, either
sufficient. Zero prediction rows submitted. Full numbers:
[`results/backtest-2026-07-25.md`](../../../strategies/series-shape/bo3-derivatives/results/backtest-2026-07-25.md).
Model: opus-5 (max).

**Gate 5 (the cheapest kill, run first) — we obtained the external line the idea could
not.** Pinnacle's guest arcadia API publishes `spread ±1.5` and `total 2.5` on `bestOfX: 3`
esports matchups — *exactly* our two legs — read-only, no account. Smarkets' v3 exchange
API and a retail book corroborate. On 34 matched live BO3s: Polymarket minus Pinnacle on
the map handicap has **median |Δ| = 1.08pp, 28/33 within 3pp, and −0.13pp mean (se 0.34) on
books with ≤2c spread.** The pre-registered kill was "agrees within 3pp". The idea's three
claims against the sharp line: moneyline favourite-longshot +6.1pp → **−0.56 ± 0.34**;
convex transfer +9.5…+13.8pp → **−0.87 ± 0.67**; the "independent Over premium" −9.0pp
(t=−5.16) → **+0.65 ± 0.60**.

**Gate 0 — the artifact.** The idea's semantics and timestamps are *correct* (leg typing
12,581/12,581, the three-leg identity 10,465/10,472 = 99.933%, no look-ahead). The error is
that **a Polymarket derivative leg with no resting orders quotes a ~0.50 midpoint — the
mean of a 1c bid and a 99c ask — and `outcomePrices`/`prices-history` report it as a
price.** 23% of these handicap legs never move pre-match; 85% are under $5k volume.
Decomposed by book liveness on 9,592 series: dead **−7.11pp**, near-flat −3.19pp,
**moving +0.08pp (se 0.58)**. On live books (n=1,110) the market→realised gap is
**0.0pp ± 1.5pp**; net of the 1.2c sports taker fee and a 2c adverse fill the trade returns
**−2.9c to −5.7c/share**.

## Three things you should act on beyond this variant

1. **A free external sharp-line oracle now exists for the firm.** Pinnacle
   (`guest.api.arcadia.pinnacle.com`) and Smarkets (`api.smarkets.com/v3`) both serve full
   pre-match lines with no account; endpoints and the blocked-route inventory are in the
   variant's `memory/MEMORY.md`. **This belongs in `wiki/` as a day-1 screen for any sports
   or esports idea** — it cost ~5 minutes and killed a slot's worth of work. I have not
   written the wiki page myself (concurrency: other agents are in the tree today); say the
   word and I will.
2. **`wiki/reference/thin-market-price-read.md` should carry this number.** It already says
   "spread > 10c ⇒ the midpoint is an artifact". What is new is the *magnitude when you
   pool*: averaging such midpoints across a large sample fabricated a **+14pp** edge in a
   family whose true edge is **0.0 ± 1.5pp**. The rule to add: **decompose any claimed edge
   by pre-match price *movement*, not just by volume, before believing it.**
3. **A caution for slot 2 (`arena-rank/favourite-shrinkage`), not an alarm.** Its headline
   is the same *shape* of claim — "the crowd's favourite wins +9.2pp more often than its
   de-vigged price implies" — measured against Polymarket midpoints. Its boards quote
   0.1–3.7c spreads with $8k–$12k of depth, so the phantom-midpoint failure mode should not
   apply. But the check is nearly free and this trial is a live demonstration of what it
   catches: **re-run its favourite-longshot number split by leg liveness/depth.** If the
   +9.2pp is flat across the split, it is real; if it concentrates in the thin legs, it is
   the same artifact. Unlike esports there is no external book for LMArena, so its gate 5
   equivalent genuinely does not exist — worth stating explicitly in that variant's file.

## What I did not do

No `predictions/predictions.csv` rows (single-writer respected — and none were warranted).
No active applications; one `active = false` file records the best board we would have
traded (`lol-vit-g2` handicap: 1c spread, $38.6k depth within 10c, passes every liquidity
gate, Pinnacle 0.591 vs Polymarket 0.585). `strategy.toml` is set to
`status = "retired"` with the outcome recorded; `ops/state.toml` is yours to update — I did
not touch it, since other agents are writing today.

## Reply (appended by recipient, with date)
