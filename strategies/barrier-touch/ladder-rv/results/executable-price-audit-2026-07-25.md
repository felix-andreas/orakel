# Executable-price audit of the first scored batch

**Date:** 2026-07-25 · **Run by:** CEO · **Model:** claude-opus-5 · **Tool:**
`tools/fillcheck` (new) → `predictions/fills.csv`

## Question

On 2026-07-24 this variant's first 21 predictions were scored and **all 21 beat the
market** on paired Brier (mean improvement +0.000945). Before that number is allowed to
mean anything, one thing has to be checked: `market_price` in our ledger is a CLOB
**midpoint**, and a midpoint on a wing leg is the average of a near-zero bid and a fat
ask. Was it a price anyone would have traded with us at?

## Method

`tools/fillcheck` replays Polymarket's public Data API trade feed for every market we
predicted on. The feed reports the *taker* side, so a taker who sold Yes at q proves a
resting **bid** at q existed — a price a seller could have hit. Trades on the No token
are folded into Yes-equivalent units so one pass answers both legs. For each prediction
row it records the best bid and best ask observed within 1h, 24h and the market's whole
remaining life, plus the notional that changed hands at a price at least as good as our
midpoint.

`scoring/` now joins that file and reports `n_fillable` and `exec_edge` alongside every
Brier aggregate.

## Result

| | rows |
|---|---|
| beat the market on paired Brier | **21 / 21** |
| a bid at-or-above the midpoint we were scored against, ever | **2 / 21** |
| …within the first hour, when we would actually have traded | **1 / 21** |

Summed midpoints **1.335** versus summed best-observed bids **1.037** — 78%, and that
is generous: it gives every row its best price over the market's entire remaining life
rather than at the moment we spoke.

Per row (mid → best bid ever seen; `—` = no bid trade at any price):

| market | mid | bid 1h | bid ever | $ at-or-above mid, 24h | improvement |
|---|---:|---:|---:|---:|---:|
| will-wti-dip-to-90-in-july-2026 | 0.8200 | 0.8300 | 0.9990 | 33,962 | +0.00223 |
| will-spy-dip-to-720-by-july-20-2026 | 0.0320 | — | 0.0401 | 4 | +0.00102 |
| will-spy-dip-to-725-by-july-20-2026 | 0.0685 | — | 0.0371 | 0 | +0.00435 |
| will-nvda-dip-to-196-by-july-20-2026 | 0.0455 | — | 0.0276 | 0 | +0.00200 |
| will-spy-dip-to-715-by-july-20-2026 | 0.0380 | — | 0.0200 | 0 | +0.00144 |
| will-spy-reach-755-by-july-20-2026 | 0.0405 | — | 0.0200 | 0 | +0.00162 |
| will-nvda-dip-to-192-by-july-20-2026 | 0.0395 | — | 0.0109 | 0 | +0.00155 |
| will-nvda-reach-224-by-july-20-2026 | 0.0315 | — | 0.0103 | 0 | +0.00098 |
| will-nvda-dip-to-176-by-july-20-2026 | 0.0115 | — | 0.0100 | 0 | +0.00013 |
| will-spy-dip-to-720-by-july-20-2026 | 0.0300 | — | 0.0100 | 0 | +0.00090 |
| will-spy-dip-to-715-by-july-20-2026 | 0.0330 | — | 0.0081 | 0 | +0.00109 |
| will-spy-dip-to-710-by-july-20-2026 | 0.0110 | — | 0.0060 | 0 | +0.00012 |
| will-spy-reach-770-by-july-20-2026 | 0.0150 | 0.0030 | 0.0030 | 0 | +0.00022 |
| will-nvda-reach-228-by-july-20-2026 | 0.0205 | — | 0.0030 | 0 | +0.00042 |
| will-nvda-dip-to-180-by-july-20-2026 | 0.0110 | — | 0.0030 | 0 | +0.00012 |
| will-spy-reach-775-by-july-20-2026 | 0.0125 | — | 0.0020 | 0 | +0.00016 |
| will-spy-reach-760-by-july-20-2026 | 0.0255 | — | 0.0012 | 0 | +0.00065 |
| will-nvda-dip-to-184-by-july-20-2026 | 0.0125 | — | 0.0010 | 0 | +0.00016 |
| will-spy-reach-765-by-july-20-2026 | 0.0075 | 0.0010 | 0.0010 | 0 | +0.00006 |
| will-nvda-dip-to-188-by-july-20-2026 | 0.0060 | — | — | 0 | +0.00004 |
| will-nvda-dip-to-192-by-july-20-2026 | 0.0240 | — | 0.0109 | 0 | +0.00058 |

## What this means

**The batch splits cleanly in two.** The single WTI monthly row is a real market: $34k of
volume, a bid above our price inside the first hour, a trade we could genuinely have done.
It is also the row where we had essentially no edge — we said 0.8263, the market said
0.82. It contributed 11% of the batch's total improvement.

The other 20 rows are SPY and NVDA **weekly** wings. They dominate the headline (89% of
the improvement) and not one of them had a bid at our midpoint in the first hour.
`will-spy-reach-760` was scored at a 2.55c midpoint against a best-ever bid of 0.12c —
a 21× overstatement of the sellable price. `will-nvda-dip-to-188` never traded on the bid
at all.

So: **the forecasting claim survives and the trading claim does not.** We said ~0 and the
answer was ~0, twenty-one times; that is a genuine calibration result and it is the right
thing to have found on day one. What it is not is money, and the previous headline
implied otherwise.

This independently corroborates the execution engine's finding from the other direction:
on `orakel-live` signals, seven of the eight execution policies took **zero** trades. Two
different methods, same conclusion — this variant's demonstrated edge lives where the
liquidity isn't.

## The limit of this evidence

`fillcheck` sees trades, not orders. A resting bid nobody hit leaves no trace, so 2/21 is
a **lower bound** on fillability. The true answer needs the book at prediction time, not
a reconstruction — see the actions below.

## Actions

1. **Weekly equity ladders are demoted to research-only for this variant.** They may
   still be predicted on (calibration data is cheap and useful) but no weekly-equity row
   may appear in a promotion case or a headline number without its own fill evidence.
   The monthly commodity boards — WTI, gold, silver — are where the volume is, and they
   are the trial's real evidence. They resolve 2026-07-31.
2. **The trial review on 2026-08-02 uses `exec_edge`, not `improvement`, as the promotion
   criterion.** `success_guideline` in `strategy.toml` is amended accordingly.
3. **Record the book.** `bid`/`ask`/`depth_usd` columns on prediction rows, sourced from
   the hourly snapshot worker. Reconstruction is a stopgap.
4. Durable rule written up as `wiki/reference/midpoint-is-not-a-fill.md`.

## Reproducing

```sh
cargo run --release --manifest-path tools/fillcheck/Cargo.toml   # writes predictions/fills.csv
cargo run --release --manifest-path scoring/Cargo.toml           # joins it, prints the fillable column
```

`predictions/fills.csv` is committed and is the snapshot — the trade feed is append-only,
so past trades are stable, but the file is what this document's numbers were computed
from.
