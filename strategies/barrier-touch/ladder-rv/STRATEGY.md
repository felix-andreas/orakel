# barrier-touch/ladder-rv

> Thesis (from `ideas/2026-07-23-hit-price-ladder-rv.md` — read it fully): price
> "Hit Price" one-touch ladders with a first-passage model (spot from the resolving
> Pyth feed, σ from listed options IV) and trade relative value: wing legs vs ATM
> (touch-vol smile), mid-window strike additions with private window starts, and
> payoff-dominance violations on weekly boards. Mispricing is model-revealed, not
> print-revealed — harvested by holding 1–9 days to resolution, no speed race.

## Method

DAY-1 STATE — to be established backtest-first per the idea's falsification sketch:
gate 0 resolution reproduction from Pyth Benchmarks 1-min candles (incl. the WTI
mid-window contract-roll case), window-open calibration, t+24h delayed-execution sim
with frozen inputs + barrier-proximity exclusion zone, wash/integrity checks. Keep
this section as-built.

## Applicability

A market fits when: it is a leg of a Hit Price ladder (any asset/tier) whose
resolution feed is Pyth, with quotes in the fundable 3–50c zone (sub-1c wings are
diagnostic, not tradeable — $3–20 top-of-book dust). Onboarding =
`applications/<market>.toml` with asset, pyth symbol, barrier, direction, window
(incl. true start — beware "after market creation" fine print), resolution date,
IV source.

## How to run

(to be written with the first working scripts in `src/`; the runningmax CLOB
pipeline in `strategies/temp-truncation/runningmax/src/` is directly reusable —
copy freely per CODING.md)

## Evidence

- (backtests land here as `results/…`)

## Changelog

- 2026-07-23 — variant created from the idea (run 2); slot 1 trial started.
