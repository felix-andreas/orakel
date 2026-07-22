# Execution

How a set of predictions becomes (paper) trades. Trading firms split this into signal
generation → combination → execution; orakel implements **two layers** and folds
combination into execution until overlapping strategies make it worth splitting out
(decision entry when that happens).

- **Generation** = `predictions/predictions.csv` (strategies produce probabilities).
- **Execution policy** = a versioned rule set that turns predictions into positions.

## Policies

One TOML per policy version in `policies/`, e.g. `policies/naive-threshold-v1.toml`:

```toml
name = "naive-threshold"
version = 1
created = ""

[combine]           # multiple variants covering one market → one house probability
method = "best-brier"   # best-brier | precision-weighted | single:<family>/<variant>

[entry]
min_edge = 0.05         # |prediction − market_price| to act
min_liquidity = 100.0   # $ within 5c of touch; skip hollow books
statuses = ["live"]     # which prediction statuses may trade (trial = shadow only)

[sizing]
method = "fractional-kelly"
kelly_fraction = 0.25
max_position = 50.0     # $ per market
bankroll = 1000.0       # paper bankroll

[exit]
take_profit_edge = 0.0  # exit when edge gone
hold_to_resolution = true
```

Policies are **backtestable**: replay a policy over the predictions log + market prices
(price at prediction time is in the CSV; fuller price history from R2 snapshots) →
fills, positions, equity curve, max drawdown. Costs must be modeled honestly: spread
crossing, slippage on thin books, and the fact that a hollow book's touch may sit far
from its mid.

The backtest engine is a Rust crate (to be built here as `backtest/`) whose core also
compiles to WASM for the dashboard's interactive policy page — one implementation, two
frontends. Results land in `results/<policy>/`.

Calibration (Brier, in `scoring/`) and profitability (PnL, here) are different questions;
a strategy is only *tradeable* when both look good.

**Real trading is out of scope** (CONSTITUTION.md §5) — this layer exists so that the day
Felix opts in, the policy that would trade has months of honest paper history.
