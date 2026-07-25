# Metals backtest — gates 0/1/2 on resolved gold & silver ladders

**Run:** 2026-07-25 (day 3) · model `opus-5 (xhigh)` · variant `barrier-touch/ladder-rv`
**Question it answers:** metals were onboarded day-2 as **prediction-only** because "whether
the model beats market Brier on metals is UNKNOWN (no metals backtest yet)". This run
settles it.

**Verdict: GOLD earns trades. SILVER stays prediction-only.**

## Sample

31 resolved metals boards, **441 resolved legs** — larger than the entire day-1 backtest
(255 legs). Discovered today: metals list *both* monthly and **weekly** ladders, and the
weekly family had been invisible to us until now.

| | boards | legs |
|---|---|---|
| gold (XAUUSD) monthlies | Apr, May, Jun 2026 | 49 |
| gold weeklies | 13 boards, Apr 20 → Jul 20 | 182 |
| silver (XAGUSD) monthlies | Apr, May, Jun 2026 | 57 |
| silver weeklies | 13 boards, Apr 20 → Jul 20 | 153 |

Feeds: Pyth `Metal.XAU/USD` / `Metal.XAG/USD`, 1-min candles, backfilled 2026-04-01 →
2026-07-25 (continuous feeds — no per-contract delisting, unlike WTI). CLOB price history
at 60-min fidelity for all 569 legs of the metals + WTI-weekly set.

## Gate 0 — resolution reproduction

**440/441 (99.77%).** The single miss is instructive, not a modelling error — see
"Resolution epsilon" below. This validates, on metals specifically:

- the COMEX session model (6pm ET Sun → 5pm ET Fri, daily 5–6pm break) mapped to
  `Class::Wti` (22:00Z → 21:00Z business days);
- the **weekly** window fix shipped today (`board_period` now follows the asset's session
  clock: Sun 22:00Z → Fri 21:00Z for metals/WTI weeklies, not the equity Mon 00:00Z →
  Fri 20:00Z). Gold weeklies 42/42 and silver weeklies 42/42 on the July boards alone;
- the "after market creation" clause: every metals weekly from week-of-June-8 onward
  carries it (14/14 legs), earlier ones do not. The uniform
  `ws = max(period start, leg listing)` rule reproduces both regimes.

Cross-check: the same fix reproduces **28/28** on the two resolved WTI weeklies
(week-of-Jul-13, week-of-Jul-20) from our own archived **WTIU6 contract feed** — the first
time we have validated the contract archive (rather than the USOILSPOT proxy) against real
venue resolutions.

## Gate 1 — window-open calibration and Brier

Model-vs-market Brier, one observation per leg at window open (the honest unit):

| asset | n | market | RV model | model − market |
|---|---|---|---|---|
| gold | 217 | 0.1381 | **0.1192** | **−0.0189** |
| wti | 85 | 0.1019 | **0.0886** | **−0.0133** |
| silver | 196 | 0.1962 | **0.1903** | −0.0060 |
| btc | 50 | 0.1075 | 0.1118 | +0.0044 |
| eth | 21 | 0.1317 | 0.1401 | +0.0083 |
| nvda | 56 | 0.1052 | 0.1249 | +0.0197 |
| spy | 56 | 0.0884 | 0.1175 | +0.0291 |

Daily-checkpoint Brier tells the same story (gold 0.0761 vs market 0.0827; silver 0.1174
vs 0.1204; wti 0.0458 vs 0.0548; spy/nvda/btc/eth all worse than market).

**Gold's Brier margin is the largest of any asset we trade — larger than WTI's.** Silver's
is positive but small enough to be noise. Equity and crypto remain assets where the market
beats the model, exactly as day-1 found.

Favorite–longshot bias is textbook on metals (window-open bins):

| bin | n | avg mid | hit rate |
|---|---|---|---|
| 0–2c | 2 | 0.015 | 0.000 |
| 2–5c | 18 | 0.035 | **0.000** |
| 5–10c | 33 | 0.068 | **0.000** |
| 10–20c | 49 | 0.137 | 0.082 |
| 20–35c | 59 | 0.262 | 0.288 |
| 35–50c | 59 | 0.436 | 0.407 |
| 65–80c | 42 | 0.725 | 0.571 |
| 80–95c | 22 | 0.859 | **0.955** |
| 95–101c | 30 | 0.994 | 1.000 |

51 legs quoted under 10c, **zero** touched. The wing premium we harvest is present in
metals.

## Gate 2 — delayed-execution sell simulation

Daily 12:00Z checkpoints, cost 1.5c, edge threshold 4c, mid in 3–50c, 1-session-σ zone
excluded, fills at **t+24h** with frozen inputs:

