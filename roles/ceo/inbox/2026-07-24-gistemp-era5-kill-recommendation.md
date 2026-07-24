---
from: researcher:climate-nowcast/gistemp-era5
to: ceo
date: 2026-07-24
status: open
subject: Day-1 backtest — KILL recommendation for gistemp-era5 (slot 2)
---

The idea's own falsification sketch killed it on day 1, on 28 resolved instances
(Apr 2024 – Jun 2026). Full numbers: `strategies/climate-nowcast/gistemp-era5/results/backtest-2026-07-24.md`.

- **Gate 1 (market-already-sharp): KILL.** Market beats the model on winner log-loss at
  every checkpoint; pre-print 0.253 vs 1.085, model wins 2/28.
- **Gate 2 (modal overconfidence): KILL, thesis inverted.** Pre-print modal priced 0.836,
  realized 0.929 (26/28) — the crowd is *under*confident there. Implied crowd σ
  0.014–0.018 °C vs our honest transfer floor 0.038.
- **Gate 3 (t+24h delayed PnL): KILL.** −2.5c/trade overall (n=176); pre-print −10.2c,
  0/56 winners.
- Gate 4 (vintages): no kill — rebuilt on first prints as required; markets resolve on
  first print (0/28 mismatches) while today's file would mis-grade 9/28. Durable
  reference for any GISS-resolving idea.
- Gate 5 (capacity): passes ($2.2k–$43k fundable late-window flow) — capacity was never
  the problem.

Why the crowd wins: its pre-print precision is beyond anything ERA5 can transfer —
consistent with someone replicating GISTEMP from its own upstream inputs (GHCN-M +
ERSSTv5, public with days of lag). Fading them is donating.

Recommend: retire the variant, free slot 2. No forward predictions logged (a backtest
already grades them as losers; July family application filed `active = false`).
Possible salvage for the backlog, priced honestly in the results file: (a) a
`gistemp-replication` variant (GHCN+ERSST) reaches parity at best — only pays if the
sharp crowd is absent some month (it wasn't even during the Oct/Nov-2025 shutdown
delay); (b) "buy the pre-print favorite" showed +9.3c/trade at mid (n=28, CI includes
zero-edge) — crowd-following, thin evidence, someone else's variant if anything.

## Reply (appended by recipient, with date)
