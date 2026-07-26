# tomatometer

> **In plain English:** Bets on the percentage of film critics who liked a new movie. The number is still being counted while the bet is running, and it settles at a fixed clock time on the Monday after opening weekend.

Rotten Tomatoes' Tomatometer is `round(100 × liked / (liked + notLiked))` over professional
critics — a running fraction, not an average rating. Polymarket and Kalshi both list ladders
of "will the score be at least X" legs that settle on whatever the page displays at
**10:00 AM ET on the Monday after wide release**, while critics are still filing. The
denominator roughly triples over the board's five-day life, so the family is a
**counting-process** family: the resolving statistic is partially realised at every
checkpoint, and the residual is binomial noise on a known number of not-yet-arrived reviews.

Born from `ideas/2026-07-26-tomatometer-review-arrival.md`, promoted into slot 2 the same
day it was filed.

Family facts, established 2026-07-26 and worth keeping whatever happens next:

- **Two venues, and Polymarket is the small one.** Kalshi runs the identical object
  (`KXRT` series, 19 resolved boards May–Jul 2026) at **20–100× the volume**, on a **10–29
  rung** ladder against Polymarket's 3–9, at a **1c median spread**. The Odyssey traded
  $7.19M on Kalshi and $41k on Polymarket.
- **Strike semantics differ by one point.** Polymarket "at least X" = `score ≥ X`; Kalshi
  "Above X" = `score ≥ X+1`. Any cross-venue arithmetic that skips this is wrong by a rung.
- **Two structural eras on Polymarket.** 70 cumulative *ladder* boards (2025-10 onward) and
  40 mutually-exclusive *bucket* boards (2024-12 → 2025-12), with two incompatible band
  conventions inside the bucket era. 108 resolved boards in total, not the 67 the founding
  idea assumed.
- **`endDate` is not the resolution instant** — the real 10:00 ET instant appears only in
  the leg `description`, and using `endDate` shifts every checkpoint by up to 15 hours.
- **Fees were off for most of the history**: 369 of 556 modern legs carry
  `feesEnabled = false`; `culture_fees` at rate 0.05 switched on in 2026-02 → 2026-04.
- **The venue can resolve a cumulative ladder incoherently.** On
  `how-to-make-a-killing` the `≥56` leg settled NO while `≥57` settled YES, with $190k of
  notional on the broken leg.
- **Board supply is 2–4 per week**, and at any moment typically **one** board has a lifted
  review embargo. On 2026-07-26 the open set was two boards, both with `reviewCount: 0`.

Variants:

- [`arrival-drift/`](arrival-drift/) — simulate the reviews still to arrive and price the
  ladder off the resulting distribution instead of off the displayed score. Slot 2, started
  **and killed** 2026-07-26.
  [Gate run](arrival-drift/results/gates-2026-07-26.md).

Cross-variant lessons (from `arrival-drift` day 1 — full numbers in that file):

- **The founding thesis is falsified in direction.** The idea held that the crowd anchors on
  a displayed score that is ~2 points too high, so `P(score ≥ s)` is over-priced. On 68
  resolved ladder boards with per-leg ground truth, over-pricing is **zero** below 50c and
  **negative** above it (−0.105 at T−72h, t = −2.09; −0.262 at T−24h, t = −7.77). The crowd
  under-prices its favourites; it does not over-price the level.
- **What lives here is favourite-longshot bias**, independently replicating
  `arena-rank/favourite-shrinkage` in a family with no shared crowd, mechanism or resolution
  source. Any future variant in this family should start from that and from the `q⁻ > q*`
  gate, not from an arrival model.
- **Kalshi is unbiased against realised settlement at every checkpoint from T−96h**
  (implied-median error +0.13 to +0.64, se 0.12–0.81, 9 down / 10 up at T−96h). Anything
  claiming an edge on the *level* of a Tomatometer score has to explain why Kalshi's line
  does not already contain it.
- **T−24h is the cleanest checkpoint, not T−6h.** Monotonicity violations rise 3% → 10% and
  implied mass 1.003 → 1.027 in the final day as deep-OTM legs go one-sided.
- **Per-leg flow is the binding constraint.** Median in-band taker notional on a single leg
  over the final 72 hours is **$238**; only 30% of in-band legs see $1,000.
