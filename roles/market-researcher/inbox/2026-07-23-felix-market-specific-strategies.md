---
from: felix
to: market-researcher
date: 2026-07-23
status: done
subject: Next idea: NOT market-agnostic — use market-specific data sources
---

Felix's directive for the next idea (tomorrow's run):

> Try to come up with a strategy that is not market agnostic. We try to generate an
> edge by using market-specific data sources, including but not limited to Polymarket.

Interpretation: both strategies so far exploited *cross-market structure* (truncation
mechanics, ladder relative value). The next idea should generate edge from **deep,
domain-specific data sources** for a particular market topic — external feeds, domain
databases, primary sources that the crowd doesn't systematically consume (poly's
heritage examples: EDGAR filings for M&A markets, GISTEMP for climate, F6 forms for
weather stations, ensemble forecasts). Pick the topic where the data advantage is
largest, not the structure that generalizes widest. Family/variant taxonomy handles
specificity fine — a variant serving ONE market is legal (strategies/README.md).

## Reply (appended by recipient, with date)

**2026-07-24, market-researcher:** Done — filed
`ideas/2026-07-24-gistemp-monthly-nowcast.md`, and it is the direct heir of the poly
heritage you cited: Polymarket's recurring monthly "Temperature Increase (ºC)" family
resolves on one cell of NASA **GISTEMP** (`GLB.Ts+dSST.txt`), and **ERA5 daily global
2m temperature** (ECMWF Climate Pulse CSV, 2–3 day lag, verified read-only from our
box) nowcasts that cell to ±0.05 °C weeks before the print. The modeled step the crowd
skips is the ERA5→GISTEMP transfer: the offset is non-stationary (2026 runs ~0.06 hot
vs the historical July mean) and seasonal. Today's July market agrees with our modal
bucket but prices it 72–83c where a 30-month hindcast (σ = 0.056 °C, only 6/30 within
±0.025) says ~31–37% — the adjacent buckets at 4.9c/20c asks carry the missing mass,
and the $108k "July 1st hottest?" sibling bids 0.94 where the model says 0.68–0.82.
Speed screen passes by construction: the resolving print IS the resolution, nothing to
race; edge horizon 8–19 days incl. a week-long post-month window that stayed tradeable
in 2025 instances. Honest weaknesses, in the file as kill conditions: thin current
books ($15–300/line top-of-book; family did $0.4–4.9M/instance historically), possible
sharp ERA5 users already present (test 1/2), GISTEMP vintage revisions vs backtest
(test 4). One market family, one data pipeline — exactly as directed.
