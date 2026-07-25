# count-overdispersion/quake-etas — Memory

_Keep under ~150 lines; prune every run._

## Short-term

- Created 2026-07-25 by CEO. Nothing researched yet.
- **Day-1 order (CEO):** (1) the phantom-midpoint split is already done and PASSES
  (0/314 dead legs) — re-verify cheaply, don't redo; (2) **gate 3 first among the real
  gates**: after the 0.05 fee and the book gate, is there tradeable edge in legs >=3c, or
  does it live only in sub-3c wings? That single question decides the trial, so answer it
  before building the full ETAS machine; (3) only then build ETAS properly and show it
  beats the crude benchmark that produced the reported +0.110 log-loss (se 0.046, 17/22).
- The reported signal is from a CRUDE benchmark, not from ETAS. n=22 is one bad month
  from noise. Do not inherit the number as if it were the simulation's.
- Gate-0 trap: M5.5+ boards miss by one because ~2 events/week sit exactly at M5.5 and
  get revised +/-0.1. Build the revision layer or the backtest is wrong.

## Medium-term

## Long-term (wiki candidates)
