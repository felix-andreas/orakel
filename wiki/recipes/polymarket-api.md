# Polymarket API recipes

All read-only, no keys. Three services: **Gamma** (discovery/metadata), **CLOB** (books,
midpoints, price history), **Data API** (trades tape). Snapshot everything you rely on
(via `tools/r2data/`).

## Discovery — Gamma

```rust
// cargo add reqwest --features json,blocking  |  serde_json
let markets: serde_json::Value = reqwest::blocking::Client::new()
    .get("https://gamma-api.polymarket.com/markets")
    .query(&[
        ("closed", "false"), ("active", "true"), ("archived", "false"),
        ("order", "volumeNum"), ("ascending", "false"), ("limit", "200"),
        // filter by tag: ("tag_id", "<id>") — list tags at /tags
    ])
    .send()?.json()?;
```

- `outcomes`, `outcomePrices`, `clobTokenIds` come back as **JSON-encoded strings** —
  parse each again with `serde_json::from_str`.
- `volume`/`liquidity` may be strings; use numeric `volumeNum`/`liquidityNum`.
- `outcomePrices` = book midpoints, NOT trade prices.
- Single market by slug: `/markets?slug=<slug>`. **Gotcha: this returns `[]` once the
  market closes** — add `&closed=true` to fetch resolved markets by slug (found while
  scoring the first resolutions, 2026-07-25). Resolution check: `closed == true` and
  `outcomePrices` collapsed to `[1, 0]` / `[0, 1]`; the winning outcome is the one whose
  price is 1.
- Prefer `curl` over Python `urllib` from agent sessions: the environment proxy 403s
  urllib's default requests while curl works.
- `closed=true` queries are the backtest goldmine: resolved markets + their metadata.
- **Series discovery** (find all instances of a recurring family, open AND resolved):
  `/public-search?q=<text>&limit_per_type=20` → `{events: [...], ...}` with `closed`
  flags. Much better than paging `/events` when you know the family's title words
  (e.g. `q=temperature+increase` returns 20+ monthly instances back to 2024).
- **Deep history: offset paging hard-caps at `offset=2000`** (`{"error":"offset too large,
  use /events/keyset for deeper pagination"}` — and it returns that as a *200 with a JSON
  object*, so a `list`-assuming parser silently drops pages). `/events/keyset` ignores a
  `cursor=` param; the documented name is **`after_cursor`**. The reliable pattern for a
  whole-family harvest is **date-windowed offset paging**: weekly
  `end_date_min` / `end_date_max` windows × `offset` 0,100,… until a short page
  (16,959 resolved esports events over 7 months, 2026-07-25).
- **Sports events are strongly typed** — don't parse titles. Each market carries
  `sportsMarketType` (`moneyline`, `child_moneyline`, `totals`, `map_handicap`,
  `round_handicap_game_N`, `kill_over_under_game`, …), plus `gameStartTime` (the exact
  match start, essential for defining a pre-match checkpoint), `gameId`, and
  `resolutionSource`. Verify semantics in `description` anyway — the leg's *meaning*
  (which side is `outcomes[0]`) is only stated there.

## Prices — CLOB (keyed by outcome token id, not market)

```
GET https://clob.polymarket.com/midpoint?token_id=<clobTokenId>
GET https://clob.polymarket.com/book?token_id=<clobTokenId>      # best bid/ask = LAST elements
GET https://clob.polymarket.com/prices-history?market=<clobTokenId>&interval=max&fidelity=60
```

- `prices-history` returns `{"history": [{"t": unix_s, "p": price}, ...]}`.
- **Gotchas:** `interval=max` silently caps at ~30d — for the full series use
  `startTs=<epoch>&fidelity=60` *alone* (adding `endTs` → 400). Sometimes returns empty
  `history`: retry with explicit `startTs`, drop `fidelity`, or rebuild the series from
  the Data API trades. These gaps are why orakel snapshots prices itself.

## Trades — Data API

```
GET https://data-api.polymarket.com/trades?market=<condition_id>&limit=500&offset=N
```

Paginate for the full tape. Polymarket headline `volume` ≈ 2× taker notional from this
endpoint (it counts shares, both sides). Sub-1-share fills may be omitted.

## Taker fees — put them in every PnL line

Polymarket charges a **taker-only** fee on most categories (makers pay nothing and share a
rebate). Documented formula, confirmed against each market's `feeSchedule` field:

```
fee_usdc = shares × feeRate × p × (1 − p)
```

