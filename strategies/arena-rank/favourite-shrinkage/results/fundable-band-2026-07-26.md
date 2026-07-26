# Day-3 pre-registered kill test: does the favourite-longshot gain live in a fundable band?

**Date:** 2026-07-26 · **Model:** claude-opus-5 (xhigh effort) · **Trial:** slot 2, day 2
(the test was pre-registered for day 3; run a day early because the July cohort resolves
2026-07-31 and the trade would have to be put on today)

**The pre-registered test.** `strategy.toml` `[trial].success_guideline`:

> the favourite-longshot gain must concentrate in a **FUNDABLE band (favourite priced
> 0.60–0.90)**. If the edge exists only on 0.93–0.99 favourites, return on locked capital
> after spread is too thin to justify a slot → retire.

## Verdict in three lines

1. **The test passes.** The gain is *larger* in the fundable band than at 0.93–1.00, and
   the fundable band is the **only** band whose 95% lower bound on the favourite's win
   rate clears its own break-even. The variant must not be retired on this test as written.
2. **The July cohort cannot express it.** Six of seven boards sit at 0.935–0.983 de-vigged
   and are quoted 0.979–0.990 on the ask. All six are deactivated.
3. **The seventh (Chinese) is in the band and still fails**, on a screen the variant did
   not have: the leaderboard **margin**. Its margin today is +3 points between two
   Preliminary sub-3.8k-vote rows. On the historical sample, the cell that matches it
   exactly prices at 0.800 and realises 0.800 — **gap +0.0pp** — while our α=1.75 rule
   would say 0.951. **Zero rows proposed.**

Everything below is reproducible from `src/` against the R2-frozen inputs.

---

## 1. The band split

138 satellite board-checkpoints (anchor `text_overall_nosc_1` excluded), 10 cohort-months
2025-09 → 2026-06, T−30/14/7d. Clustered by cohort-month throughout: within a month the
boards mostly resolve to the same company, so board-level n is fake.

| band | n | months | mean p_fav | realised | gap | se | t | LL gain | t |
|---|---|---|---|---|---|---|---|---|---|
| <0.60 | 26 | 9 | 0.433 | 0.615 | +13.5pp | 14.1 | +0.96 | −0.024 | −0.09 |
| **FUNDABLE 0.60–0.90** | **74** | **10** | **0.785** | **0.919** | **+16.8pp** | **2.8** | **+5.94** | **+0.146** | **+3.90** |
| 0.90–0.93 | 20 | 7 | 0.916 | 1.000 | +8.2pp | 0.3 | +31.2 | +0.080 | +29.4 |
| 0.93–1.00 | 18 | 7 | 0.951 | 1.000 | +4.9pp | 0.5 | +10.3 | +0.045 | +9.05 |

The 74 checkpoints are **46 distinct board-instances**; at instance level the fundable band
is **42/46 favourites won, gap +17.2pp (se 2.4, t = +7.03)**, pnl **+12.88c (t = +5.46)**.
At T−7d alone (the horizon this trial trades): 21/22, gap +17.3pp, t = +5.24, 9 months.

**The gain does not live only at 0.93–0.99. It is roughly 3.4× larger in the fundable band.**

## 2. The same bands priced as a business

`execution/DESIGN.md` §3 and §4. Buy the favourite at raw mid + 2c adverse, hold to the
check, taker fee `0.04 × p × (1−p)` charged **once** (entry; settlement is a redemption,
not a match). Rate 0.04 read off the live `feeSchedule` — these markets are `tech_fees`.

| band | n | mean cost | fee | pnl/trade | t | RoLC | days | annualised |
|---|---|---|---|---|---|---|---|---|
| <0.60 | 26 | 0.513 | 0.96c | +6.59c | +0.49 | +12.8% | 21.5 | +386% |
| **FUNDABLE 0.60–0.90** | **74** | **0.823** | **0.56c** | **+12.49c** | **+4.64** | **+15.2%** | **17.5** | **+457%** |
| 0.90–0.93 | 20 | 0.943 | 0.22c | +5.17c | +9.24 | +5.5% | 10.8 | +196% |
| 0.93–1.00 | 15 | 0.971 | 0.11c | +2.82c | +7.61 | +2.9% | 8.4 | +137% |

Cents per trade would have ranked these 12.5 / 6.6 / 5.2 / 2.8 and told you the fundable
band is 4× the 0.93 band. Return on locked capital says 15.2% vs 2.9% — **5.2×** — and
that is the number `execution/DESIGN.md` says decides.

### The number that actually kills the high band

At cost `c` the favourite must win `c + fee` of the time just to break even.

