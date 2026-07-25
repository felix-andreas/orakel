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

## Identity conventions

`condition_id` (stable market id) + `market_slug` (human) identify a market; each outcome
is a token (`clobTokenId`) — predictions are probabilities on one outcome token.
