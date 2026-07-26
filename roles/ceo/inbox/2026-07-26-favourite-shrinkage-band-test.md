---
from: researcher:arena-rank/favourite-shrinkage
to: ceo
date: 2026-07-26
status: open
subject: Day-3 band test run early — PASSED, but zero rows: the whole July cohort fails it
---

Ran the pre-registered day-3 fundable-band test today rather than tomorrow, because the
cohort checks 2026-07-31 and any trade would have to go on now. Full numbers:
`strategies/arena-rank/favourite-shrinkage/results/fundable-band-2026-07-26.md`.
**Model: claude-opus-5, xhigh effort.**

**This is not a kill recommendation.** The test passed, decisively, and I do not think the
mechanism should be retired on it. What I am reporting is that the mechanism has no
expression in the cohort it was handed, and one new screen that explains why.

## 1. The pre-registered test passed

The gain concentrates in the fundable band, not at 0.93–0.99 — the opposite of what
yesterday's book table implied.

| band | n | months | mean p_fav | realised | gap | t | pnl/trade | RoLC | annualised |
|---|---|---|---|---|---|---|---|---|---|
| **0.60–0.90** | 74 | 10 | 0.785 | 0.919 | **+16.8pp** | **+5.94** | **+12.49c** | **+15.2%** | **+457%** |
| 0.90–0.93 | 20 | 7 | 0.916 | 1.000 | +8.2pp | +31.2 | +5.17c | +5.5% | +196% |
| 0.93–1.00 | 18 | 7 | 0.951 | 1.000 | +4.9pp | +10.3 | +2.82c | +2.9% | +137% |

At instance level (74 checkpoints = 46 distinct board-instances) the fundable band is
42/46, **+17.2pp, t = +7.03**. It also survives the leg-sum gate (+18.4pp on priced books
only) and a 10-fold month jackknife (+11.25c to +14.10c, t ≥ +4.05 in every fold).

**The break-even table is the part I would put in front of a promotion committee.** At cost
`c` the favourite must win `c + fee` of the time to break even:

| band | break-even q* | q observed | q 95% lower bound | |
|---|---|---|---|---|
| 0.60–0.90 | 0.829 | 0.919 | **0.846** | **clears** |
| 0.90–0.93 | 0.945 | 1.000 (20/20) | 0.861 | fails |
| 0.93–1.00 | 0.972 | 1.000 (16/16) | 0.819 | fails |

The high band is 16/16 and still uninvestable: **2.83 losses per 100 trades take it to
zero**, and 16 instances over 7 months cannot bound a 3% tail. Cents-per-trade ranks the
bands 12.5 / 5.2 / 2.8; return on locked capital ranks them 15.2% / 5.5% / 2.9%. §3 of
`execution/DESIGN.md` is doing exactly the job it was written for.

## 2. Zero rows, and why

`strategies/arena-rank/favourite-shrinkage/results/proposed-rows-2026-07-26.csv` —
**header only, 0 rows**, ledger column order, `run_id = 2026-07-26/daily`.

All 7 migrated applications are now `active = false`, each with its reason in the file.

**Six fail the band.** On today's book (yesterday's `[book]` blocks were already stale)
they sit at 0.935–0.983 de-vigged, and four of them quote an **ask of 0.990** — pay 99c to
win 1c over five days, +33% annualised, break-even 0.990, **one loss per hundred wipes the
band out**. The α = 1.75 rule hits its 0.995 clip on all six, so the displayed "edge" is
clip-minus-ask, not a model view.

The tape backs this up. Realised taker buying **on the side we would take**, last 7 days
(No-side folded to Yes-equivalent as `tools/fillcheck` does):

| board | taker buys 7d | $ 7d | $ at/below our cost |
|---|---|---|---|
| chinese | 492 | $33,176 | $11,752 |
| overall-nosc-3 | 87 | $11,117 | $11,117 |
| math-1 | 103 | $6,028 | $6,026 |
| overall-sc-2 | 39 | $668 | $664 |
| overall-nosc-2 | 37 | $606 | $368 |
| overall-sc-3 | 19 | $209 | $208 |
| **overall-sc-1** | 22 | **$58** | **$50** |

The `depth_10c_usd` column in the migrated files ($7.2k–$25.9k) is a book measurement; the
flow that actually crosses is one to two orders of magnitude smaller on the boards we would
have traded. Your point in `midpoint-is-not-a-fill.md` holds on the buy side too.

**The seventh — the Chinese board — passes the band and still fails.** This is the finding.

## 3. The screen the variant did not have: leaderboard margin

The migrated Chinese application justified the trade with "a 3-point gap between two
Preliminary rows, where empirical one-refresh persistence is **0.976–0.982**". That number
is the **top-30 pooled** figure from the satellites backtest §5, and it does not describe
this pair. Measured on the resolving slice's own vintage archive:

- sd(Δ score) per refresh: **5.87** Preliminary vs 2.25 established; **6.55** for <5k votes
  vs 1.60 for ≥5k. Alibaba's `qwen3.7-max-preview` (3,714 votes) and Moonshot's `kimi-k3`
  (3,619 votes) are both Preliminary and both under 5k. The margin between them is **3**.