| band | n | mean c | break-even q* | q observed | q 95% lower bound | verdict |
|---|---|---|---|---|---|---|
| <0.60 | 26 | 0.513 | 0.523 | 0.615 | 0.436 | cannot be shown profitable |
| **FUNDABLE 0.60–0.90** | **74** | **0.823** | **0.829** | **0.919** | **0.846** | **survives a 95% bound (+1.7pp)** |
| 0.90–0.93 | 20 | 0.943 | 0.945 | **1.000** | 0.861 | cannot be shown profitable |
| 0.93–1.00 | 15 | 0.971 | 0.972 | **1.000** | 0.819 | cannot be shown profitable |

The 0.93–1.00 band went **16/16** at instance level and is still not investable: it needs
**97.2%** and 16 observations cannot bound a 3% tail. Concretely — a win pays +2.83c, a
loss costs −97.17c, so **2.83 losses per 100 trades take the band to zero**. Our entire
evidence base for that band is 16 instances over 7 months.

The fundable band needs 82.9% against an observed 91.9%, and **17.10c up against 82.90c
down: 17.1 losses per 100** before it is flat. That is a business; the other is a coin
placed under a steamroller.

## 3. Mandatory re-checks (both clean)

**Checkpoint-artifact gate** (`wiki/reference/checkpoint-artifact.md`). Leg-sum over the
138 historical checkpoints: median 1.015, p10 0.977, p90 1.123; 25/138 above 1.05.

| | n | LL gain |
|---|---|---|
| leg-sum ≤ 1.05 (priced) | 113 | **+0.129** (se 0.028, t +4.70) |
| leg-sum > 1.05 | 25 | −0.184 (se 0.313, t −0.59) |

The gain is *entirely* in the priced half — the gate is doing real work, in the right
direction. Restricted to priced books the fundable band goes to **+18.4pp (t +8.43)**.

**Null models, same pipeline.** Uniform-over-legs loses to the market by 1.98 log-loss;
flat-0.90-on-the-favourite loses by 0.103; the leaderboard-margin null (§5) loses in every
margin band. **No null beats the market anywhere.** We are not measuring an unpriced book.

**Phantom-midpoint split** (`wiki/reference/phantom-midpoints.md`), by the favourite's
14-day total variation before the checkpoint: **137/138 checkpoints are LIVE (tv ≥ 5c)**,
1 near-flat, 0 dead. Live-book headline = pooled headline (+0.111 LL, +13.8pp). This
family does not have the bo3/tennis disease.

**Live leg-sums today**, all seven boards: 0.9710, 0.9865, 1.0085, 1.0060, 1.0035, 1.0255,
1.0275 — all inside the ≤1.05 gate.

## 4. The July cohort on today's book — and the six that leave

Yesterday's `[book]` blocks were already stale: the Chinese favourite moved
**0.8275 → 0.7765** overnight on **no new leaderboard data** (the resolving table still
stamps `Jul 21, 2026`).

| board | favourite | mid | bid | ask | spread | de-vig | band | ask cost | gross edge | RoLC | annualised | break-even q* | losses/100 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **chinese** | Alibaba | 0.7765 | 0.775 | 0.778 | 0.3c | **0.7997** | **FUNDABLE** | 0.7780 | +14.51c | +18.7% | +1318% | 0.785 | 21.5 |
| math-1 | Anthropic | 0.9625 | 0.953 | 0.972 | 1.9c | 0.9757 | out | 0.9720 | +2.19c | +2.3% | +159% | 0.973 | 2.7 |
| overall-nosc-2 | Anthropic | 0.9775 | 0.965 | 0.990 | 2.5c | 0.9693 | out | 0.9900 | +0.46c | +0.5% | +33% | 0.990 | **1.0** |
| overall-nosc-3 | Anthropic | 0.9890 | 0.988 | 0.990 | 0.2c | 0.9831 | out | 0.9900 | +0.46c | +0.5% | +33% | 0.990 | **1.0** |
| overall-sc-1 | Anthropic | 0.9750 | 0.960 | 0.990 | 3.0c | 0.9716 | out | 0.9900 | +0.46c | +0.5% | +33% | 0.990 | **1.0** |
| overall-sc-2 | Anthropic | 0.9715 | 0.953 | 0.990 | 3.7c | 0.9473 | out | 0.9900 | +0.46c | +0.5% | +33% | 0.990 | **1.0** |
| overall-sc-3 | Anthropic | 0.9605 | 0.942 | 0.979 | 3.7c | 0.9348 | out | 0.9790 | +1.52c | +1.6% | +110% | 0.980 | 2.0 |

Four of the six quote an **ask of 0.990** — pay 99c to win 1c over five days, and one loss
per hundred erases the band. The "gross edge" is not even a model output: the sharpening
rule hits its 0.995 clip on all six, so the entire displayed edge is *clip minus ask*.

### Fill evidence from the tape, on the side we would take

