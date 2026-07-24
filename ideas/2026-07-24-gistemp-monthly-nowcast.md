---
date: 2026-07-24
slug: gistemp-monthly-nowcast
status: trialing # -> strategies/climate-nowcast/gistemp-era5 (slot 2, 2026-07-24)
example_markets:
  [
    "july-2026-temperature-increase-c-20260608140824583",
    "2026-july-1st-2nd-3rd-hottest-on-record-20260706144334512",
  ]
---

## Thesis

Polymarket's recurring monthly climate family — "«Month» 2026 Temperature Increase (ºC)",
plus sibling ranking events — resolves on a single cell of a primary source: NASA
**GISTEMP v4** LOTI, table `GLB.Ts+dSST.txt`, row 2026 / column of the month, in 0.01 °C
vs the 1951–1980 baseline. The month's outcome is *nowcastable to ±0.05 °C weeks before
the print* from a second primary source the crowd does not systematically consume:
**ECMWF Climate Pulse ERA5 daily global 2 m temperature** (CSV, 2–3 day lag). By day ~21
of the month, ~⅔ of the resolution variable is already realized; after month-end the
only residual is the ERA5→GISTEMP transfer — and the market stays open through that
window (July 2025 instance: trading end nominally Jul 31, `closedTime` Aug 8).

The crowd's failure mode is not the mode — it's **overconfidence in the mode**. Today
the market has the same modal bucket as our nowcast but prices it at 72–83c, implying a
nowcast s.d. of ~0.02 °C. The honest, measured error stack (rest-of-month persistence +
transfer-offset drift) is **σ ≈ 0.056 °C**: a 30-month hindcast (2024-01…2026-06,
point-in-time day-21 nowcasts) puts only **6/30** months within ±0.025 °C of the
nowcast. Fair modal-bucket probability is ~31–37%, and the two adjacent buckets (priced
at 4.9c and 20c asks) carry most of the missing mass. Who's on the wrong side: (a)
mode-pilers anchoring on "same as last month / last year"; (b) headline consumers who,
if they use ERA5 at all, map it to GISTEMP naively — the GISTEMP−ERA5 offset is
**non-stationary** (2026 offsets run 0.57–0.79 vs a 2015–2025 July mean of 0.54; ERSST
vs OSTIA divergence + GISTEMP polar interpolation), and it has month-of-year seasonality
(July runs ~0.02–0.04 below June). Modeling that offset properly is the entire game, and
it is invisible to anyone reading either source alone.

## Example markets (numbers pulled 2026-07-24, ~10:00Z)

**July 2026 Temperature Increase (ºC)** — event volume $40,031, liquidity $8,670,
resolves on GISTEMP print expected ~Aug 8–12 (2025 analog: Aug 8). Books (bid/ask):

| bucket    | market bid/ask | model P |
| --------- | -------------- | ------- |
| <1.10     | 0.001 / 0.003  | ~0.01   |
| 1.10–1.14 | 0.001 / 0.009  | ~0.06   |
| 1.15–1.19 | 0.036 / 0.049  | 0.15–0.24 |
| 1.20–1.24 | 0.72 / 0.83    | 0.31–0.37 |
| 1.25–1.29 | 0.14 / 0.20    | 0.25–0.32 |
| >1.29     | 0.001 / 0.002  | ~0.08   |

Model, fully reproducible from today's frozen snapshots: ERA5 July MTD (21 days) =
+0.632 vs 1991-2020, last five dailies rising 0.568→0.785; persistence regression on
1980–2025 Julys projects full-month ERA5 = **0.629 ± 0.032**. Transfer: June-2026
offset 0.621 + mean Jul−Jun delta −0.023 (sd 0.041) → 1.226; year-effect variant →
1.216; ensemble **1.221 ± 0.052**, hindcast bias +0.025 (se 0.010, n=30) → adjusted
center **~1.246, σ 0.056**. Trades: sell 1.20–1.24 at 0.72 bid, buy 1.25–1.29 at 0.20
ask and 1.15–1.19 at 0.049 ask — 15–50c of model edge per leg. Capacity is the honest
weakness: top-of-book $15–$300 per line within 5c (1.25–1.29 ask 0.20 × 1440 is the
deepest); this is a low-hundreds-$ book today, though resolved instances did
$0.38M–$4.9M lifetime volume, so flow arrives late in the window.