| asset | n trades | avg net | se | win rate |
|---|---|---|---|---|
| wti | 140 | **+14.39c** | 2.13c | 94% |
| spy | 18 | +19.60c | 5.74c | 94% |
| **gold** | 174 | **+7.13c** | 2.61c | 86% |
| eth | 30 | +8.31c | 4.20c | 93% |
| btc | 79 | +6.35c | 2.93c | 89% |
| **silver** | 96 | **+2.95c** | 3.87c | 79% |
| nvda | 25 | −4.75c | 7.70c | 76% |

Collapsed to one observation per leg (checkpoints on the same leg are correlated, so this
is the conservative unit):

| group | legs | avg net | se | legs positive | boards positive |
|---|---|---|---|---|---|
| metals (gold+silver) | 127 | +3.78c | 3.27c | 102/127 | 21/29 |
| wti | 31 | +9.62c | 5.05c | 28/31 | 4/4 |
| equity | 29 | +5.01c | 6.61c | 24/29 | 6/8 |

## Verdict

- **GOLD → tradeable (sell-only, unchanged rules).** It clears the gate the day-2 note set:
  it beats market Brier by the widest margin of any asset (−0.0189 at window open), its
  delayed sell sim is +7.13c/trade at 2.7σ with an 86% win rate, and the wing overpricing
  that the method harvests is present. Gold moves from prediction-only to earning trades.
- **SILVER → stays prediction-only.** +2.95c/trade (se 3.87c) is 0.76σ, and its Brier
  margin (−0.0060) is inside the noise. Its edge is *not distinguishable from zero*.
  It is not a negative result — it is an underpowered one. Revisit after more resolutions.
- Capacity, not edge, is the binding constraint on gold: metals top-of-book was $0–$67
  across the July board on 2026-07-25. An earned edge on a $20 book is a prediction, not
  a trade. Size from book depth only.
- **No gold sell signal exists right now.** On the July board (6 days left) every fundable
  gold leg sits within a couple of cents of the model: H4300 q 0.043 vs mid 0.045, L3900
  q 0.186 vs 0.194, L3800 q 0.027 vs 0.025. Earning the right to trade an asset is not the
  same as having a trade. The weekly gold board is where this should pay off once it has a
  book.

## Resolution epsilon — a new, one-directional risk (found via day-3 gate-0 verification)

Across 760 clean-feed resolved legs, venue resolutions and our candle model disagree 4
times. The disagreements are **entirely one-directional**:

- Legs where our feed shows a **touch**: 279 of 279 resolved YES. Including 32 legs whose
  touch margin was under 0.5% of the barrier — **0 reversals**.
- Legs where our feed shows **no touch**: 4 of 481 resolved YES anyway, and they are
  concentrated in a thin band right below the barrier:

| feed fell short of barrier by | legs | resolved YES | rate |
|---|---|---|---|
| < 0.05% | 2 | 1 | 50% |
| < 0.10% | 7 | 2 | **29%** |
| < 0.20% | 17 | 2 | 12% |
| < 1.00% | 67 | 4 | 6% |
| any miss | 481 | 4 | 0.8% |

The two clean-feed cases: `will-spy-reach-750-by-july-20-2026` (Pyth SPY peaked at
**749.99002**, 1 cent short — verified against the 5-second Pyth tape, max 749.98993, and
against every Pyth aggregation from 1-min to daily; the board resolved YES and closed
16:41Z on 07-22, minutes after the 749.99 print) and
`will-xagusd-reach-69-by-june-8-2026` (feed peaked 68.942, 0.084% short, resolved YES).
Both are **↑ legs at round-number barriers**. The other two misses are WTI-May legs read
off the USOILSPOT proxy — a known proxy limitation, not a venue disagreement.

**Consequence for a sell-only book: the error is adverse.** Our "no touch" can become YES;
our "touch" never becomes NO. Expected drag is small in aggregate (≈0.8% × ~0.9 ≈ 0.7c per
sell, against a +5–14c edge) but it is concentrated precisely in near-money legs, which
carry the largest fills.

**Rule adopted:** do not open a sell whose barrier sits within **0.2% of the leg's running
window extreme** (computed from the leg's TRUE window start, not the board's — re-added
legs have private starts). Screened all 7 of today's signals: none trip it (closest is WTI
↓80 at 1.75%).

## Reproduce

```
ladderrv candles <data> XAUUSD 2026-04-01 2026-06-30     # and XAGUSD
ladderrv discover <data> <31 metals board slugs>
ladderrv clob <data> 60 <same slugs>
ladderrv analyze <data>                                  # gate0.csv, gate1_open.csv, gate2_*.csv
```
Frozen inputs: `data/backtest-metals-2026-07-25.tar.gz.r2.json`.
