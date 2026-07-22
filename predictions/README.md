# Predictions

The canonical, append-only prediction log. **Single writer: the CEO's orchestration** —
researchers/executors report rows, never append. Small enough to live in git; if it ever
approaches ~1 MB, mirror to R2 (decision entry required).

## predictions.csv

One row per predicted **outcome token** per run:

```
timestamp,market_slug,condition_id,outcome,token_id,family,variant,model,prediction,market_price,run_id,status
```

- `timestamp` — RFC3339 UTC of the prediction.
- `outcome` / `token_id` — the outcome token (`clobTokenId`) the probability is for.
- `family` / `variant` — the strategy that produced it (variant dir name, e.g. `gbm-v2`).
- `model` — exact model id of the session that produced it.
- `prediction` — probability in [0,1]. `market_price` — CLOB midpoint at prediction time
  (thin books: still the midpoint, by convention — see wiki on reading thin markets).
- `run_id` — `<YYYY-MM-DD>/<trigger>` linking to `ops/runs/`.
- `status` — the variant's status at prediction time: `trial` | `live`.

`edge = prediction − market_price` is derived, never stored.

## resolutions.csv

```
market_slug,condition_id,winning_outcome,resolved_date,note
```

Appended by the CEO when a market resolves; then run `scoring/` and check trial reviews.

## scores

`scoring/` writes `scores.csv` (aggregates per variant / family / model / status + market
baseline) and `scores_detail.csv` (per prediction). Generated — never hand-edit.
