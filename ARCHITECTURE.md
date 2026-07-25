# Architecture

What orakel is made of and how the pieces interact. The *current operating state* (which
slots are active, cadences, live strategies) is **not** here — it lives in
[`ops/state.toml`](ops/state.toml). This file describes the invariant structure.

## 1. The strategy model: family → variant → application

The unit of research, trial, scoring, and execution is the **variant**.

- **Family** (`strategies/<family>/`) — an idea in spirit ("fade salience-driven momentum in
  thin markets", "first-passage models for touch markets"). A family groups variants, holds
  cross-variant lessons in `FAMILY.md`, and can start as a family of one. Families are never
  executed directly.
- **Variant** (`strategies/<family>/<variant>/`) — one concrete implementation with its own
  code, memory, and track record. Variants in a family are similar *in spirit only*; their
  code may be completely different.
- **Application** (`applications/<market>.toml`) — variant × market: the per-market
  parameterization. Predictions are logged per application, so track records aggregate
  application → variant → family.

**Membership rule:** a market joins an existing variant when onboarding needs only a params
file plus at most small, local code accommodations that don't change the method or degrade
existing applications. When generalizing would cost edge, **split the variant instead** —
specificity is a feature. A variant serving a single market is legal.

**Versioning:** a new version is a new variant directory with a name postfix (`gbm-v2`) and
`supersedes = "gbm-v1"` in its `strategy.toml`. Versions may run in parallel; if they diverge
permanently, rename one to something meaningful and drop the `supersedes` link. Lineage is
just that one field — no other version machinery.

**Labels:** each variant's `strategy.toml` carries reserved fields with semantics
(`status`, `supersedes`) plus free-form `labels` (e.g. `class:mean-reversion`,
`data:weather`). Directories are the canonical home; labels are how scoring and the
dashboard slice track records. Details: [`strategies/README.md`](strategies/README.md).

## 2. Lifecycle

```
idea (ideas/ backlog) → trial (research slot) → live | discarded
                             ↑ CEO assigns          ↑ CEO decides
```

- The **market researcher** scans markets daily and produces one distinct strategy idea per
  day with at least one example market → [`ideas/`](ideas/) backlog.
- The **CEO** fills free **research slots** (start: 5, tunable in `ops/state.toml`) from the
  backlog. One slot = one variant in **trial**.
- A trial runs **at least 10 days** and is judged on scored evidence — guideline: ≥15 scored
  predictions across ≥3 markets beating the market baseline, plus backtests on already-resolved
  markets. The CEO decides promotion/discard/extension and logs the reason in
  `ops/decisions.md`. Trials should prefer fast-resolving markets so scoring compounds.
- A **promoted** variant (`status = "live"`) is persistent: an **executor** runs it daily.
- Discarded variants keep their folder (`status = "retired"`) — negative results are knowledge.

## 3. Roles

Every role has a playbook (`roles/<role>/PLAYBOOK.md`), its own memory, and an inbox.
Role continuity lives in **memory files in git, not conversation history** — any session that
loads the playbook + memory *is* the role. Details and message format:
[`roles/README.md`](roles/README.md).

| Role | Cadence | Does |
|------|---------|------|
| **CEO** | daily (Felix's trigger) | Orchestrates everything; owns structure; never researches |
| **Market researcher** | daily (spawned by CEO) | Scans markets, one strategy idea/day, reads papers, maintains wiki |
| **Researcher** | daily per active slot | Develops one variant: code, backtests, predictions, more applicable markets |
| **Executor** | daily per live variant | Runs live variants; small logged adjustments allowed; fundamental changes → CEO inbox |
| **Felix** (human) | — | Owns trigger, budget, secrets; inbox at `roles/felix/inbox/` |

The CEO may create new roles (e.g. a CRO/CTO) when justified — early on it should do the
ops work itself. Every role change is a structural change → `ops/decisions.md`.

## 4. Scheduling

Felix configures exactly **one daily trigger: the CEO**. The CEO spawns all other roles as
subagents within its run, parallelizing independent work and serializing canonical writes.
The CEO may later create additional triggers itself (it has scheduling tools in its
environment) — but every trigger must fire inside the working window in
[`CONSTITUTION.md`](CONSTITUTION.md) and be recorded in `ops/state.toml`.

## 5. Predictions, scoring, execution

- **Signal generation** — researchers and executors produce probabilities; every prediction
  is one row per outcome token in [`predictions/predictions.csv`](predictions/README.md)
  (append-only, **single writer: the CEO's orchestration**, never concurrent).
- **Scoring** — [`scoring/`](scoring/) joins predictions with
  `predictions/resolutions.csv` → Brier + log-loss per variant / family / model / status +
  a market baseline (market price at prediction time). Scoring is what makes knowledge
  compound; an unscored prediction is worthless as evidence.
- **Tradeability** — [`tools/fillcheck/`](tools/fillcheck/) replays Polymarket's public
  trade feed and writes `predictions/fills.csv`: the best price a counterparty was
  demonstrably reachable at, per prediction row. Scoring joins it and reports
  `n_fillable` / `exec_edge` beside every Brier number, because the two answer different
  questions and only one of them is money — our first batch beat the market 21/21 and was
  reachable 2/21 ([`wiki/reference/midpoint-is-not-a-fill.md`](wiki/reference/midpoint-is-not-a-fill.md)).
  Run fillcheck **before** scoring; scoring alone silently drops the column.
- **Execution policies** — [`execution/`](execution/) turns sets of predictions into
  paper-trades: versioned rule sets (edge threshold, sizing, liquidity respect, exit) with
  signal **combination folded in** (when multiple variants cover one market). Policies are
  backtestable: replay over the predictions log + stored prices → PnL curve. Calibration
  (Brier) and profitability (PnL) are different things; we measure both. Real trading is
  out of scope but this is its staging ground.

## 6. Data: git is the index, R2 holds the bytes

poly committed 70 MB of data snapshots into git; orakel does not.

- **Git**: all markdown, code, configs, and small canonical CSVs (predictions, resolutions,
  scores). Rough threshold: anything over ~100 KB or binary goes to R2.
- **R2**: every frozen dataset, uploaded via [`tools/r2data/`](tools/r2data/) under an
  **immutable content-addressed key** (`blobs/<sha256>`), with a small manifest JSON
  (`<name>.r2.json`: key, sha256, bytes, source URL, fetched-at) committed to git where the
  data logically lives.
- **Consistency rule (hard):** upload to R2 **before** the git commit that references the
  manifest. Git must never point at bytes that don't exist; an orphaned R2 object is
  harmless, a dangling manifest is a bug. Keys are never overwritten or deleted while
  referenced.
- Reproducibility is unchanged from poly: agents snapshot what they pull; the manifest is
  the freeze.
- **Bucket layout** (`orakel`): `blobs/<sha256>` — research freezes, immutable, git-
  manifested; `snapshots/books/<YYYY-MM-DD>/<HH>.json.gz` — hourly price/book snapshots
  written by the snapshot Worker ([`workers/snapshot/`](workers/snapshot/)), immutable,
  **not** individually manifested (keys are deterministic from time — git documents the
  scheme, not each object); `config/watchlist.json` — the mutable list of markets to
  snapshot, mirrored by the CEO from the union of active `applications/` whenever it
  changes.

## 7. Dashboard

A dynamic Rust app on Cloudflare Workers ([`dashboard/`](dashboard/)) — the human window
into the firm, private behind Cloudflare Access. It reads git via the GitHub API (cached),
big data via a native R2 binding, and live Polymarket prices server-side; nothing needs a
redeploy to refresh data. Server-rendered (askama) + Tailwind + htmx partial swaps +
ECharts. Pages: operations (state, decisions, runs, spend), strategies (slots, lineage,
track records), predictions vs market, inboxes, research browser (rendered markdown), and
later execution/backtest views. Deployed with `wrangler deploy` from agent sessions.

## 8. Tech stack

- **Rust-first** (nudge, not law): inline `cargo -Zscript` for one-offs, real crates for
  anything sophisticated. See [`CODING.md`](CODING.md).
- Polymarket data via Gamma / CLOB / Data APIs (see
  [`wiki/recipes/polymarket-api.md`](wiki/recipes/polymarket-api.md)) — read-only, no keys.
- Secrets via environment variables (`R2_*`, `CLOUDFLARE_API_TOKEN`); never committed.

## 9. Model routing & provenance

Sessions run on different models with different strengths and costs; poly's scored data
showed the model matters. Current routing policy lives in `ops/state.toml`; the standing
default is in [`CONSTITUTION.md`](CONSTITUTION.md). Every prediction row and worklog entry
records the **exact model id** that produced it, so scoring can separate method-edge from
model-edge.
