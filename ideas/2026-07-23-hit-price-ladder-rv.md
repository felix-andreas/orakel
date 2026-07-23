---
date: 2026-07-23
slug: hit-price-ladder-rv
status: trialing # -> strategies/barrier-touch/ladder-rv (slot 1, 2026-07-23 run 2) # second idea of 2026-07-23 (extra cycle requested by Felix)
example_markets: ["what-price-will-wti-hit-in-july-2026", "will-spy-hit-week-of-july-20-2026", "will-nvda-hit-week-of-july-20-2026"]
---

## Thesis

Polymarket's "Hit Price" one-touch barrier ladders are a recurring cross-asset family:
**64 open events / 753 markets / $87.9M open-board volume** today, split daily (crypto,
$30k), **weekly (24 boards, $650k)**, **monthly (25 boards, $29.9M)**, annual ($57.3M —
avoid, slow). Assets: BTC/ETH/SOL/XRP/DOGE, ~18 US equities + SPY, WTI, gold, silver,
natgas. Each leg is an independent binary (negRisk=false): "will any 1-minute candle
High/Low touch $B during <window>", resolved on **Pyth prints** (equities: regular
trading hours only; WTI: active-month futures, session 18:00 ET prior day; crypto:
24/7). Weeklies resolve every Friday, monthlies at month-end.

The one-touch probability is a textbook object — driftless GBM gives
P(touch) ≈ 2·N(−|ln(B/S)|/σ√τ) — and *everything needed to price it is free and
read-only*: spot from the same Pyth feed that resolves the market (verified live:
Hermes API, WTI U6 $89.98 @ 09:18:29Z), σ from listed options IV (VIX/OVX/GVZ for
equities/commodities, Deribit DVOL for crypto). We can hold a full fair-value curve per
board; the crowd demonstrably does not. Three groups are on the wrong side:

1. **Wing lottery buyers** (favorite-longshot inside ladders,
   `wiki/reference/favorite-longshot-bias.md`). Today's WTI July board implies a
   monotone-rising touch-vol curve on the HIGH side: $95→~53%, $100→~59%, $105→~62%,
   $110→~69%, $120→~76%, **$130→~88%** — the wings price nearly double the
   at-the-money vol. The flow is real, not dust: BTC "reach $100k in July" (a ~0.15c
   moonshot) has **$1.9M lifetime / $95.8k 24h volume**.
2. **Board-title readers on extension legs.** Strikes are *added mid-window* as spot
   moves, and the fine print says "at any point **after market creation**" — an added
   leg carries a private window start. Live example: WTI printed ≤$80 on Jul 20 (the
   weekly $80-LOW leg is resolved 1.0), then rallied to $90; the *monthly* board then
   grew a "$80 (LOW) in July" leg with `startDate 2026-07-20T16:30Z` now trading
   0.25/0.26 — it prices a **re-touch**, but the board reads "in July". Anyone pricing
   the calendar month (either direction) is wrong by construction. Same trap class:
   equities' RTH-only touch means overnight/weekend extremes that reverse before the
   open **don't count** — a 24/7-crypto-calibrated crowd systematically overprices
   equity touch legs at the same headline vol.
3. **Stale weekly quoters / nobody enforcing coherence.** P(touch B) must be monotone
   in |B−spot| *for the remaining window* regardless of per-leg start dates (a future
   touch of the deeper barrier passes through the shallower one). The live SPY weekly
   board violates this outright and *stayed violated for ≥65 minutes across two
   independent snapshots* (08:11:21Z frozen scan vs 09:16:10Z CLOB books, quotes
   unchanged): LOW $715 mid 3.8c > $720 3.15c > **$725 2.05c** — the dominating $725
   claim asks 3.1c while the dominated $715 bids 2.0c. NVDA weekly: $188-LOW asks 1.1c
   while $184-LOW bids 1.1c — a zero-net-cost dominance spread. Caveat kept honest:
   touch size on these tail legs is $3–$20 (midpoint-artifact territory per
   `wiki/reference/thin-market-price-read.md`), so the violations are primarily
   **proof that no coherence-enforcing bot operates in these books** — the tradeable
   edge is the 3–50c zone priced off the IV anchor, where depth is real (WTI monthly
   $105-HIGH: 0.111/0.112 tick-wide, $5.2k bid depth within 5c; $110-HIGH $3.6k).

