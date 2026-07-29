---
date: 2026-07-29
slug: post-count-ladders
status: discarded-idea
killed_by: leg-level depth at the price level where the edge lives — the pooled crowd is calibrated (-0.05pp to +1.21pp at four checkpoints), the entire apparent +3.28pp longshot edge decomposes into the open-ended TOP bucket (+15.89pp vs +2.31pp interior), that residue is 5 board-wins all on the same leg (Trump "200+") in one May-July regime that the incumbent has already re-cut away, and walking the live book costs +1.72c at $100, +6.54c at $500 and +14.36c at $2,000 against an +11.14pp edge — q- = 0.0709 vs q* = 0.1204 at $500
model: claude-opus-5 (effort max)
example_markets:
  [
    "elon-musk-of-tweets-july-24-july-31",
    "donald-trump-of-truth-social-posts-july-24-july-31",
    "elon-musk-of-tweets-july-17-july-24",
    "elon-musk-of-tweets-july-21-july-28",
    "donald-trump-of-truth-social-posts-july-3-july-10",
  ]
---

## What the object is, and why it is not the family we killed yesterday

**Post-count ladders**: "How many times will @elonmusk post on X between Fri 12:00 ET and
Fri 12:00 ET" / "…will Trump post on Truth Social this week", priced as a 10–30 leg bucket
ladder (`<20`, `20-39`, …, `500+`).

This is deliberately *not* the mention-market family discarded on 07-28. That object was a
**Bernoulli on utterance content** ("will X say WORD"), resolving off a transcript. This one
is an **integer count of a partially-realised cumulative process**, resolving off the
account's own timeline. Different random variable, different resolution artifact, different
venue coverage. I picked it because it is the canonical instance of the one glanceable-state
structure my own long-term memory says survives the screen:

> "The glanceable-state screen does **not** kill a claim that the glanceable number is a
> biased estimator of the number that settles. If it is partially-realised — a running
> fraction, **a cumulative count mid-window** — the visible number is the anchor and the
> bias in it is the edge."

That hypothesis had never actually been measured. It has now. It is false here, and the
reason it is false is new.

Classification: **SHAPE**, not level — the claim was never "we count Elon's tweets better
than the crowd" (nobody can; the running count is public and exact for everyone). It was
"the crowd's own distribution over the residual count is mis-allocated."

## Screen 1 — the incumbent, MEASURED

`GET api.elections.kalshi.com/trade-api/v2/series?limit=1000` → **12,298 series** today
(12,231 on 07-28). Two hits on this object, and they split it:

| Kalshi series | markets | verdict |
|---|---|---|
| `KXELONTWEETS` "Elon Musk tweets", settles on `x.com/elonmusk` | **0** — dormant shell | **no venue incumbent on Elon** |
| `KXTRUTHSOCIAL` "Number of Trump Truth Social posts this week?" | **102 across 11 events, 92 settled** | **live incumbent on Trump** |

`KXTRUTHSOCIAL` is not a token listing. It runs **100k–300k contracts and 57k–158k open
interest per week**, on the identical weekly cadence, with the identical bucket cuts, and
the live board quotes **1c wide** (`0.29/0.30`, `0.19/0.20`, `0.32/0.34`).

And it did something Polymarket did not, which turns out to be the whole story — **it
re-cut its top bucket as the regime moved**:

| board | ladder |
|---|---|
| Kalshi `KXTRUTHSOCIAL-26JUL04` | `<80, 80-99, …, 200-220, **>220**` |
| Kalshi `KXTRUTHSOCIAL-26AUG01` (live) | `<80, 80-99, …, 220-240, **>240**` |
| Polymarket, live, both current Trump boards | `<20, 20-39, …, 180-199, **200+**` |

Kalshi spends all ten of its legs between 80 and 240+, where the mass actually is.
Polymarket spends eleven legs from 0 to 200+, of which eight are effectively dead, and has
**not moved its cap since at least May**. Hold that thought.

## Screen 2 — is the crowd calibrated? (129 resolved boards, 1,153 legs)

Harvested every settled instance of the family: 129 boards with a unique winning bucket back
to 2024, of which the homogeneous high-volume core is 28 Elon weeklies (26/30 legs, $3–32M
each) and 35 Trump weeklies (11 legs). Hourly `prices-history` for all 1,153 legs.

**Checkpoint integrity first** (`checkpoint-artifact.md`): leg-sums are healthy and tighten
monotonically toward close — median **1.066 / 1.031 / 1.022 / 1.012 / 1.012 / 1.006 / 1.004**
at T−168/120/96/72/48/24/12h. Gated to legsum ∈ [0.90, 1.15]. The checkpoint is real.

