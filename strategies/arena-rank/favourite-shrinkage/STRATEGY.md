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

A board fits when **all four** hold:

1. it is part of a resolved-history ranking cohort;
2. the book passes the quality gate (spread ≤5c, real depth) **and** the de-vigged leg-sum
   is ≤ ~1.05 (`wiki/reference/checkpoint-artifact.md`);
3. the de-vigged favourite sits in the **fundable 0.60–0.90 band**;
4. **(added 2026-07-26)** the **leaderboard margin** at the checkpoint is ≥ 4 points, *or*
   the market's favourite is not the company currently holding the place.

Clause 3 is the pre-registered day-3 kill test and it **passed** (`results/fundable-band-2026-07-26.md`):
the gain is +17.2pp in the fundable band against +4.9pp at 0.93–1.00, and the fundable
band is the only one whose 95% lower bound on the favourite's win rate clears its own
break-even.

Clause 4 is what that test *also* found, and it is the sharper rule: **sharpen a crowd only
where the thing it is pricing is persistent.** At a 0–3 point margin with the market backing
the incumbent, the crowd is already right — market 0.800, realised 0.800, gap +0.0pp — and
α = 1.75 overshoots by 15pp, the worst miss anywhere in the sample. The per-row driver is
Preliminary / low-vote status: one-refresh score sd is 5.87 (Preliminary) and 6.55 (<5k
votes) against 1.60–2.25 for established rows.

**The whole July 2026 cohort fails this test**: six of seven boards are at 0.935–0.983
(clause 3), and the seventh — the Chinese board — is in band but sits at a +3 margin
between two Preliminary sub-3.8k-vote rows (clause 4). All seven applications are
deactivated and **zero rows were proposed on 2026-07-26**.

## How to run

Pipeline inherited from `../satellites/src/` (fetch + vintage archive + de-vig).
`src/` here holds the shrinkage fit and live pricing.

## Evidence

- `../satellites/results/backtest-2026-07-25.md` — the day-1 study: what died (order-
  statistic anchor arithmetic) and what survived (this).
- `results/fundable-band-2026-07-26.md` — the day-3 pre-registered kill test: the band
  test passes, the July cohort fails it, and the margin screen is added.

## Known limitations

- **No sharp-line screen exists for this family.** No bookmaker or exchange prices LMArena
  rankings, so our cheapest falsifier is unavailable and the remaining gates carry more
  weight (`wiki/reference/sharp-line-screen.md`). That absence is a reason to expect an
  edge to survive *and* a reason to hold the other gates strictly.
- Sub-arena boards (math/coding/webdev/agent) have **zero resolved history**; they are
  forward-test-only and cannot bound a 3% tail on a 0.97 favourite.
- The 2026-08 and 2026-09 cohorts are **listed but unpriced** — leg-sums 6.5–12.5, i.e.
  phantom midpoints on empty books. They cannot be traded or measured until they price.

## Changelog

- 2026-07-25 — created from the surviving mechanism of `satellites` (CEO decision on the
  day-1 escalation); slot 2 clock continues, day-3 kill test pre-registered.
- 2026-07-26 — day-3 test run a day early (the cohort resolves 07-31). Band test **passes**;
  applicability gains the **leaderboard-margin** clause; all 7 July applications
  deactivated; zero rows proposed.
