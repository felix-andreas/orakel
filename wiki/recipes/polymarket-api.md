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
- Single market by slug: `/markets?slug=<slug>`. Resolution check: `closed == true` and
  `outcomePrices` collapsed to `[1, 0]` / `[0, 1]`.
- `closed=true` queries are the backtest goldmine: resolved markets + their metadata.
- **Series discovery** (find all instances of a recurring family, open AND resolved):
  `/public-search?q=<text>&limit_per_type=20` → `{events: [...], ...}` with `closed`
  flags. Much better than paging `/events` when you know the family's title words
  (e.g. `q=temperature+increase` returns 20+ monthly instances back to 2024).

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

## Identity conventions

`condition_id` (stable market id) + `market_slug` (human) identify a market; each outcome
is a token (`clobTokenId`) — predictions are probabilities on one outcome token.