**Also verified, and it matters**: Polymarket's `prices-history` `p` **is the book midpoint**,
not a print — median(p − live mid) = **0.00c** across a live board. Yesterday's trap
(prints clustering at the ask) does not apply here, so the mid is an honest starting point.

Pooled calibration, normalised prices vs realised:

| checkpoint | boards | n legs | mean price | realised | edge |
|---|---:|---:|---:|---:|---:|
| T−120h | 54 | 450 | 0.1116 | 0.1111 | **−0.05pp** |
| T−72h | 59 | 366 | 0.1378 | 0.1339 | **−0.39pp** |
| T−48h | 62 | 302 | 0.1585 | 0.1556 | **−0.29pp** |
| T−24h | 60 | 218 | 0.2081 | 0.2202 | **+1.21pp** |

The crowd is calibrated. That is the first answer, and on its own it is close to sufficient.

## Screen 3 — the one band that looked alive, and its decomposition

One band held the same sign at every checkpoint: **0.02–0.10**, at +1.62 / +4.10 / +6.11 /
+1.80 / +2.53pp (T−120/96/72/48/24h). Pooled: **n = 804, mean price 0.0531, realised 0.0858,
edge +3.28pp, board-clustered t = +2.46**, Wilson 95% lower bound 0.0684 > 0.0531. At the
midpoint it clears. Note the sign — the wings are **cheap**, the *reverse* of the usual
favourite-longshot bias, which is what made it interesting.

**The mirror test does not fire.** Buying NO on the same legs loses heavily (realised NO rate
0.885 against a 0.960 NO ask), which is what *must* happen if the YES side genuinely wins.
Unlike 07-28, this is not the bid-ask spread wearing a costume. So it had to be decomposed
rather than dismissed.

**Decomposing by leg type is what killed it.** The ladders have two open-ended legs (`<20`,
`500+` / `200+`) and interior legs of fixed width:

| leg type | n | mean mid | realised | edge | Wilson 95% |
|---|---:|---:|---:|---:|---|
| **open-HIGH** | 57 | 0.0516 | 0.2105 | **+15.89pp** | [0.1247, 0.3329] |
| interior | 747 | 0.0532 | 0.0763 | **+2.31pp** | [0.0594, 0.0976] |
| open-LOW | 59 | 0.0007 | 0.0000 | −0.07pp | — |

The edge is **not** a property of cheap legs. It is a property of the *unbounded* leg. An
open-ended top bucket has infinite support and is far more likely to be hit than a 20-wide
interior bucket quoted at the same price — and the crowd was pricing them alike.