Data API, taker-buy fills folded to Yes-equivalent units exactly as `tools/fillcheck`
does (a taker who **bought** proves a resting **ask** existed at that price).

| board | taker buys 7d | $ 7d | taker buys 30d | $ 30d | best ask traded 7d | $ at/below our cost 7d |
|---|---|---|---|---|---|---|
| **chinese** | 492 | **$33,176** | 1,507 | $72,390 | 0.540 | **$11,752** |
| math-1 | 103 | $6,028 | 645 | $38,195 | 0.890 | $6,026 |
| overall-nosc-3 | 87 | $11,117 | 261 | $30,282 | 0.910 | $11,117 |
| overall-sc-2 | 39 | $668 | 92 | $3,755 | 0.940 | $664 |
| overall-nosc-2 | 37 | $606 | 134 | $5,307 | 0.940 | $368 |
| overall-sc-3 | 19 | $209 | 83 | $3,422 | 0.928 | $208 |
| overall-sc-1 | 22 | **$58** | 133 | $8,297 | 0.986 | $50 |

`overall-sc-1` has **$58** of realised taker buying in a week. Three of the six are under
$700. The depth column in yesterday's application files ($7.2k–$25.9k within 10c) is a
book measurement; the tape says the flow that actually crosses is one to two orders of
magnitude smaller on the boards we would have traded.

**All six out-of-band boards are deactivated.**

## 5. The screen the variant did not have: leaderboard margin

The migrated Chinese application justifies its edge with:

> a 3-point gap between two Preliminary rows, where empirical one-refresh persistence is
> **0.976–0.982**

That figure is the **top-30 pooled** number from `../satellites/results/backtest-2026-07-25.md`
§5, and it does not apply here. Measured on the resolving slice's own vintage archive
(38 distinct data-dates, 2025-08 → 2026-07):

| sd(Δ score), one refresh, top-30 | value |
|---|---|
| established rows | 2.25 |
| **Preliminary rows** | **5.87** |
| ≥5k votes | 1.60 |
| **<5k votes** | **6.55** |

P(leader stays ahead) over one refresh, gap 3–5 points: **all pairs 0.974 (n=1967)**, but
**both rows <5k votes → 0.846 (11/13)**. The pooled number is dominated by established
50k-vote rows. Applying it to two Preliminary sub-3.8k-vote rows is the
published-CI-vs-printed error (`wiki/reference/published-ci-vs-printed.md`) run in reverse:
using a statistic computed on one population as if it described another.

At the **company** level — what the board actually resolves on — leadership of the Chinese
sub-ranking persists **0.62–0.68 over 1–14 days**, and **0.44–0.54 at a 0–3 point margin**.
The Chinese leader in the archive goes Alibaba → Z.ai → Baidu → Alibaba → Baidu →
Bytedance → Alibaba → Baidu → Z.ai → Alibaba. It churns; the overall board does not.

### But the margin null does *not* invert the FLB result — it localises it

131 checkpoints with a pinned vintage **and** a live book. Margin = score gap between the
place-holder and the best model of any other company at or below that place.

| margin | n | market fav == table leader | leader kept it | market p_fav | fav won | market gap | our p | our gap |
|---|---|---|---|---|---|---|---|---|
| 0–3 | 22 | 0.455 | 0.364 | 0.676 | 0.818 | +14.2pp | 0.809 | +0.9pp |
| 4–7 | 38 | 0.711 | 0.632 | 0.789 | 0.921 | +13.2pp | 0.923 | −0.2pp |
| 8–14 | 48 | 0.750 | 0.771 | 0.780 | 0.854 | +7.4pp | 0.918 | −6.4pp |
| 15+ | 23 | 0.870 | 0.870 | 0.798 | 0.957 | +15.8pp | 0.935 | +2.1pp |

The market beats the margin null in every band (log-loss 0.25–0.46 vs 0.52–1.26). At a
0–3 margin the market's favourite **is not the current leader 55% of the time**, and it
still wins 81.8% — the crowd is anticipating refreshes and is right. That is a genuinely
impressive crowd, not an underconfident one.

### The decisive cell — today's Chinese board, exactly

| cell | n | months | fav won | market | market gap | our α=1.75 price | our gap | 95% lo |
|---|---|---|---|---|---|---|---|---|
| fundable band, all margins | 71 | 10 | 65/71 = 0.915 | 0.786 | **+12.9pp** | 0.947 | −3.2pp | 0.840 |
| margin 0–3, **market fav IS the leader** | 10 | 5 | 7/10 = 0.700 | 0.635 | +6.5pp | 0.777 | **−7.7pp** | 0.393 |
| …**and** favourite in 0.60–0.90 | **5** | **4** | **4/5 = 0.800** | **0.800** | **+0.0pp** | **0.951** | **−15.1pp** | 0.343 |
| margin 0–3, market fav is **not** the leader | 12 | 5 | 11/12 = 0.917 | 0.709 | +20.7pp | 0.836 | +8.1pp | 0.661 |
| margin 4–7, fav == leader | 27 | 10 | 24/27 = 0.889 | 0.777 | +11.1pp | 0.918 | −2.9pp | 0.737 |
| margin 8+, fav == leader | 56 | 9 | 53/56 = 0.946 | 0.815 | +13.2pp | 0.954 | −0.8pp | 0.867 |

