# Execution

What our signals would have earned. Design and the accounting rules: **[DESIGN.md](DESIGN.md)**
(read it first — the capital-lockup rule in §3 is what makes these numbers honest).

```
execution/
├── DESIGN.md            # the model, the accounting rule, the cost model, the honesty rules
├── policies/*.toml      # the eight named execution strategies (versioned, never edited)
├── signals/*.csv        # canonical signal sets (policy x SET is the unit of evaluation)
├── engine/              # the simulator (Rust)
└── results/             # per set/policy JSON + summary.csv (the matrix)
```

## The eight policies

| Name | Character |
|---|---|
| `mirror` | Every signal, flat stake, no gates. The naive baseline. |
| `gate` | Discipline only (edge/spread/depth/epsilon), still flat. Isolates the value of filtering. |
| `kelly` | Gate + quarter-Kelly sizing. Does conviction-sizing pay? |
| `anchor` | Kelly + capacity realism (depth and per-market caps). |
| `fade` | Anchor, sell-side only — the house finding. |
| `patient` | Fade with 24h delayed entry — the wing-drift finding. |
| `sniper` | Top-decile edge, double size, few trades. Concentration vs breadth. |
| `harvest` | Fade with early exit at 60% edge closure. Is patience paid for? |

## Signal sets

- `orakel-live` — our own predictions ⋈ resolutions. The real thing; small and growing.
- `ladder-rv-hist` — the ladder-rv variant's resolved-leg checkpoints. One regime, but
  large enough to separate policies.

## Running it

```sh
cd engine && cargo build --release && cargo test
target/release/engine run --set all --policy all      # writes results/ + summary.csv + SUMMARY.md
```

The matrix lives in **[results/SUMMARY.md](results/SUMMARY.md)**. How the sets
were built, and every interpretive call behind them:
[signals/README.md](signals/README.md). Engine defaults and mechanics:
[engine/README.md](engine/README.md).

## Policy versions

Policies are versioned and **never edited** (DESIGN.md §5); results are written per
version to `results/<set>/<policy>-v<n>.json`, so an old number always stays traceable
to the file that produced it.

- **`-v1`** — the first matrix. Charges **no venue fee**, which was wrong. Kept only so
  earlier reports remain attributable.
- **`-v2`** — identical in every respect except that it charges Polymarket's real taker
  fee (`shares × rate × p × (1 − p)` per fill; DESIGN.md §4.4). **Read conclusions off
  v2.** SUMMARY.md leads with an explicit v1→v2 before/after and re-checks each
  conclusion the firm has on record.

## The number that decides

Not cents per trade — **annualized return on locked capital**. Selling a 15c wing and
buying a 97c favourite can both show "+10c/trade" and be completely different businesses,
because the second one ties up 97c to earn it. See DESIGN.md §3.

Real trading remains out of scope (CONSTITUTION.md §5). This is the instrument that tells
us which beliefs survive a spread, before any money is at risk.
