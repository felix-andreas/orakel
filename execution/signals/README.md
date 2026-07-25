# Signal sets

A **signal set** is a named, frozen collection of signals with known outcomes
(DESIGN.md §2). Policies are evaluated *policy × set*, because "which execution
strategy is best" is meaningless without saying *on whose signals*.

Each set is one CSV plus a `<set>.source.json` sidecar recording exactly what it
was built from (input file sha256s, the R2 manifest, the snapshot keys used).
Rebuilding a set from the sidecar's sources must reproduce the CSV byte for byte.

## Canonical schema

```
signal_set,t,market_slug,condition_id,outcome,token_id,family,variant,model,p_model,p_market,bid,ask,depth_bid_usd,depth_ask_usd,resolved_outcome,resolved_date,synthetic_book
```

| column | meaning |
|---|---|
| `signal_set` | the set's name, repeated on every row |
| `t` | signal time, RFC3339 UTC |
| `market_slug`, `condition_id` | the market (`condition_id` is the exposure key for per-market caps) |
| `outcome`, `token_id` | the outcome token the probability is about |
| `family`, `variant` | the strategy that produced it (`barrier-touch`, `ladder-rv`) |
| `model` | the exact model id that produced `p_model` |
| `p_model` | our probability, in [0,1] |
| `p_market` | the market midpoint at `t` (recorded by convention even in thin books) |
| `bid`, `ask` | real touch prices, **may be empty** |
| `depth_bid_usd`, `depth_ask_usd` | USD notional resting within 5c of each touch, **may be empty** |
| `resolved_outcome` | the market's winning outcome label |
| `resolved_date` | when the position ends: `YYYY-MM-DD` (read as **12:00 UTC**, `scoring/`'s convention) or full RFC3339 |
| `synthetic_book` | `1` when the row carries no real book |

Empty `bid`/`ask` ⇒ the engine applies the policy's `assumed_spread`
symmetrically around `p_market` and marks the trade `synthetic_fill`
(DESIGN.md §4.1). Empty depths ⇒ the depth gate and the depth cap cannot be
evaluated; the engine counts those trades in `depth_unknown` and reports them.

**Optional trailing column `asset`** — an attribution key coarser than the
market (`wti`, `spy`, `gold`, …). Absent ⇒ the engine attributes by
`market_slug`. Both sets below carry it; DESIGN.md §6 requires per-asset
attribution and the canonical columns alone cannot provide it.

Rows are read by header name, so extra columns are ignored and column order is
not load-bearing. `#` starts a comment line.

## `orakel-live`

Our own `predictions/predictions.csv` ⋈ `predictions/resolutions.csv`, restricted
to predictions whose market has resolved. The real thing — the only set with no
hindsight anywhere in the signals.

```sh
execution/engine/target/release/build-orakel-live \
  --predictions predictions \
  --out execution/signals/orakel-live.csv \
  --cache /tmp/snapshot-cache          # add --offline to rebuild from the cache
```

Book data comes from the firm's own hourly R2 snapshots
(`snapshots/books/<YYYY-MM-DD>/<HH>.json.gz`, schema in
`workers/snapshot/README.md`). For each prediction the adapter takes the
**latest snapshot whose `ts` is at or before the prediction time**, at most 24h
stale — never a later one, which would be lookahead.

**What that costs us today: 3 of 21 rows have a real book.** The snapshot
watchlist grew from 18 to 40 markets at 2026-07-24 02:07Z — *26 minutes after*
the 01:41Z prediction run that produced 18 of the 21 resolved signals. Those 18
rows were priced by a market we were not yet watching. This is an operational
finding, not a data limitation: adding a market to the watchlist **before** the
run that predicts it would have made this set 21/21 real books.

## `ladder-rv-hist`

The `barrier-touch/ladder-rv` variant's resolved-leg checkpoints: one row per
leg per daily 12:00Z checkpoint inside its resolution window, 5,927 rows over
633 resolved legs, 46 boards, 7 assets, 2026-04-01 → 2026-07-24.

Source: the variant's day-3 R2 freeze
`strategies/barrier-touch/ladder-rv/data/backtest-metals-2026-07-25.tar.gz.r2.json`
(sha256 `2ecbbd6c…`).

```sh
tools/r2data/target/release/r2data pull \
  strategies/barrier-touch/ladder-rv/data/backtest-metals-2026-07-25.tar.gz.r2.json \
  --out /tmp/lrv.tar.gz
mkdir -p /tmp/lrv && tar xzf /tmp/lrv.tar.gz -C /tmp/lrv
execution/engine/target/release/build-ladder-rv-hist \
  --data /tmp/lrv \
  --manifest strategies/barrier-touch/ladder-rv/data/backtest-metals-2026-07-25.tar.gz.r2.json \
  --out execution/signals/ladder-rv-hist.csv
```

### Interpretive decisions, and why

Reading another variant's backtest output into an execution simulator requires
judgement calls. All of them are listed here; none of them are silent.

1. **Which archive.** DESIGN.md points at the *day-1* freeze
   (`backtest-2026-07-23`, 3,100 checkpoints / 229 legs). The **day-3 metals
   freeze is a strict superset** — every day-1 `(market_slug, t)` checkpoint is
   present, plus gold, silver and the WTI/metals weekly family discovered on
   2026-07-25, and it was produced *after* the weekly-window fix (`board_period`
   now follows the asset's session clock). Verified containment: 3,100 of 3,100
   day-1 rows present, 2,827 additional. Using the superset nearly doubles the
   sample and removes a known window bug.

2. **`p_model` = `q_rv`, not `q_iv`.** `STRATEGY.md` §2 makes realized vol the
   primary model and the IV anchor the secondary. Both columns exist in the
   archive; the set carries the primary.

3. **No book, at all.** The archive stores CLOB *price history*
   (`{t, p}` at 60-minute fidelity), not order books — Polymarket has no
   historical book API, which is precisely why the firm's snapshot worker
   exists. Every row is therefore `synthetic_book = 1`, and every result on this
   set is priced at `mid ± assumed_spread/2`. Per DESIGN.md §4 those results are
   flagged, not celebrated.

4. **`resolved_date` = when the position actually ends** — `gate0.first_touch`
   for a leg that touched (a one-touch market resolves on the touch, and the
   seller's loss crystallizes there), the leg's window end `we` otherwise. Gate 0
   reproduced all 633 of these legs from candle data, so `first_touch` is a
   record, not an estimate. Using `we` for touched legs instead would have
   overstated hold times and understated annualized returns.

5. **The venue-epsilon field is genuinely absent.**
   `wiki/reference/venue-resolution-epsilon.md` screens sells whose barrier sits
   within 0.2% of the leg's **running** extreme — the extreme *so far*, at
   decision time. `gate0.csv` carries `extreme` and `margin`, but those are
   whole-window values: using them at a checkpoint would be hindsight, and
   hindsight that removes exactly the legs about to touch. So the field is
   omitted and every policy with `respect_venue_epsilon = true` reports its
   unscreened sells instead of pretending. This is the single largest gap in the
   set.

6. **Checkpoints inside the variant's own σ-proximity exclusion are kept**
   (203 rows, 3.4%, `zone_excl = 1` in the archive). That flag is part of the
   variant's *signal generation*, not of an execution policy, and it is
   computed without hindsight — the engine simply cannot see it under the
   canonical schema. It is the natural stand-in for the epsilon screen in a
   future version of this set.

7. **76 of the 633 legs are WTI legs priced off the `USOILSPOT` proxy**
   (`gate0.proxy = 1`) because expired per-contract Pyth feeds are deleted. The
   day-1 backtest measured that proxy's basis (|Δhigh| p50 $0.21, p95 $0.42) and
   flagged legs with `|barrier − extreme| ≲ $0.5` as unreliable. They are kept —
   they are part of the variant's own evidence base — but a resolution that
   turns on less than half a dollar of WTI is not fully trustworthy.

8. **Each checkpoint is its own candidate trade**, as in the variant's gate-2
   simulation. Checkpoints on one leg share a single outcome, so per-trade
   standard errors on this set are optimistic; the engine says so in every
   result's `notes`.

## What a future signal set should carry

In rough order of how much it would change the answers:

- **Real books.** Bid, ask and depth from our own hourly snapshots, for every
  market we predict, *from before the prediction run*. Everything on
  `ladder-rv-hist` and 86% of `orakel-live` is currently priced off a
  midpoint plus an assumption.
- **`barrier_extreme_frac` at `t`** — |barrier − running extreme| / barrier,
  computed with data available at `t`. Without it `respect_venue_epsilon` cannot
  run on any set, and it is a screen the firm has already proved it needs.
- **A denser price path.** Delayed entry and take-profit exits can only use
  prices that exist as rows. Daily checkpoints make "+24h" the only delay the
  set can express, and they make a leg that touches simply *stop* — which biases
  every delayed policy away from its own losses (quantified in the results).
- **Full book ladders, not aggregate depth**, so the engine can walk levels
  instead of applying a linear slippage penalty.
- **A second variant on the same markets**, which is the only thing that would
  make the `[combine]` step do any work.
