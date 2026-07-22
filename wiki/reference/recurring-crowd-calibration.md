# Crowd-calibration test for recurring bucket markets

Before spending a research slot on a *recurring* bucket-family market (weekly tweet
counts, monthly prints), measure whether its crowd is already calibrated — a calibrated
recurring crowd is the strongest prior that window-open prices are fair.

## Method

1. For each **resolved** instance of the series, fetch the family's prices at **window
   start** (CLOB price history of the winning token).
2. Average the eventual winner's window-start price over instances.
3. Benchmark: under perfect calibration, E[winner price] = the family **Herfindahl**
   Σ pᵢ² of window-start de-vigged prices.
4. Mean winner price ≈ Herfindahl → calibrated. Substantially **below** → crowd
   overconfident (favorites win less than priced); **above** → underconfident.

poly's result on Musk weeklies: 0.100 vs benchmark 0.103 over 17 resolved weeks →
calibrated. Three research angles then failed to beat that market by more than a cent —
the test predicted the outcome of the whole run for a fraction of its cost.

## Caveats

- n≈17 rejects *gross* miscalibration only; don't read structure into ±0.01.
- Use de-vigged prices for the Herfindahl; raw mids carry 2–6% overround.
- Tail-floor artifact: far buckets can't trade below ~0.5c, so implied sd from de-vigged
  mids overstates true crowd sd — correct before calling the crowd's width wrong.
- A pass means "no edge at window-open", not "no edge ever" — mid-window news can still
  outrun a thin family's repricing. That *repricing lag* is itself a strategy-shaped idea.
