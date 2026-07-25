# climate-nowcast

> **In plain English:** Bets that settle on a published climate figure — a monthly global temperature, a sea-ice minimum. The number is not known in advance, but the data that determines it is published as the month goes along.

Nowcast the exact climate-index print a market resolves on, from higher-frequency
upstream data the crowd doesn't systematically consume. The edge is a *data pipeline*,
not structure: the resolving print (GISTEMP cell, NSIDC minimum, …) is predictable to
known error bars days-to-weeks ahead via reanalysis feeds (ERA5 et al.) plus a modeled
transfer function — while crowds price the modal bucket with overconfident width.

Born from `ideas/2026-07-24-gistemp-monthly-nowcast.md` (Felix directive:
market-specific data-source strategies). Poly heritage: the hottest-year-2026 GISTEMP
work. Future candidates in the same spirit: NSIDC Arctic sea-ice minimum, hurricane
landfall (NHC).

Variants:

- [`gistemp-era5/`](gistemp-era5/) — monthly GISTEMP LOTI bucket families nowcast from
  ERA5 daily 2m via a seasonal transfer model (trial, slot 2, started 2026-07-24;
  day-1 backtest KILL recommendation same day — see its `results/`).

Cross-variant lessons:

- **Ask "who is the sharpest agent already in this market?" before building.** The
  GISTEMP family is priced pre-print at σ ≈ 0.014–0.018 °C — only achievable by
  replicating the index from its own upstream inputs (GHCN-M + ERSST). A *proxy* feed
  (ERA5 reanalysis) has an irreducible transfer floor (~0.038 here); if the incumbent
  crowd plausibly runs the *primary* inputs, a proxy-based nowcast is structurally
  dominated regardless of how well it's built. Future nowcast variants should target
  prints where the primary inputs are NOT public ahead of the print, or where no one
  credibly runs them.
- **Resolution = first print, never the settled series.** GISTEMP revisions (sd 0.019,
  bucket-flips in 9/28 instances) would silently corrupt any backtest scored on the
  current file. Wayback vintages of the resolution file are cheap ground truth.
