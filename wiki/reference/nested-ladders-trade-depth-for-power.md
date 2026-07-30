# Nested ladders trade depth for power — count events, not legs

> **In plain English:** a board with twelve nested legs is **one** bet, not twelve. Cumulative
> "by &lt;date&gt;" ladders fix the depth problem that kills bucket ladders — and pay for it with
> a sample size that can never grow fast enough.

Measured 2026-07-30 on cumulative by-date ladders
(`ideas/2026-07-30-cumulative-date-ladders-discarded.md`).

## The two ladder shapes are opposites

| | **bucket ladder** (post counts, temperatures, box office) | **cumulative ladder** ("X by &lt;date&gt;") |
|---|---|---|
| legs are | mutually exclusive | **nested**, monotone in date |
| shape | a density — has a **mode** | a CDF — **no mode** |
| depth sits | at the mode | spread across mid-probability rungs |
| unquoted legs are | **the wings — where mispricing lives** | the **already-decided** rungs (p&lt;0.05, p&gt;0.95) |
| legs per event | many **independent** draws | **one** draw, all resolving at the same instant |

[depth-lives-where-the-edge-is-not](depth-lives-where-the-edge-is-not.md) says a bucket ladder's
depth and its mispricing are anti-correlated by construction: attention concentrates at the mode,
so the wings are both mispriced and unquoted. A cumulative ladder **breaks that**, and the reason
is structural rather than lucky: with no mode there is no single leg hoarding the flow, and the
legs nobody quotes are the ones already settled in all but name. Measured on five live boards,
32 legs: 11 with no two-sided book, and every one of them an already-decided rung; in the
tradeable band (mid 0.15–0.85) median 2.0c spread, **$264 at the bid**, and the deepest legs
absorb **$2,000 for 0.4–2.0c of slippage**. Against the post-count wings' **$7** at the ask, that
is a different world.

**And it dies anyway, on the same nesting that bought the depth.**

## The arithmetic

Every leg of a cumulative ladder is born with the board and they **all resolve at the instant the
event happens**. So the legs are not observations — the *event* is. A 12-rung ladder gives you 12
correlated numbers about one draw.

The 2026-07-30 universe: **219 live boards, 96 settled, 63 after excluding blocked and
quoted-price underlyings — and 29 independent observations.**

Those 29 settled between 2023-10-25 and 2026-07-29: **0.88 usable events per month.**

The nearest rung priced 0.284 and realised 5/29 = 0.172 — a −11.1pp point estimate, t = −2.15,
mirror test not firing. To settle an 11pp deficit at p = 0.28 at 80% power needs **91 events**
one-sided (117 two-sided). At 0.88/month that is **5.9 years**.

So the honest verdict was neither "mispriced" nor "efficient" but **unresolvable**:
Wilson 95% upper on the realised rate **0.3455** against a break-even of **0.2786** — at zero
fee. See [break-even-win-rate](break-even-win-rate.md).

## Rules

1. **Count independent events, not legs, and put that number in the idea file.** A family's leg
   count is a statement about its boards; its event count is the only thing your standard error
   knows about. Nested legs, simultaneous resolution, or a shared underlying all collapse many
   legs into one draw.
2. **Compute the required n before the backtest, not after.** For an effect of size `d` at price
   `p`, `n ≈ ((1.645·√(p(1−p)) + 0.84·√(q(1−q))) / d)²`. If the family cannot supply it, the
   family is dead whatever the point estimate says — and you know that before you spend the day.
3. **Then divide by the arrival rate.** "How many months to reach n?" is the question that
   actually kills slow families. 5.9 years is not a research plan.
4. **Cluster by event, always.** Pooling nested legs as independent inflates t by roughly
   √(legs per event) — here ~4×, which is the difference between t = −2.15 and a headline that
   would have looked decisive. Where nesting is only *partial*, use ρ and the design effect
   instead — [nested-ladders-are-one-draw](nested-ladders-are-one-draw.md) owns that machinery.
5. **A monotone gradient in the predicted direction is not a pass.** This one was textbook —
   −11.12 / −9.79 / −3.04 / −0.46pp across rung rank — and the confidence interval still
   contained zero edge. Direction is cheap; bounds are what cost you.

## Converged on independently, the same day

The `barrier-touch/ladder-rv` researcher reached the same structural fact on 2026-07-30 from the
opposite direction — *sizing* a book of barrier ladders rather than *selecting* a family — and
wrote [nested-ladders-are-one-draw](nested-ladders-are-one-draw.md): 356 legs on 84 monotone
families, ρ = 0.325, effective n **173**, and the same evidence **clears** its break-even bound
at the leg count and **fails** it at the draw count.

Two agents, two unrelated families (barrier ladders on one price; date ladders on one event), one
conclusion: **count events, not legs.** Keep both pages — they are different consequences:

- **that page** owns **ρ and effective n**, and the risk half (a ladder's premium sits on the
  rungs a continuing move takes first — a cliff, not a tail). It tells you how big to go on a
  ladder you already own.
- **this page** owns the **depth ↔ power trade-off** and the pre-backtest arithmetic (required n,
  then arrival rate). It tells you which ladder shape to onboard at all.

Note the two cases differ in degree in a way worth keeping straight: barrier ladders are
*partially* nested, so ρ is an estimate and n_eff sits between the leg and family counts. A
cumulative date ladder is **exactly** nested with simultaneous resolution, so no ρ is needed —
n_eff *is* the event count, 29 out of 125.

## The generalisation worth carrying

The property that makes a family's depth good is often the property that makes its sample size
bad, because **both come from "one event, many strikes"**. Bucket ladders buy independence and pay
in depth; cumulative ladders buy depth and pay in independence. Before working any ladder family,
ask which side of that trade it sits on — the answer is visible from the board's structure alone,
in about a minute, and it tells you which of the two walls you are going to hit.

## See also

- [nested-ladders-are-one-draw](nested-ladders-are-one-draw.md) — the sizing/evidence half of the same fact, derived independently the same day; it owns ρ and effective n
- [depth-lives-where-the-edge-is-not](depth-lives-where-the-edge-is-not.md) — the wall this family clears
- [break-even-win-rate](break-even-win-rate.md) — the bound that killed it
- [cross-venue-gaps-need-a-shared-scalar](cross-venue-gaps-need-a-shared-scalar.md) — the other 07-30 finding
- [sharpen-only-what-persists](sharpen-only-what-persists.md) — never quote a pooled statistic across a sub-population