**Why they stay wrong for days, not minutes** (the speed-race screen, upfront): the
mispricing is a *level/allocation* error against an options-IV anchor plus fine-print
misreads. No public print ever reveals it — correcting it requires running a per-leg
windowed touch model, and the parties who own such models (options desks) cannot size
into $25k–$740k boards. It is harvested by **holding 1–9 days to resolution**, not by
racing an event. Measured: coherence violations persisted ≥65 min with unchanged books
(vs the weather kill's 0–3 min collapse). The one genuinely fast component — post-touch
convergence to 1.0 when spot crosses a barrier — is bot territory (runningmax lesson)
and is **explicitly excluded** from the claimed edge and from backtest P&L. Daily
prediction cadence suffices: fair value drifts with spot/IV on a days scale, entries
rest in the 3–50c zone, and a 24h-late fill changes entry by theta the sim can price.

## Example market(s)

All numbers fetched fresh 2026-07-23 09:15–09:20 UTC (scan baseline 08:11:21Z, frozen
at `roles/market-researcher/data/scans/2026-07-23-events-vol24.csv.r2.json`):

- **what-price-will-wti-hit-in-july-2026** — $7.78M board, 21 legs, $914k/24h, ends
  Aug 1 (9 days). Pyth active month $89.98. HIGH $95: 0.51/0.524; $100: 0.247/0.26;
  $105: 0.111/0.112 (spread 1 tick, $5.2k bid within 5c); $110: 0.053/0.074 (moved
  from 0.033/0.048 at 08:11Z as oil rallied — book does track spot at ~hour scale;
  it is the *smile*, not the level, we fade). LOW $80 (created Jul 20 16:30Z,
  re-touch leg): 0.25/0.26. Implied touch-vol wings 76–88% vs ~53% ATM.
- **will-spy-hit-week-of-july-20-2026** — $53.4k board, 14 legs, ends Jul 24 20:00Z.
  Monotonicity violation on the LOW side (715/720/725 inverted, numbers above),
  persistent ≥65 min. HIGH $755: 0.11/0.13, $6.2k volume.
- **will-nvda-hit-week-of-july-20-2026** — $24.2k board. $188-LOW 0.001/0.011 vs
  $184-LOW 0.011/0.032 (dominance violation); HIGH $216: 0.41/0.45, $220: 0.13/0.15.
- Monthly equity/commodity boards in the thin-to-mid band: gold $738k, silver $680k,
  SPY $402k, natgas $179k, NVDA $78k, MSFT $73k — plus BTC $14.7M / ETH $3.2M
  (deeper; use as model-calibration checks, trade only their 2–20c wings).

Resolved backtest supply (verified closed via Gamma): WTI May monthly $40.2M/30 legs;
BTC June monthly $25.2M; SPY week-of-Jul-13 $79.6k/14 legs; NVDA week-of-Jul-13
$36.1k/14 legs. Monthlies run back months across ~25 assets; weeklies every Friday.

## Falsification sketch

Data: CLOB `prices-history` per leg (runningmax pipeline reuses directly), per-leg
`startDate` from Gamma (load-bearing — extension legs), underlying 1-min candles
(Binance klines for crypto; Pyth Benchmarks / exchange candles for equities+futures —
**first derisk step: confirm historical 1-min availability for Pyth symbols**), IV
anchors (VIX/OVX/GVZ/DVOL histories, all public).

1. **Gate 0 — resolution reproduction:** reproduce ≥99% of resolved leg outcomes from
   candle data respecting per-leg creation windows, RTH/session rules, and the WTI
   active-month roll (Q6 expired Jul 21 *mid-window*). Kill if we can't — fine print
   misunderstood.
2. **Gate 1 — speed screen, measured (mandatory per
   `wiki/reference/delayed-execution-test.md`):** on ≥8 resolved weekly boards,
   measure lifetime distribution of (a) ≥1c monotonicity violations, (b) ≥3c
   model-vs-market gaps at daily checkpoints. **Kill if p50 lifetime is minutes, not
   hours** — that would mean coherence bots exist after all. Also record executable
   size behind violations; p50 < $50 downgrades mechanism 3 to diagnostic-only.
3. **Gate 2 — main sim with delayed execution:** daily 12:00Z checkpoints on resolved
   boards; trade legs with mid in 3–50c where |model − market| > spread + 2c; hold to
   resolution; **exclude any entry whose barrier is within 1 remaining-day sigma of
   spot** (that zone is the touch race, not ours). Re-run all fills at **t+24h with
   model inputs frozen at t**. **Kill if** delayed-execution avg net < +2c/trade
   (≈ half typical touch spread) or the sign flips across sample halves
   (June vs July), per the wiki's rules of thumb.
4. **Gate 3 — jump-premium check:** attribute delayed-sim P&L; if losses on
   short-wing positions during touch events ≥ wing premium collected, the "lottery
   premium" is fair jump compensation, not bias. Kill.
5. **Gate 4 — integrity:** wash-test 2–3 boards (`wiki/reference/wash-trading.md`);
   check wallet concentration on weekly boards — if one MM wallet is the entire book,
   "stale quotes" may be adversarially refreshed against takers.

Capital note: shorting a wing ties ~$1/share; sub-1c legs are unfundable (0.15c over
9 days ≈ 6% annualized ceiling) — the 3–50c band requirement is structural, not taste.

Fast scoring: a 2-week trial sees ~24 weekly boards resolve each Friday, crypto
dailies every day, plus the Jul-31 month-end batch across ~25 assets.
