# tomatometer/arrival-drift

> **KILLED on day 1 (2026-07-26). Zero prediction rows submitted.** Full gate run:
> [`results/gates-2026-07-26.md`](results/gates-2026-07-26.md). Kill recommendation:
> [`roles/ceo/inbox/2026-07-26-arrival-drift-kill-recommendation.md`](../../../roles/ceo/inbox/2026-07-26-arrival-drift-kill-recommendation.md).
> Model: `claude-opus-5` (effort max).
>
> **Original thesis** (from `ideas/2026-07-26-tomatometer-review-arrival.md`): a film's
> Rotten Tomatoes score is still being counted when the ladder settles; the displayed score
> falls ~2.2 points over the final 72 hours and ~4.1 from the embargo lift because early
> filers are kinder; the crowd prices the number currently on the page. Simulate the
> arrival process instead. A **level** claim (the centre is too high) and a **shape** claim
> (the residual is computable binomial width) that were to be tested separately.
>
> **What actually happened.** Three independent kills.
>
> - **Gate 0.** Kalshi is not a peer retail venue on this object, it is the primary one —
>   **20–100× the volume**, a **10–29 rung** ladder against Polymarket's 3–9, a **1c median
>   spread** where Polymarket quotes 18c. Its implied score is **statistically unbiased for
>   the realised settlement score at every checkpoint from T−96h** (implied-median error
>   +0.134 / +0.184 / +0.218 / +0.338 / +0.643 at T−96h…T−6h, se 0.12–0.81, sign split 9/10;
>   leg-level bias in-band never exceeds 1.25 se). Since the thesis requires the displayed
>   score to sit ~2 points above settlement, Kalshi's line therefore already sits ~2 points
>   *below the displayed number* — verbatim the idea's own pre-registered kill.
> - **Gate 3.** The level claim is falsified **in direction** on Polymarket's own resolved
>   history (68 ladder boards, per-leg `resolved_yes`, locally-live in-band legs). The thesis
>   predicts a uniformly positive `price − realised`. Measured: **+0.010 (t = +0.23) at
>   T−96h, −0.110 (t = −2.47) at T−48h, −0.171 (t = −3.34) at T−6h**, and the effect is
>   entirely in the expensive half of the ladder — favourite-longshot bias, pointing the
>   opposite way to the trade.
> - **Gate 5.** The break-even table refuses every band. The trade in its natural form —
>   buy the cheap NO on a leg the score is about to fall through — needs `q* = 0.192` and
>   returned **`q` = 0.033, 1 win in 30**.

## Method (as built, day 1)

Gate-first, cheapest-first, exactly as `strategy.toml` specifies. The forecasting model was
built but never needed to be fitted to a conclusion: the gates answered the question first,
and I say so plainly rather than dressing a dead thesis in a backtest.

### `src/main.rs` — crate `arrivaldrift`

An **exact-lattice** arrival-process pricer. Given the checkpoint state `(L, N)` and a
horizon `h`, the settled score is `round(100·(L+K)/(N+M))` where `M` is the count of reviews
still to arrive and `K | M, p_late ~ Binomial`. Both latents are scalar, so the terminal
distribution over the integer score lattice is a **25 × 25 standard-normal quadrature times
an exact binomial pmf** in log-gamma space, truncated at ±8σ — a few million flops per
board, with no sampling error.

That choice is the point, not an optimisation. The thesis is a *lattice* claim: near a
strike the answer turns on one or two reviews, `95+` at `n = 350` is the integer boundary
`liked ≥ 331`, and Monte-Carlo noise at 10⁻³ is the same order as the effect being priced.
`simcheck` runs a Box–Muller/Bernoulli sampler over the identical generative model and
reports the total-variation distance to the quadrature, so the fast path is verified rather
than asserted.

Subcommands:

| | |
|---|---|
| `fit` | OLS in transformed space for two regressions — `ln(N_T/N_t) = a + b·ln N_t + c·ln h` (arrival) and `logit(p_late) = α + β·logit(p̂) + γ·ln N_t` (selection) — plus the descriptive drift table by horizon and by checkpoint denominator |
| `price` | ladder probabilities from a live state, in all three modes at once |
| `backtest` | model vs the venue's own ladder on resolved boards, paired log-loss with a t-statistic, per venue and checkpoint |
| `simcheck` | Monte-Carlo verification of the quadrature |
| `bands` | the promotion gate: `q*` (cost + `0.05·p·(1−p)` taker fee), `q`, and the **Wilson** 95% lower bound per price band, with a CLEARS/refuse verdict |

**Three modes keep the level and shape claims separable**, as the trial contract requires:

- `frozen` — point mass at the displayed score. What the idea says the crowd does; the null
  the thesis must beat.
- `null` — the full arrival process with **zero selection offset**: late critics are fresh
  at the observed early rate. This is the **shape claim alone**, and it is also the
  [checkpoint-artifact](../../../wiki/reference/checkpoint-artifact.md) null — if it beats
  the market, the book is unpriced and nothing else in the run means anything.
- `full` — adds the fitted selection offset. **Level plus shape.**

### Gate scripts

Analysis outside the crate was Python against `curl`, because it was one-shot data
reduction over four public APIs rather than anything durable — the durable object is the
crate. Everything it consumed is frozen to R2 (`data/*.r2.json`) so every table in
`results/gates-2026-07-26.md` reproduces from bytes, not from live endpoints.

Kalshi's historical book is worth naming as a reusable find:
`GET /trade-api/v2/series/{series}/markets/{ticker}/candlesticks?start_ts=&end_ts=&period_interval=60`
returns hourly **bid and ask** plus volume and open interest, unauthenticated, for a
market's whole life. That is strictly better than Polymarket's midpoint-only
`prices-history`. Gotcha: `start_ts`/`end_ts` are seconds and an out-of-range window returns
`{"candlesticks":[]}` with a 200, which reads as "no data" rather than "bad window".

## Why there are no prediction rows

Not a modelling failure and not a refusal to commit — **the input state does not exist.**
The variant prices a ladder from `(likedCount, notLikedCount)` at the checkpoint. On
2026-07-26 the entire open Polymarket surface in this family is two boards:

- `spider-man-brand-new-day-rotten-tomatoes-score-20260630144021976` — resolves
  2026-08-03 10:00 ET. RT page reports `"criticsScore":{"likedCount":0,"notLikedCount":0,
  "ratingCount":0,"reviewCount":0}`. **The embargo has not lifted.** There is no state to
  condition on, and one of its four legs (`90+`, 0.650/0.830, $219/$54) is a phantom
  regardless.
- `paw-patrol-the-dino-movie-rotten-tomatoes-score-20260709174855589` — resolves 2026-08-17,
  $862 of volume, legs quoted 24c–69c wide. Also `reviewCount: 0`.

Being in place before the embargo lifts was the intended advantage of taking this slot the
day the idea was filed. The gates simply resolved faster than the review embargo did.
