# engine — the execution simulator

Implements [DESIGN.md](../DESIGN.md). Read §3 (capital lockup), §4 (cost model)
and §7 (honesty) first; this file only documents the choices the design left to
the implementation.

```sh
cargo build --release
cargo test                                    # 37 tests; the accounting ones are hand-computed

target/release/engine run --set all --policy all
target/release/engine run --set ladder-rv-hist --policy fade
target/release/engine report                  # rebuild summary.csv + SUMMARY.md from results/
```

## Shape

```
src/lib.rs        exports + timestamp/format helpers
src/signal.rs     canonical signal schema + CSV reader        (pure)
src/policy.rs     policy TOML + engine defaults               (pure)
src/sim.rs        simulate(signals, policy) -> SimResult      (pure)
src/metrics.rs    metrics, attribution, equity curve, notes   (pure)
src/main.rs       the CLI: the only file that touches disk
src/bin/*.rs      the signal-set adapters (R2, gzip, CSV)
```

The library has **no filesystem, network or clock access** — `chrono` is built
without its `clock` feature on purpose — so it compiles to wasm for the
dashboard:

```sh
cargo build --lib --release --target wasm32-unknown-unknown
```

`simulate` takes `(&[Signal], &Policy)` and returns a `SimResult`. Nothing else
is needed to run a policy.

## Engine defaults (not in the policy files)

| default | value | why |
|---|---|---|
| `sizing.bankroll_usd` | 1000 | flat policies do not declare one; capital efficiency needs a denominator |
| `sizing.min_stake_usd` | 1.00 | Polymarket's minimum ticket. Depth that cannot fund it ⇒ **unfundable** |
| `entry.delay_tolerance_hours` | 12 | how far past `t + delay_hours` an observation may sit and still count as "delay_hours later" |
| depth band | 5c | DESIGN.md §4.2; also the scale of the slippage penalty |

Changing one of these changes every result, so they live in one place
(`policy.rs`) and every result JSON echoes the bankroll and spread it used.

## How the mechanics are implemented

**Side and edge.** A buy lifts the ask, a sell hits the bid. So the side is
determined by the *executable* price, not the midpoint: `p > ask` ⇒ buy with
edge `p − ask`; `p < bid` ⇒ sell with edge `bid − p`; a probability inside the
spread is not tradeable at any size and is counted as `no_executable_edge`. With
no real book, `bid = mid − assumed_spread/2` and `ask = mid + assumed_spread/2`,
and the trade is marked `synthetic_fill`.

**Sizing.** `stake` is the capital *committed*, which by DESIGN.md §3 is
`ask × shares` for a buy and `(1 − bid) × shares` for a sell. Kelly on the
executable price: buy `f = (p − a)/(1 − a)`, sell `f = (b − p)/b` (selling YES
at `b` is buying NO at `1 − b` with win probability `1 − p`). Clamps, in order:
`max_bankroll_fraction × bankroll`, then the room left under
`max_per_market_usd` for that market's *currently open* exposure, then
`max_book_fraction × depth`. Below `min_stake_usd` after all that, the signal is
rejected with the reason that did it.

**Slippage.** With real depth, `slip = band × (stake / depth)` where
`band = min(5c, room to the price boundary)` — a 1c bid cannot slip 5c, it can
only slip to zero. Consuming the whole visible depth walks the price the whole
band. We cannot walk individual levels, because the canonical schema carries
aggregate depth rather than the ladder; that is a future-signal-set ask.

**Fees.** `costs.fee_model = "polymarket-taker"` charges the venue's published
taker fee `shares × rate × p × (1 − p)` USDC **per fill**, at that fill's price,
with `rate` looked up as `asset → costs.asset_category → costs.fee_rate`.
Rounding follows the venue: 5 decimals, and anything under 0.00001 USDC is not
charged. Because `p(1−p)` is symmetric, selling YES at `p` and buying NO at
`1 − p` — the same trade under DESIGN.md §3 — cost the same, so the fee is
computed on the YES price whatever the side.

Entry is always one fill. An exit is a second fill **only** for `take-profit`:
settling at resolution is a redemption, not a match, and the venue charges
nothing for it. So a held position pays the fee once and a round trip pays it
twice — which is why `harvest`, alone among the policies, loses about twice as
much to fees as its hold-to-resolution twin `fade` (1.18c/share vs 0.78c).

Three deliberate non-choices: the fee does **not** shrink the stake (sizing is
identical across versions, so a v1→v2 delta is the fee and nothing else), it does
**not** enter `capital_locked` (that is collateral, not expense), and there is no
maker path (every fill here crosses the spread). `fee_model = "none"` is the
default and makes every affected result carry a note saying fees were not
charged — a fee-free result that looks costed is the failure mode this field
exists to prevent.

**Exits.** `hold-to-resolution` settles at 1 or 0 with no spread paid.
`take-profit` computes `target = fill ∓ close_fraction × |fill − p|` and exits at
the first later observation of the same token whose *executable* exit price
reaches it — buying back a short lifts the ask, selling a long hits the bid, and
the exit pays slippage too. If the target is never reached, it holds.

**Delayed entry.** `delay_hours > 0` enters at the first observation of the same
token in `[t + delay, t + delay + tolerance]`, with `p_model` frozen at `t` and
the gates re-checked at the delayed price — the same convention ladder-rv's own
gate-2 simulation uses. A signal with no such observation is **excluded and
counted**, together with how many of the excluded rows sit on markets that
resolved in the token's favour, because that attrition is not random.

**Bankroll.** Sizing uses a static bankroll and the engine does **not** refuse a
trade for lack of free capital. It reports `capital_efficiency` (time-weighted
mean deployment / bankroll) and `max_capital_efficiency` instead, and flags any
policy whose peak deployment exceeded the stated bankroll. Enforcing the
constraint would have made the comparison between policies a race about who ran
out of money first; reporting it keeps the comparison about edge and still says
plainly which policies the stated bankroll could not fund.

**Equity** is bankroll plus *realized* PnL. Open positions are not marked to
market, because the signal sets carry no price for every day of every hold.

## The three interpretive choices worth arguing with

1. **`require_book = true` does not reject synthetic fills.** DESIGN.md §4.1
   requires that a midpoint-only historical signal be priced with
   `assumed_spread` and marked, and the task's gate list does not include
   `require_book`. Under the strict reading — reject any row without a real book —
   seven of the eight policies would take **zero** trades on `ladder-rv-hist`,
   the only set large enough to separate them, and the exercise would be empty.
   So the engine prices such rows honestly, marks them, and emits a note on every
   affected result saying exactly this. It is the single most consequential
   choice in the engine, and it is the first thing a real book would settle.

2. **`respect_venue_epsilon` is reported, never silently passed.** No signal set
   carries the barrier-to-running-extreme distance (see
   `../signals/README.md` §5 for why the archived whole-window extreme would be
   hindsight), so the screen cannot run. Each result reports how many of the
   sells it took were unscreened.

3. **`edge_percentile` ranks *claimed* edge** (`|p_model − p_market|`), not
   executable edge, so "the top decile of the signal set" is a property of the
   set rather than of the policy's spread assumption. `sniper`'s character is
   "our biggest claimed edges", and this is what that means.

## A note on DESIGN.md §3's worked examples

The formula §3 states — `pnl / (capital_locked × days_held) × 365` — is what the
engine implements, and it is what `annualized_return_on_locked_capital` means.
The two illustrative numbers in that paragraph ("~53%", "~13%") come out of the
formula as ~537% and ~125%: a consistent factor-of-ten in the prose. The
*argument* is unaffected — the ratio between the two examples is the same either
way, and that ratio is the point.
