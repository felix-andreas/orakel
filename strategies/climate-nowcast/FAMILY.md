# climate-nowcast

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
  ERA5 daily 2m via a seasonal transfer model (trial, slot 2, started 2026-07-24).

Cross-variant lessons: (none yet)
