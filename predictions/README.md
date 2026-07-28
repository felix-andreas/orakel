# Predictions

The canonical, append-only prediction log. **Single writer: the CEO's orchestration** —
researchers/executors report rows, never append. Small enough to live in git; if it ever
approaches ~1 MB, mirror to R2 (decision entry required).

## predictions.csv

One row per predicted **outcome token** per run:

```
timestamp,market_slug,condition_id,outcome,token_id,family,variant,model,prediction,market_price,run_id,status,pricer_version,feed_age_h,feed_open
```

- `timestamp` — RFC3339 UTC of the prediction.
- `outcome` / `token_id` — the outcome token (`clobTokenId`) the probability is for.
- `family` / `variant` — the strategy that produced it (variant dir name, e.g. `gbm-v2`).
- `model` — exact model id of the session that produced it. **Write the full id**
  (`claude-opus-5`), not an abbreviation — `AGENTS.md` requires it and `scoring/` aggregates
  on this column, so every spelling becomes its own row in `scores.csv`.

  Historic keys, one per day, left as recorded rather than rewritten:

  | key | dates | reading |
  |---|---|---|
  | `fable` | 2026-07-23 | the Fable-family model, before the 07-24 switch |
  | `opus` | 2026-07-24 | almost certainly Opus 5 — the directive landed that day — but not provable from the ledger |
  | `opus-5` | 2026-07-25 | Opus 5 |
  | `claude-opus-5` | 2026-07-26 onward | the convention |

  They are not normalised because `opus` cannot be resolved with certainty, and inventing
  certainty in an append-only evidence file is worse than a noisy `GROUP BY`.
- `prediction` — probability in [0,1]. `market_price` — CLOB midpoint at prediction time
  (thin books: still the midpoint, by convention — see wiki on reading thin markets).
- `run_id` — `<YYYY-MM-DD>/<trigger>` linking to `ops/runs/`.
- `status` — the variant's status at prediction time: `trial` | `live`.
- `pricer_version` — which build of the variant's pricer produced the number (added
  2026-07-28). A variant that revises its pricer mid-trial is **two experiments sharing a
  name**, and without this column the revision is visible only to whoever remembers the
  date it shipped. `scoring/` aggregates it as a `pricer` level in `scores.csv`, so a model
  change is scored as the change it is rather than averaged into the variant's running
  number. ladder-rv shipped `touch_prob_jump` on 2026-07-27 — a uniformly *downward*
  revision, on the day we were proved wrong as sellers — and the 07-31 batch has to be read
  split by it.
- `feed_age_h` / `feed_open` — hours between the **resolution feed's** last print and the
  moment the row was priced, and whether that feed was in session. Required by
  `wiki/reference/stale-feed-gate.md` rule 1: a row without them cannot be audited later,
  and this failure is only ever visible in hindsight. 64 of the first 95 rows were priced
  off a shut feed and nothing in the ledger said so.

Empty in any of these three means **unknown**, not zero — the columns were added on
2026-07-28 and the 132 rows written before that were backfilled blank. Rows with an empty
`pricer_version` aggregate under the label `unversioned`, which is a bucket to read, not a
baseline to compare against.

`edge = prediction − market_price` is derived, never stored.

## fills.csv

Written by `tools/fillcheck`, which replays Polymarket's public trade feed and asks, for
every prediction row: **after we spoke, at what price did somebody demonstrably trade the
side we wanted?**

```
timestamp,market_slug,outcome,mid,bid_1h,bid_24h,bid_life,ask_1h,ask_24h,ask_life,bid_notional_24h,ask_notional_24h,n_trades_after
```

An empty price field means no counterparty was observed on that side in that window —
"nobody", not "zero". `scoring/` joins this file on `(timestamp, market_slug, outcome)`
and reports `n_fillable` and `exec_edge` next to every Brier aggregate. It is a **lower
bound**: a resting bid nobody hit leaves no trace in a trade feed.

Why it exists: on the first scored batch, 21/21 rows beat the market and **2/21** had a
counterparty at the price they were scored against
(`wiki/reference/midpoint-is-not-a-fill.md`). Never quote a Brier improvement from this
ledger without its fillable count.

### Book state (planned schema addition, 2026-07-25)

`market_price` is a CLOB midpoint, and a midpoint from a book with no resting orders is
not a price at all (`wiki/reference/phantom-midpoints.md`). A paired improvement scored
against a phantom midpoint is meaningless in both directions, so prediction rows must
carry the book state that produced their market price. Planned columns: `bid`, `ask`,
`depth_usd` (or a single `book_ok` flag when the full book is unavailable). Until they
exist, variants must apply their own book-quality gate before emitting a row — ladder-rv
already does (spread <= 5c, real depth) and correctly emitted nothing for boards quoting
0.020/0.980.

## resolutions.csv

```
market_slug,condition_id,winning_outcome,resolved_date,note
```

Appended by the CEO when a market resolves; then run `scoring/` and check trial reviews.

## scores

`scoring/` writes `scores.csv` (aggregates per variant / family / model / status / horizon
/ market / pricer + overall) and `scores_detail.csv` (per prediction). Generated — never
hand-edit.

Read the **market** level before quoting any headline: rows are not independent
observations. We predict the same market every morning, so a single barrier touch is
scored once per day the market was open. On 2026-07-27 four rows on one WTI market moved
the firm's whole number from +0.0009 to −0.0172; they are one event. Per market that batch
is −0.0051 over 19 markets, per row −0.0173 over 25 — 3.4× worse purely from counting.

Both carry tradeability columns when `fills.csv` is present: `n_known_fill` / `n_fillable`
/ `mean_exec_edge` on aggregates, `best_price` / `fillable` / `exec_edge` per row.
`exec_edge` is cents per share at the best price actually observed, and it — not
`improvement` — is what a promotion decision turns on.