**Where the fundable-band gain lives:** margin ≥ 4 with the market backing the incumbent
(+11 to +13pp, n = 27 and 56), or a small margin where the market backs the *challenger*
and is right (+20.7pp, n = 12). **The one cell with no measured gain is the one the live
Chinese board sits in**: incumbent, 0–3 point margin, priced 0.60–0.90 — market 0.800,
realised 0.800, and our rule would have overshot by 15.1pp, the worst miss of any cell.

n = 5 cannot prove the edge is zero there. It can and does establish that **there is no
evidence for the trade**, that the point estimate is nil, and that our model's error in
that cell is the largest we measure anywhere.

### The live Chinese board, all numbers together

| | |
|---|---|
| resolving table (`text/overall-no-style-control`, `Jul 21, 2026`) | Alibaba `qwen3.7-max-preview` **rank 11, 1476 ±10, Preliminary, 3,714 votes** |
| challenger | Moonshot `kimi-k3` **rank 13, 1473 ±10, Preliminary, 3,619 votes** — margin **+3** |
| Alibaba's backstop | `qwen3.5-max-preview` 1470 — *below* kimi-k3 |
| refresh cadence | median 7d, last refresh Jul 21, check Jul 31 → **1–2 refreshes due** |
| sd(Δscore) for these rows | ≈ 5.9–6.6 per refresh, i.e. ~2× the margin |
| naive company-leadership persistence at margin 0–3 | **0.44–0.54** |
| historical cell (incumbent, margin 0–3, 0.60–0.90) | 0.800 → 0.800 |
| market today, de-vigged | **0.7997** |
| our α=1.75 sharpened price | **0.9300** |
| break-even at the 0.778 ask incl. fee | **0.7849** |

The market is already ~26pp above the naive persistence for this board. The remaining
uncertainty is *when the next refresh lands and what a Qwen/Kimi release does to it* —
release timing, which is private information and which `wiki/market-selection.md` says to
select against, and which `../satellites/results/backtest-2026-07-25.md` §5 already
identified as this family's dominant unmodelable risk.

**Not tradeable. Deactivated.**

## 6. Proposed rows

`results/proposed-rows-2026-07-26.csv` — **header only, zero rows.**

Six boards fail the band test; the seventh fails the margin screen. Proposing the six
0.995-vs-0.976 rows would score a near-certain paired-Brier win with **$58–$668** of weekly
reachable flow behind it — precisely the calibration-mistaken-for-a-business pattern
`wiki/reference/midpoint-is-not-a-fill.md` was written about yesterday. We are not padding
the ledger with it.

## 7. What this changes about the variant

The mechanism is **not** falsified — it passed its own pre-registered test with the widest
margin available in the data. What is falsified is the **applicability rule**: "a board fits
when the book passes the quality gate AND the favourite sits in a fundable band" is
insufficient. It needs a third clause:

> **the leaderboard margin at the checkpoint must be ≥ 4 points, or the market's favourite
> must differ from the current place-holder.** At a 0–3 point margin with the market backing
> the incumbent, the crowd is already right and sharpening overshoots by ~15pp.

Mechanistically this is one sentence: **sharpen a crowd only where the thing it is pricing
is persistent.** The margin, measured on the resolution variable's own archive rather than
on prices, is the persistence proxy — and the per-row split that makes it work is
Preliminary/low-vote status, where one-refresh score volatility is 3–4× higher.

## 8. Files

- `src/band_split.py` — the pre-registered test, leg-sum gate, nulls, phantom split
- `src/robust.py` — jackknife, board-type split, break-even bounds
- `src/live_analysis.py` — today's books, executable-ask economics, tape fill evidence
- `src/persistence_screen.py` — per-board-type persistence of the resolution variable
- `src/margin_null.py` — the margin null and the decisive cell
- `src/chinese_persistence.py`, `src/h2h_preliminary.py` — the Chinese board in detail
- `src/archive_tables.py` — daily forward vintage archive (memory duty 1)
- `data/live-2026-07-26.json.gz.r2.json` — books + full taker tape, 182 legs, 26k trades
- `data/arena-tables-2026-07-26.tar.gz.r2.json` — today's resolving tables, raw + parsed
