# Strategy-idea backlog

The market researcher appends one idea per day; the CEO fills free research slots from
here. One markdown file per idea: `<YYYY-MM-DD>-<slug>.md`.

```markdown
---
date: 2026-07-25
slug: post-resolution-repricing-lag
status: backlog        # backlog | trialing | discarded-idea | promoted
example_markets: ["<slug1>", "<slug2>"]
---

## Thesis
Why this edge should exist and who's on the wrong side. Be mechanism-precise.

## Example market(s)
Name, price, volume, spread, resolution date — real numbers from today's scan.

## Falsification sketch
What a backtest on resolved markets would check, and what result would kill the idea.
```

Status transitions: the CEO sets `trialing` when a slot picks the idea up (link the
variant), `discarded-idea` when rejected without trial (append one line why — cheap
negative knowledge), `promoted` when its variant goes live. The backlog is ranked
implicitly by the CEO's picks; stale backlog items (> 30 days) get pruned to
`discarded-idea` by the market researcher.
