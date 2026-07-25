# Strategies

The heart of orakel. Taxonomy: **family → variant → application** (see
[`ARCHITECTURE.md`](../ARCHITECTURE.md) §1 for rules; this file is the practical layout +
schema reference).

```
strategies/<family>/
├── FAMILY.md                     # the idea in spirit; cross-variant lessons; thin
└── <variant>/
    ├── STRATEGY.md               # the runbook — always reflects the method's CURRENT state
    ├── strategy.toml             # manifest (schema below)
    ├── src/                      # this variant's code (own crate or -Zscript files)
    ├── applications/<market>.toml  # per-market params (schema below)
    ├── data/                     # R2 manifests (*.r2.json) for frozen datasets
    ├── memory/                   # MEMORY.md + WORKLOG.md of the researcher/executor
    └── results/                  # backtests, trial notes, post-mortems
```

Copy [`_template/`](_template/) to start a variant. Names: short kebab-case; variant names
are unique within their family (`barrier-touch/gbm`, `barrier-touch/gbm-v2`).

## strategy.toml

```toml
family = "barrier-touch"
variant = "gbm"
# REQUIRED: one sentence, plain English, no jargon - what the idea IS, understandable by
# a smart outsider. This is what the dashboard shows. Internal vocabulary belongs in
# STRATEGY.md, never here.
summary = "Bets on whether a price will ever touch a far-away level are priced too generously; we sell the far-fetched ones."
# REQUIRED: a SELF-CONTAINED explanation, plain English, no jargon. A smart outsider with
# no prior knowledge must be able to read this cold and understand (a) what the market is
# and how it works, (b) what we do, and (c) WHY that should work - the motivation, not
# just the claim. 4-8 sentences. State the honest catch too. This is what the dashboard
# shows on the strategy page; jargon and internal detail belong in STRATEGY.md.
explainer = """
Polymarket runs bets like "will the price of oil ever touch $110 before the month ends?"
... (see any live strategy.toml for a full example)
"""
status = "trial"          # trial | live | retired  (CEO owns transitions)
created = "2026-07-25"
supersedes = ""           # e.g. "gbm-v1" — the one piece of version lineage
labels = ["class:first-passage", "data:price-history"]

[trial]                   # while status = "trial"
slot = 1
started = "2026-07-25"
review_due = "2026-08-04" # >= started + 10 days
success_guideline = "≥15 scored predictions across ≥3 markets, Brier < market baseline"

[retirement]              # once status = "retired"
date = ""
reason = ""               # one honest sentence; details in results/post-mortem.md
```

## applications/<market>.toml

```toml
market_slug = "btc-67500-july-2026"
condition_id = "0x..."
added = "2026-07-25"
active = true             # false = paused (say why in a comment)
# ... then whatever parameters the variant's code needs:
[params]
barrier = 67500.0
```

## Lifecycle recap

`ideas/` backlog → CEO assigns a slot → **trial** (researcher develops it daily) →
CEO promotes (**live**, executor takes over) or discards (**retired**, folder stays,
post-mortem in `results/`). Status changes are CEO-only and always paired with
`ops/state.toml` + `ops/decisions.md` updates in the same commit.