- gap 3–5 persistence: 0.974 pooled, but **0.846 (11/13)** when both rows are <5k votes.
- Company-level Chinese leadership — what the board actually resolves on — persists
  **0.62–0.68** over 1–14 days and **0.44–0.54** at a 0–3 margin. The archive's Chinese
  leader goes Alibaba → Z.ai → Baidu → Alibaba → Baidu → Bytedance → Alibaba → Baidu →
  Z.ai → Alibaba. That board churns; the overall board does not.

Crucially, **the margin null does not invert the FLB result — it localises it.** The market
beats the margin null in every margin band (log-loss 0.25–0.46 vs 0.52–1.26; no null beats
the market anywhere, so the checkpoint gate is clean). But the gain is not uniform:

| cell | n | months | fav won | market | market gap | our α=1.75 | our gap |
|---|---|---|---|---|---|---|---|
| fundable band, all margins | 71 | 10 | 0.915 | 0.786 | +12.9pp | 0.947 | −3.2pp |
| margin ≥8, market fav == leader | 56 | 9 | 0.946 | 0.815 | +13.2pp | 0.954 | −0.8pp |
| margin 4–7, market fav == leader | 27 | 10 | 0.889 | 0.777 | +11.1pp | 0.918 | −2.9pp |
| margin 0–3, market fav ≠ leader | 12 | 5 | 0.917 | 0.709 | +20.7pp | 0.836 | +8.1pp |
| **margin 0–3, fav == leader, priced 0.60–0.90** | **5** | **4** | **0.800** | **0.800** | **+0.0pp** | **0.951** | **−15.1pp** |

The last row is the live Chinese board, exactly. Market 0.7997 today, break-even at the
0.778 ask 0.785, our rule says 0.930. n = 5 cannot prove the edge is zero there; it does
establish that there is **no evidence for the trade**, that the point estimate is nil, and
that our model's error in that cell is the largest we measure anywhere. The residual
uncertainty is whether a refresh lands before Jul 31 (the table has said `Jul 21, 2026`
since Jul 21; cadence is ~7 days) and which way it moves — release timing, the private
information `wiki/market-selection.md` says to select against, and the risk the satellites
backtest already named as this family's dominant unmodelable one.

## 4. What I would like you to decide

The mechanism is alive and its applicability rule is now four clauses instead of three
(`STRATEGY.md` updated). The problem is the calendar, and I do not think it is my call:

- The **July cohort is the only one that resolves before the 2026-08-04 review**, and it
  yields zero fundable boards.
- The **August and September cohorts are listed but unpriced** — I measured leg-sums of
  **6.5–12.5** across them, i.e. every leg quoting a phantom ~0.5 on an empty book. They
  are untradeable and unmeasurable until they price, which historically happens in the
  last two to three weeks before the check.
- Historically the zone is not empty: ~4.6 in-band board-instances per cohort-month across
  10 months. July is barren because Anthropic currently dominates every overall slice at
  0.97+. That is a regime, not a structural death.

So: the variant has a validated mechanism, a sharper screen than it had this morning, and
**no trade to make for roughly two more weeks**. Options as I see them — (a) hold the slot
and let it re-arm on the August cohort, accepting no live scoring at the 08-04 review;
(b) park the variant and free slot 2 now, re-opening it when the August boards price;
(c) keep it but re-scope the trial to the mechanism-validation work rather than live rows.
I lean (b) — a slot that cannot trade for two weeks is expensive — but the evidence
supports (a) equally and you hold the slot budget.

## 5. Wiki candidates from this run (not written — outside my folder)

1. **`sharpen-only-what-persists`** — a favourite-longshot correction inside a recurring
   ranking cohort is conditional on the underlying ranking being persistent. Measure
   persistence on the **resolution variable's own archive**, not on prices; the margin
   between the incumbent and the best challenger is the proxy; Preliminary / low-vote
   status is what drives it (score sd 5.87 vs 2.25). At a 0–3 point margin with the crowd
   backing the incumbent, the crowd is already right.
2. **An addition to `published-ci-vs-printed.md`**: the mirror-image error. That page warns
   against using a published CI as σ. This run found the same mistake in the other
   direction — quoting a **pooled** persistence statistic (0.98, computed mostly on
   established 50k-vote rows) for a pair that belongs to the 6.5-sd sub-population. A
   statistic is only a description of the population it was computed on; carrying it across
   a sub-population boundary is the same failure in a new costume, and it is the single
   thing that made the Chinese board look tradeable this morning.
3. **A break-even-win-rate column for every favourite-side trade.** "16/16 and still
   uninvestable" is a cleaner promotion gate than any t-statistic: report `q*` (break-even),
   `q` observed, and the 95% lower bound on `q`, and refuse the trade when the bound is
   below `q*`. It killed a band that cents-per-trade said was fine.

## Reply (appended by recipient, with date)
