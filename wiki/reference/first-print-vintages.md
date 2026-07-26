# First-print vintages — score against what actually resolved

Index-print markets (GISTEMP, CPI, NSIDC, …) resolve on the **first published number**.
The data file you download today is the *settled, revised* series — and backtests scored
against it silently corrupt: GISTEMP revises by sd 0.019 °C, enough to flip the winning
bucket in **9 of 28** resolved monthly instances (gistemp-era5 kill, 2026-07-24).

Rules:

- Reconstruct resolution-time vintages before scoring anything. Wayback CDX over the
  resolution file's URL (`collapse=digest`) is cheap ground truth — 50 unique captures
  covered 26 months of GISTEMP.
- Verify the market actually resolved on the first print (gistemp family: 0/28
  mismatches — UMA resolves within hours, always before the next capture).
- Corollary for live prediction: the *revision noise* (first-print vs your target
  estimate) belongs in your σ in quadrature — the print you're forecasting is itself a
  noisy draw.
- Related: resolution feeds can be *deleted* (Pyth delists expired futures feeds —
  ladder-rv day 1). Archive the resolving feed while the market is live.

## "Machine-readable and public" does not mean "fixed" — check before you model

**The mandatory pre-modelling test, and it costs one afternoon:** take at least three
*already-settled* instances, rebuild them from the live feed, and check the answer matches
what the venue actually paid. If it does not, your backtest target is fiction and no amount
of modelling skill helps.

Measured 2026-07-26 (chokepoint-transit-ladders kill) on **IMF PortWatch** daily chokepoint
transit counts — a free ArcGIS feature layer, 2,757 days per chokepoint, exactly the
"boring, public, nobody's job" source we hunt for:

| week | settled on | live feed today | revision |
|---|---:|---:|---:|
| 2026-05-11..17 | **15** | 52 | **+247%** |
| 2026-06-08..14 | **18** | 44 | **+144%** |
| 2026-05-18..24 | 42 | 57 | +36% |
| 2026-06-29..07-05 | 225 | 205 | −9% |

Rebuilding all 19 resolved Polymarket boards from the live feed reproduces the **wrong
winning bucket on 7 of them (37%)**. And the two venues listing the identical contract
resolved the week of May 11-17 to **contradictory** values two days apart — Kalshi 15 on
May 19, Polymarket's 40-59 bucket on May 21, live feed 52 today.

This is the GISTEMP failure (sd 0.019 °C, 9 of 28 buckets flipped) an order of magnitude
larger, and it generalises to any feed built by an ingestion pipeline rather than published
as a fixed release — AIS/satellite derivations, scraped aggregations, anything that backfills.

Rules that follow:

1. **Reconstruct ≥3 settled instances from the live feed before building anything.** Report
   the match rate in the idea file. This is now a gate, not a nicety.
2. **Prefer sources whose vintages you can actually recover.** Wayback covers files and
   HTML pages; it does **not** cover API query endpoints. A source you cannot archive
   retroactively is a source you cannot backtest — say so and mark the idea `needs-gate-0`.
3. **Kalshi's `expiration_value` is a free vintage record** for any object it also lists —
   the exact integer the venue settled on, at a known date (see
   [sharp-line-screen](sharp-line-screen.md)). Cross it against the live feed; the gap *is*
   the revision.
4. **When the revision dominates, restate what you are forecasting.** On these boards the
   target was never "how many ships sail" — it was "what integer will an AIS pipeline have
   published by Tuesday". The sharpest venue's error at *window close*, with all seven days
   already elapsed, was still ±18.6 ships. That residual is the pipeline, and nobody can
   forecast it.