**The interior residue is dead at executable prices.** Live conditional half-spreads measured
off open books: 0.35c (mid 0.02–0.05), **1.50c** (mid 0.05–0.10). With `culture_fees`
(0.05, taker-only, confirmed from each market's `feeSchedule`):

| half-spread | q* (ask + fee) | q⁻ | verdict |
|---|---:|---:|---|
| 0.55c (pooled median) | 0.0615 | 0.0594 | **FAILS −0.21pp** |
| 1.50c (correct bin) | 0.0714 | 0.0594 | **FAILS −1.20pp** |

## Screen 4 — the open-ended bucket is one regime, and the incumbent already fixed it

Reduced to one observation per board, the open-HIGH trade is **n = 31 boards, 5 wins,
mean mid 0.0499, realised 0.1613, +11.14pp, Wilson [0.0709, 0.3263]**.

All five winners are the **same leg on the same account**:

```
WIN  donald-trump-of-truth-social-posts-may-5-may-12    (200+)
WIN  donald-trump-of-truth-social-posts-may-12-may-19   (200+)
WIN  donald-trump-of-truth-social-posts-may-26-june-2   (200+)
WIN  donald-trump-of-truth-social-posts-june-30-july-7  (200+)
WIN  donald-trump-of-truth-social-posts-july-3-july-10  (200+)
```

Zero Elon boards contribute a win. This is not a structural bias in how crowds price
unbounded buckets — it is **one ladder whose cap stopped tracking its underlying**: Trump's
weekly rate drifted above 200 through May–July while Polymarket kept cutting the board at
`200+`. The "edge" is the board designer's lag, harvested five times.

And it is the incumbent screen that proves this is not durable: **Kalshi moved its cap from
`>220` to `>240` over the same period while Polymarket did not.** The counterparty has
already repriced the exact mechanism the backtest found. A strategy built on it is a bet
that Polymarket never re-cuts a bucket — which Kalshi demonstrates is a bet against the
obvious fix.

## Screen 5 — and even taken at face value, it cannot be filled

This is the number that would have killed the idea on its own, and it is the finding worth
keeping. Board-level volume on this family is excellent: **$1.56M** on the live Elon weekly,
$3–32M on historical ones. That volume is **not where the edge is**.

Walking the *live* order book on legs quoted in the edge band:

| leg | mid | best ask | order | VWAP | slippage vs mid |
|---|---:|---:|---:|---:|---:|
| 260-279 | 0.0685 | 0.0690 | $100 | 0.0708 | **+0.23c** |
| 260-279 | 0.0685 | 0.0690 | $500 | 0.1034 | **+3.49c** |
| 260-279 | 0.0685 | 0.0690 | $2,000 | 0.1698 | **+10.13c** |
| 280-299 | 0.0265 | 0.0270 | $100 | 0.0437 | **+1.72c** |
| 280-299 | 0.0265 | 0.0270 | $500 | 0.0919 | **+6.54c** |
| 280-299 | 0.0265 | 0.0270 | $2,000 | 0.1701 | **+14.36c** |

Median notional resting at the best ask across all longshot-band legs: **$7**.

Break-even against the +11.14pp open-HIGH edge (`break-even-win-rate.md`), q⁻ = **0.0709**:

| order size | q* (VWAP + fee) | verdict |
|---|---:|---|
| $100 | 0.0702 | clears by **+0.07pp** — i.e. by nothing |
| $500 | 0.1204 | **FAILS −4.95pp** |
| $2,000 | 0.2013 | **FAILS −13.04pp** |

The strategy's capacity is roughly one hundred dollars per leg, on ~1 qualifying leg per
week, resting on five wins from one account in one regime.

The live boards show the same thing without any backtest at all:

- Elon `500+` leg: **$96,266 of lifetime volume and no book on either side right now.**
  Same for `380-399` ($81,193), `400-419` ($65,109), `440-459` ($71,815).
- Trump `160-179`: quoted **0.050 / 0.110** — a 6c spread on a 5c leg — with $652 traded.
  `180-199`: **0.200 / 0.350**, an **11c spread**, $373 traded. Meanwhile `200-219` and
  `220-239` on the Elon board quote 1c wide on $74–79k.

**Volume concentrates in the middle of a ladder; edge, when it exists, lives in the wings;
the two sets are disjoint.** A board-level tape gate passes this family comfortably and is
answering a question about legs we would never trade.

## Falsification sketch (pre-registered, and how it resolved)

| test | kill condition | result |
|---|---|---|
| Kalshi prices the object | incumbent line unbiased / better-constructed | **FIRED** — `KXTRUTHSOCIAL` live at 100–300k contracts/wk, and it re-cut the cap Polymarket left stale |
| pooled calibration at a tradeable checkpoint | \|edge\| < 2pp | **FIRED** — −0.05 to +1.21pp across four checkpoints |
| any surviving band survives leg-type decomposition | edge concentrated in one leg type | **FIRED** — 100% of it is the open-ended top bucket |
| residue survives executable prices | q⁻ < q* | **FIRED** — interior fails at both spread assumptions; open-HIGH fails at $500 |
| edge is more than one regime | wins concentrated in one account/period | **FIRED** — 5/5 wins are Trump `200+`, May–July |

## What I would need to see to reopen this

A venue running post-count ladders whose **cap is re-cut every week** (so the open-ended leg
carries no design lag) *and* quotes the wing legs two-sided with ≥$500 at the top of book.
Kalshi satisfies the first condition and I have not measured its wing depth; that is the only
live thread, and it is a Kalshi-execution question, not a research one. The firm cannot rest
orders (`CONSTITUTION.md` §5), so the maker-side version — where the 6–11c wing spreads are
income rather than cost — is out of reach for the same reason mention markets were.

## Risk notes recorded for the funnel

- **Resolution feed session calendar**: X and Truth Social are 24/7; the resolving artifact is
  the account's own timeline. No stale-feed exposure. The window boundary is 12:00 ET
  (16:00Z), *outside* our 00:00–13:00Z working window, but the boards run for seven days so
  tradeable checkpoints sit inside it. This family passes the stale-feed gate cleanly — it
  died for other reasons.
- **Look-ahead audit** (`lifetime-volume-is-look-ahead.md`): no filter used here is
  post-checkpoint. Board selection is by leg count, the legsum gate is computed from
  checkpoint prices, and **no volume or liquidity field was used anywhere**. Deliberately —
  the live Elon board is the cleanest example of that trap I have seen, with $96k of lifetime
  volume sitting on a leg that has no quote at all.
