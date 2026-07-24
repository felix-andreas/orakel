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
