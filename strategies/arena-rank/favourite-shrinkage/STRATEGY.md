# arena-rank/favourite-shrinkage

> **This variant carries the one mechanism that survived day-1 falsification of
> `arena-rank/satellites`** (see that folder's `results/backtest-2026-07-25.md` for the
> full kill). Thesis: the crowds on LMArena monthly ranking boards are **underconfident
> in their own favourite** — at T−7d the favourite wins **+9.2pp** more often than its
> de-vigged price implies (se 1.9pp, t=4.77, 9/10 cohort-months positive; corroborated
> by a Herfindahl benchmark +0.066, se 0.019). Sharpening the crowd's own distribution
> (p^α, α fit leave-one-month-out) gains **+0.111 log-loss out-of-sample** (t=+2.63,
> 9/10 months); at T−7d **+0.106, t=+7.49, 10/10 months**.
>
> This is textbook favourite-longshot bias — but **larger here (6–9c) than the 1–3c the
> wiki says usually hides inside the favourite's spread**, on boards quoting 0.1–3.7c.
> We take the *crowd's* distribution as the input and sharpen it; we do NOT claim to
> model the ranking better than the market (that claim died on gate 2).

## Method

DAY-1 STATE. Inputs: de-vigged board prices; α from leave-one-month-out fit. Output:
sharpened distribution per board. Resolution reproduction is solid (gate 0: 47/47 exact
via Wayback vintages of `arena.ai/leaderboard/text/overall-no-style-control` — **the
style-control-OFF slice; the default page is style-control ON and gives a different
ordering**). Book-quality gate (spread ≤5c, real depth) applies as for ladder-rv.

## Applicability

A board fits when: it is part of a resolved-history ranking cohort, the book passes the
quality gate, AND the favourite sits in a fundable band. **The fundable-band question is
the pre-registered day-3 kill test** — this cohort's favourites sit at 0.93–0.99, where
return on locked capital after spread may not justify the trade even if the edge is real.

## How to run

Pipeline inherited from `../satellites/src/` (fetch + vintage archive + de-vig).
`src/` here holds the shrinkage fit and live pricing.

## Evidence

- `../satellites/results/backtest-2026-07-25.md` — the day-1 study: what died (order-
  statistic anchor arithmetic) and what survived (this).

## Changelog

- 2026-07-25 — created from the surviving mechanism of `satellites` (CEO decision on the
  day-1 escalation); slot 2 clock continues, day-3 kill test pre-registered.
