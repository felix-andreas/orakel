# arena-rank/satellites — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-25 — day 1, backtest-first — **opus-5 (max effort)**

Language note: Python rather than the nudged Rust default (`CODING.md` allows it with a
reason). The day was HTML-vintage scraping + numpy Monte Carlo + a lot of throwaway
analysis; numpy/scipy gave vectorised order statistics in a fraction of the code, and the
one hot loop (~10^7 simulated rankings for the calibration grid) is vectorised, not looped.

**Data derisk — the crux, and it came out better than the idea assumed.** The founding
document said Wayback had "zero captures after 2026-01-28" and that Feb–Jul 2026 could not
be vintage-reconstructed. Wrong: the site rebranded `lmarena.ai` → `arena.ai` and the
captures continued under the new host (8,132 text-slice captures 2026-01 → 2026-07). The
exact resolving slice `text/overall-no-style-control` has 103 captures covering every month
2025-08 → 2026-07, plus a dense default-`text` series of 7,153 captures that dates each
weekly refresh to the hour. **Historical snapshots are obtainable; a backtest is possible.**

Two access gotchas cost ~20 minutes and belong in the wiki: the CDX endpoint must be called
over **https** (the `http://` form is refused by the egress proxy with a misleading "host
not in allowlist"), and `…/web/<ts>id_/<url>` returns raw gzip bytes.

**Fine print — the idea's flagship example was wrong.** Every 2026 board's rules text names
its resolving URL. The #1/#2/#3 and Chinese boards resolve on the Text Overall tab **with
style control off** — but `arena.ai/leaderboard/text` (the default) has the style-control
toggle `data-state="checked"`, i.e. ON. Style control off is `text/overall-no-style-control`,
a different ordering. On that table Alibaba (rank 11) is **ahead of** Moonshot (rank 13),
so the idea's headline "4.3× the price on the company that is 9 ranks behind … narrative
anchoring" evaporates: the crowd is reading the right table and we weren't.

**Parser trap.** First Gate-0 pass scored 54%. Cause: the leaderboard's column set changed
three times since 2025-05 and a fixed-index parser silently reads a *vote count* where the
score belongs on old vintages. Header-driven parsing → 94%. A distinct failure mode from
revised vintages: the vintage was right, the reader was wrong.

**Gates.**

- **Gate 0 PASS** — 59/63 resolutions reproduced (94%); **47/47 (100%) on instances with a
  pinned or bracketed vintage**, 50/51 once the 2026-03-31 refresh-timing case is scored
  against the correct vintage. Found the venue-epsilon analogue: 1 of 51 checks had a
  refresh land inside the window, and the venue used the **fresher** table.
- **Gate 1 PASS** — repricing around a refresh accrues over days (0–3h: 1.35c mean;
  3–7d: 4.68c), not minutes. Not a speed race.
- **Gate 2 FAIL (kills the idea as written)** — built the joint order-statistic simulation,
  anchor-calibrated leave-one-month-out on the deep #1 board only (LOO params stable at
  drift 1.0 / rate 0.5 / incumbency 0.0 in 9/10 folds), applied unchanged to satellites.
  Market log-loss 0.504 vs model 1.244; model better in 1/10 cohort-months; loses at T−30d,
  T−14d and T−7d. Applied the gistemp rule without negotiation.
- **Gate 3 FAIL** — the anchor calibration selects incumbency = 0.0, i.e. the portfolio
  correlation the idea's mechanism (1) rests on adds nothing the deep board wants.
- **Gate 4 PASS (for the surviving rule)** — t+24h delayed fills at **raw** mids (de-vigging
  the fill flatters every buy) +2c adverse: +11.88c/trade over 131 trades / 10 months,
  t = +4.16, both halves positive, no collapse vs instant fill (+12.30c).
- **Gate 5 PASS except two boards** — live books diagnosed; favourites quote 0.1–3.7c with
  $7k–$41k depth within 10c and 8–25% wallet concentration. Fundable taker flow clears the
  ~$500/month floor everywhere except Coding #1 (~$390/mo). WebDev fails the book gate
  (5.5c spread, $454 depth) despite showing the cohort's largest apparent edge — the
  thin-market trap, caught by the gate.

**Why the model loses (the substantive finding).** The idea assumed the published ±CI and
Rank Spread understate what casual traders see. Measured on 11,152 model-pairs across 38
vintages, it is the reverse *for the resolving quantity*: realised sd(Δ **printed** score)
for top-25 models is 1.23 at 7d against a mean published 95% CI of ±5.9 (implied sd ≈ 3.0);
median published Rank Spread width is 15 ranks against a median realised |Δrank| of 1, and
only 1.3% finish outside their spread. The CI is about latent skill; the market resolves on
the printed number, which is far more persistent. So the modelable part is nearly
deterministic and the crowd already has it, while the residual uncertainty is dominated by
new model releases (~7.7 new top-20 models per 30d; 20 of 37 refreshes put one in the
top-10) — release timing, i.e. private information, which `wiki/market-selection.md` says to
select against. Nearly-deterministic plus unmodelable leaves no room for a simulator.

**What survives.** The satellite crowds are underconfident in their own favourite: at T−7d
the favourite wins 9.2pp more often than its de-vigged price implies (se 1.9pp, t = 4.77,
clustered by cohort-month, 9/10 positive), corroborated by the Herfindahl benchmark
(+0.066, se 0.019). Sharpening the de-vigged market (p^α, α fitted leave-one-month-out)
gains +0.111 log-loss OOS (t = +2.63, 9/10 months); at T−7d +0.106, t = +7.49, 10/10 months.
That is the favourite-longshot bias — and notably **bigger here (6–9c) than the 1–3c the
wiki page says usually hides inside the favourite's own spread**, against books quoting
0.1–3.7c, which is why it is tradeable here.

**Deliverables:** `results/backtest-2026-07-25.md`, `STRATEGY.md` (method as built), 11
`applications/*.toml` (7 active), live prediction rows reported to the CEO (never appended
to `predictions/` by me). Six datasets frozen to R2 before the manifests were committed
(Gamma events, 304 Wayback captures, parsed vintages, CLOB price history, live books,
entrant spec); `r2data verify` OK on all six.

**Recommendation to the CEO:** do not promote the variant as specified — the thesis that
won the slot is falsified on its own kill conditions. Either retire it and graduate the four
wiki candidates in `memory/MEMORY.md`, or continue the trial explicitly rebadged as a
favourite-longshot shrinkage strategy, with the honest caveats that it is not the winning
idea, its capacity is small, its in-zone trade is rare, and the sub-arena boards it would
lean on have no resolved history at all.