`feeRate` by category: **crypto 0.07 · sports 0.05 · economics/culture/weather/other 0.05 ·
finance/politics/tech/mentions 0.04 · geopolitics 0 (fee-free)**. The dollar fee is
symmetric about 0.50 and *peaks* there: sports costs **1.25c/share at p=0.50**, 1.20c at
0.40, 0.45c at 0.10. Read it per market from `feeSchedule`
(`{rate, exponent, takerOnly, rebateRate}`) + `feesEnabled` + `feeType`; `/fee-rate?token_id=`
returns only the `base_fee` in bps.

Consequences: (a) mid-priced legs are the *most* fee-expensive per share, so a 3–50c
"fundable band" trade pays ~1.2c before spread; (b) resting limit orders (maker) avoid the
fee entirely — worth modelling separately from a taker fill; (c) geopolitics markets are
the only genuinely fee-free ones.

*Added 2026-07-25 by the execution agent while implementing this in the simulator — three
things the entry did not say that change PnL:*

- **(d) Charged per fill: entry AND exit, but never at resolution.** Fees apply "at match
  time", so a position closed in the market pays **twice** while one held to settlement
  pays **once** — redemption is not a match. This is the single most consequential fact
  for execution modelling: it is a standing tax on early-exit and rebalancing policies,
  and it does not appear anywhere in a hold-to-resolution backtest.
- **(e) Commodities and equities are `finance` (0.04), not `economics` (0.05).** Read off
  `feeType`: gold/silver/WTI and SPY/NVDA barrier markets all return
  `finance_prices_fees` @ 0.04; BTC/ETH return `crypto_fees_v2` @ 0.07. `economics_fees`
  is Fed/CPI-style macro, not commodity prices. Do not infer the category from the
  underlying — read `feeSchedule` per market.
- **(f) The sports rate is a moving target: it was 0.03 until 2026-07-10**, when it rose
  to 0.05. Any backtest spanning that date needs both. Rates have changed twice in 2026
  and are off-chain operator policy, not a protocol invariant — re-read them, don't cache
  them.

Verified against `docs.polymarket.com/trading/fees` (which serves a 403 to plain fetchers —
use curl with a browser UA), the per-market `feeSchedule`, and by fitting ~2,300 real
executed fills from the Data API: the fee residual matches `p × (1 − p)` and rules out the
`min(p, 1−p)` form; fills with zero implied fee are the maker side.

## Identity conventions

`condition_id` (stable market id) + `market_slug` (human) identify a market; each outcome
is a token (`clobTokenId`) — predictions are probabilities on one outcome token.

## `closed` is a FILTER, not an include-flag (Gamma `/markets`)

Verified 2026-07-26 on both query forms — this bit us twice in one run, in both directions:

| query | open market | closed market |
|---|---|---|
| `?condition_ids=<cid>` | returns the row | **`[]`** |
| `?condition_ids=<cid>&closed=true` | **`[]`** | returns the row |
| `?slug=<slug>` | returns the row | **`[]`** |
| `?slug=<slug>&closed=true` | **`[]`** | returns the row |

So `&closed=true` does not mean "include closed too", it means "closed only". Consequences,
both of which actually happened:

- **A resolution sweep without the flag can never find anything.** A CEO check for "which of
  our open markets have resolved?" ran without it and returned a confident `0 newly closed`
  that was structurally incapable of returning anything else.
- **A row-verification sweep with the flag can never find anything either.** Validating 13
  freshly proposed rows *with* `&closed=true` reported all 13 unresolvable — they were open.

**Query both, or query on the state you expect and treat an empty result as "wrong filter"
before treating it as "no such market".**

## Neg-risk boards: placeholder legs exist only in `/events`

A neg-risk group lists unnamed placeholder outcomes — `will-company-a-…` through
`will-company-m-…`, plus `will-any-other-company-…`. They are open markets with
`enableOrderBook: true` and **no book at all**, sitting at `volumeNum: 0` and
`liquidityNum: 0`.

Two things follow, both learned the hard way on 2026-07-26:

1. **They are invisible to `/markets?condition_ids=<cid>`** — that endpoint returns `[]` for
   them even though they are open and appear in the parent event. Any filter written against
   `/markets` silently classifies them as "unknown" and keeps them. Read `volumeNum` /
   `liquidityNum` from `/events?slug=<event>` instead.
2. **Snapshotting them poisons your error channel.** Feeding them to the CLOB `POST /books`
   yields `no book returned for token` for every one. On our first mirror of the arena
   cohort that was 166 of 430 tokens — 39% of the fetch failing by construction, which makes
   a genuine failure invisible.

Filter on `volumeNum == 0 && liquidityNum == 0` at build time rather than blocklisting slug
patterns, and rebuild the list each run, so a leg that later activates returns by itself.
