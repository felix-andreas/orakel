# orakel snapshot worker

Cron-triggered Cloudflare Worker (Rust, workers-rs) that builds the firm's own market
history: every hour it snapshots Polymarket prices + order books for all watched markets
into R2. This is the dataset that makes execution backtests honest — real spreads, real
depth, no reliance on Polymarket's gappy history API.

## What it does (cron `7 * * * *`, UTC)

1. Reads `config/watchlist.json` from the `orakel` bucket (R2 binding, no credentials):
   ```json
   {"updated": "2026-07-23T09:00:00Z",
    "markets": [{"condition_id": "0x…", "market_slug": "…", "token_ids": ["…", "…"]}]}
   ```
   Maintained by the **CEO**, mirrored from the union of active
   `strategies/*/*/applications/`. Absent/empty → the run logs "watchlist empty" and
   exits (normal until strategies exist).
2. Fetches order books + midpoints for all tokens via the CLOB **batch** endpoints
   (`POST /books`, `POST /midpoints`) — a handful of subrequests regardless of watchlist
   size (Workers free plan allows 50/invocation).
3. Writes one gzipped JSON to `snapshots/books/<YYYY-MM-DD>/<HH>.json.gz` — key derived
   from the scheduled hour, so reruns overwrite (idempotent) and readers can compute keys
   from time alone (no index, no git manifests — see ARCHITECTURE.md §6).

## Snapshot schema

```json
{"ts": "2026-07-23T09:07:00Z",
 "markets": [{"condition_id": "0x…", "market_slug": "…",
   "tokens": [{"token_id": "…", "midpoint": 0.55,
     "bids": [[0.54, 120.5], …], "asks": [[0.56, 80.0], …]}]}]}
```

- `bids`/`asks`: top 10 levels, **best first** (the CLOB API returns best LAST — this
  worker normalizes), `[price, size]` as numbers.
- `midpoint`: number or `null`. A token whose book fetch failed carries an `"error"`
  string instead of aborting the run.

## Deploy / operate

```sh
cd workers/snapshot
export WASM_BINDGEN_BIN=wasm-bindgen   # proxy-restricted sessions (see dashboard/README.md)
CLOUDFLARE_API_TOKEN=… npx wrangler deploy
```

`GET /` on the deployed worker returns a one-line status text (used by health checks).
Logs: `npx wrangler tail orakel-snapshot`. Local test: `npx wrangler dev --test-scheduled`
then `curl "http://localhost:8787/__scheduled?cron=7+*+*+*+*"` (local R2 sim; put a
watchlist with `npx wrangler r2 object put --local`).