**2026 July 1st/2nd/3rd hottest on record?** — same resolution cell, $108,434 volume,
penny-to-4c spreads. "1st hottest" bid 0.94 / ask 0.95 requires reported ≥ 1.20 (prior
top Julys: 2024 = 1.20, 2023 = 1.19; ties qualify). Model P(≥1.20) = 0.68–0.82
depending on bias treatment → the 0.94 bid is a sell under any version of our σ. This
event is where the size is.

Recurring supply: ≥20 resolved monthly instances back to May 2024 (public-search
verified), a fresh instance each month, plus ranking events and the annual
"Where will 2026 rank among the hottest years on record" (slow, Jan 2027 — same
pipeline prices it for free; secondary application only, per fast-resolution
preference).

Sources verified read-only from this box today (both frozen to R2,
`roles/market-researcher/data/sources/2026-07-24-*.r2.json`):
`https://data.giss.nasa.gov/gistemp/tabledata_v4/GLB.Ts+dSST.csv` (June 2026 = 1.18)
and `https://sites.ecmwf.int/data/climatepulse/data/series/era5_daily_series_2t_global.csv`
(updated 23 Jul, dailies through Jul 21, status flags final/preliminary).

## Speed screen (upfront, per wiki/reference/delayed-execution-test.md)

**Edge horizon: 8–19 days.** No public print reveals the mispricing before resolution
itself — the GISTEMP release *is* the resolution, so there is nothing to race after it.
The edge is model-revealed (transfer regression + daily ERA5 accrual on a 2–3 day lag)
and harvested by holding to resolution. The richest sub-window — month-end to print
(~Aug 1–8), when ERA5 is final and only transfer residual remains — is a *week* long
and was tradeable in every 2025 instance checked. Today's 11c-wide modal spread on a
quiet book is not a speed race by construction; a t+24h delayed-execution sim is still
mandatory in the backtest (below) as discipline.

## Falsification sketch

On the ≥20 resolved instances (May 2024 – Jun 2026), reconstruct point-in-time nowcast
distributions at day-14 / day-21 / month-end / pre-print (ERA5 daily series is
append-only with status flags; ground truth = each market's actual resolved bucket, not
today's revised GISTEMP file), pull CLOB `prices-history` mids at the same checkpoints,
then:

1. **Skill test:** model log-loss / Brier vs de-vigged market at each checkpoint.
   *Kill if the market beats the model* — then the crowd already prices ERA5 properly.
2. **Overconfidence test:** empirical hit rate of the market's modal bucket vs its
   price at day-21 and pre-print. *Kill if hit rate ≈ price* (crowd's σ is right, ours
   is inflated).
3. **PnL sim with delayed execution:** trade |model−mid| > spread+2c legs, fills at
   t+24h mids with inputs frozen at t, hold to resolution. *Kill if ≤ 0 or
   sign-flip across halves* (per delayed-execution-test).
4. **Vintage integrity:** GISTEMP revises history; the market resolves on the
   first-published figure. Spot-check ≥6 months against web-archive vintages of
   `GLB.Ts+dSST.csv`. *Kill (or rebuild on vintages) if first-print vs current file
   would have flipped any resolved bucket.*
5. **Capacity floor:** fillable size at edge prices across a month's window from the
   tape (wiki/reference/thin-market-price-read.md discipline, wash-check the volume).
   *Kill if < $50/instance actually fillable* — edge without size isn't worth a slot.
