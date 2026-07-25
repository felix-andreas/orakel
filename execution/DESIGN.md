# Execution — design

Signals say *what we believe*. Execution says *what we would have done about it, and what
that would have earned*. This document defines the model; `policies/` holds the named
strategies; `engine/` implements the simulator; `results/` holds the output.

Authored by the CEO, 2026-07-25.

## 1. The three layers, and where we sit

Trading firms split **signal generation → signal combination → signal execution**. orakel
implements generation (strategy variants) and execution; **combination is folded into
execution** as its first step (`[combine]` in every policy) until enough variants overlap
on the same market to justify its own layer.

## 2. Core objects

**Signal** — one row: `(t, market, outcome token, our probability p, market price m,
book state, eventual resolution)`. Every prediction in `predictions/predictions.csv` is a
signal; so is every historical checkpoint inside a variant's backtest.

**Signal set** — a named, frozen collection of signals with known outcomes. Policies are
evaluated **policy × signal set**, because "which execution strategy is best" is
meaningless without saying *on whose signals*. Current sets:

| Set | Source | Size | Character |
|---|---|---|---|
| `orakel-live` | our own `predictions.csv` ⋈ `resolutions.csv` | small, growing | the real thing; the only set with no hindsight in the signals |
| `ladder-rv-hist` | ladder-rv's resolved-leg checkpoints (R2-frozen) | hundreds | one variant, one regime; rich enough to separate policies |

**Policy** — a named, versioned TOML: selection → sizing → entry → exit → costs, plus a
combination rule. Named because we will talk about them constantly; a policy is a
*character*, not a parameter dump.

## 3. The accounting rule that decides everything

A binary outcome token trades in [0,1]. **Selling YES at price p is economically buying NO
at 1−p: you receive p and must post 1−p as collateral until resolution.** So:

- **Profit per share** on a winning wing sale = p (the premium), not "10 cents of edge".
- **Capital locked** = (1−p) per share, for the *whole* holding period.
- Therefore the honest metric is **return on locked capital, annualized** —
  `pnl / (capital_locked × days_held) × 365`.

Why this is the crux: "+10c per trade" is a *fine* number for selling a 15c wing (locks
85c for 8 days → 11.8% over the hold, ~537% annualized) and a *much* weaker one for
buying a 97c favourite (+2c locks 97c for 6 days → 2.1% over the hold, ~125% annualized
before the spread, which on those books eats most of it). Corrected 2026-07-25: the
first version of this paragraph mis-stated both figures by a factor of ten; the formula
above is the authority and the engine implements it as written. Cents-per-trade
flatters exactly the trades we should refuse. **Every policy result reports cents/trade,
return on locked capital, and annualized return on locked capital together — and the
third one is the one that decides.**

Secondary but mandatory: `capital_efficiency` = mean fraction of bankroll actually
deployed. A policy that earns 40% annualized on 3% of the bankroll is a rounding error on
the fund, and the matrix must show that plainly.

## 4. Cost model — conservative by construction

Our own wiki says thin-market midpoints are decorative
(`wiki/reference/thin-market-price-read.md`). The simulator therefore:

1. **Never fills at mid.** Buys lift the ask, sells hit the bid. If only a mid is
   available for a historical signal, apply `assumed_spread` (policy field, default 3c)
   symmetrically and record the row as `synthetic_fill = true`.
2. **Caps size by visible depth** within 5c of touch (`max_book_fraction`, default 0.25).
   Depth below `min_stake` ⇒ the signal is **unfundable**, counted and reported, never
   silently dropped.
3. **Walks the book** when the stake exceeds top-of-book (with real book data), else
   applies a linear slippage penalty.
4. **Charges the venue's real taker fee.** *Corrected 2026-07-25: the first version of
   this clause said Polymarket charges nothing. It does.* Since 2026-01-05 it charges
   takers, per fill:

   ```
   fee_usdc = shares × rate × p × (1 − p)
   ```

   `rate` is per market category, read off each market's own `feeSchedule` on the Gamma
   API: **crypto 0.07 · sports 0.05 · economics/culture/weather/other 0.05 ·
   finance/politics/tech/mentions 0.04 · geopolitics 0**. The dollar fee is symmetric
   about p = 0.50 and *peaks* there (1.25c/share at rate 0.05) — i.e. it is largest
   exactly in the 3–50c band every policy targets. Makers pay nothing, but every fill
   this simulator takes crosses the spread, so every fill is a taker fill.

   The fee is charged **on entry, and again on an exit taken in the market**. Settlement
   at resolution is a redemption, not a match, and costs nothing — so a held position
   pays once and a round trip pays twice. Fees enter `pnl`; `capital_locked` stays
   collateral only, so the fee moves the numerator of every return and leaves the
   denominator alone.

   Policies declare this as `costs.fee_model = "polymarket-taker"` plus a `fee_rate`
   table and an `asset_category` map. `fee_model = "none"` (the default, and what every
   `-v1` policy carries) charges nothing and every such result **says so in its notes**:
   a fee-free number must never be mistakable for a costed one. An asset with no
   category mapping is charged `fee_rate_default` and **counted** (`fee_rate_unmapped`),
   never silently freed.
5. **Never trades inside the venue-epsilon** (`wiki/reference/venue-resolution-epsilon.md`):
   no sell of a barrier within 0.2% of its running extreme.

A policy that looks good only under `synthetic_fill` is flagged, not celebrated.

## 5. The named policies

Eight, spanning the space from "no discipline" to "our current house style". The first two
exist to make the others prove they add value.

| Name | One-line character |
|---|---|
| **`mirror`** | Trade every signal with any edge, flat stake, hold to resolution. The naive baseline. |
| **`gate`** | Same, but only through the discipline gates (edge, spread, depth, epsilon). Isolates how much value is *filtering alone*. |
| **`kelly`** | Gate + quarter-Kelly sizing on the executable price. Does conviction-sizing pay? |
| **`anchor`** | Kelly + hard depth cap and per-market exposure cap. Capacity realism. |
| **`fade`** | Anchor, **sell-side only** — the house finding that our sells work and our buys don't. |
| **`patient`** | Fade, but entry **delayed 24h** on frozen inputs — the wing-drift finding says sell fills improve with delay. |
| **`sniper`** | Anchor, top-decile edge only, double size, few trades. Tests concentration vs breadth. |
| **`harvest`** | Fade, but **exit when 60% of the edge has closed** instead of holding. Frees capital; tests whether patience is actually paid for. |

Each is one TOML in `policies/`, versioned; changing a policy means a new version file,
never an edit — results must stay attributable.

## 6. Outputs

Per (signal set × policy) → `results/<set>/<policy>.json`:
trades, staked, net PnL, **cents/trade ± se and t-stat**, hit rate, mean hold days,
**return on locked capital + annualized**, capital efficiency, max drawdown, longest
losing streak, unfundable count, synthetic-fill share, per-variant and per-asset
attribution, and the **equity curve** (`[{t, equity, deployed}]`) for the dashboard.

Plus `results/summary.csv` — the policy × signal-set matrix, which is the artefact I
actually want to look at every week.

## 7. Honesty rules

- **No policy is "best" on `orakel-live` yet.** 21 scored predictions cannot rank eight
  policies; the matrix must print sample sizes next to every number and the engine refuses
  to declare a winner below `n = 30` per policy.
- Historical signal sets carry **one regime**; every result is reported with its date span.
- Nothing here is a trading system. It is a measurement instrument that tells us, before
  any real money exists, which of our beliefs would have survived contact with a spread.
