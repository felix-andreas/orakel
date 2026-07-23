# Decision log

Append-only. Every structural change to the firm gets a dated entry: **what changed, why,
who decided**. Newest first.

---

## 2026-07-22 — CEO instantiated (Felix's instruction)

The scaffolding session is promoted to the CEO: it becomes the CEO's persistent session,
woken daily by a self-bind trigger at 01:07 UTC (03:07 German summer time — inside the
working window year-round). Felix chose self-bind over fresh-session mode because it
keeps all MCP connectors (verified live earlier today; fresh sessions from agent-created
triggers lose them). Model routing per constitution §4: subagents on Fable run at high
effort only; Opus subagents may run extra-high. First CEO run starts immediately:
market researcher scan → first strategy idea → fill research slot 1.

## 2026-07-22 — Founding (Felix + scaffolding session)

The firm is founded as the successor of `poly`, redesigned around lessons from its ~2-week
run. Founding decisions, agreed between Felix and the scaffolding agent:

- **Research unit = strategy variant**, not market. poly's per-market research (3
  researchers/market) produced correlated one-shot papers, n=2-3 per method, and its
  `strategies/` promotion path never fired once.
- **Family → variant → application** taxonomy with the params-plus-small-local-changes
  membership rule; split variants rather than over-generalize. Versions = name postfix +
  `supersedes` field.
- **5 research slots**, ≥10-day trials judged on scored evidence (guideline: ≥15 scored
  predictions across ≥3 markets beating the market baseline + backtests on resolved
  markets). CEO decides promote/discard/extend.
- **Roles with own memory + inboxes**: CEO (orchestrates, never researches), market
  researcher (daily scan → one idea/day), researchers (per slot), executors (per live
  variant). Felix is a role with an inbox.
- **One daily CEO trigger owned by Felix**; CEO spawns everything else and may create
  further triggers inside the working window (weekdays 02:00–15:00, weekends 02:00–08:00
  Europe/Berlin).
- **No hard token cap initially**; spend logged per run. Model routing: Fable
  (high/xhigh) for market research + initial research, Opus for recurring research +
  execution.
- **Git = index, R2 = bytes** (poly committed 70 MB of snapshots into git). Upload-before-
  commit, immutable content-addressed keys.
- **Execution layer from day one** (paper only): versioned execution policies with signal
  combination folded in, PnL-backtestable. Real trading stays a Felix-only decision.
- **Dashboard**: dynamic Rust app on Cloudflare Workers, private via Cloudflare Access,
  htmx + ECharts, deployed from agent sessions via wrangler.
- **Wiki seeded** with a curated handful of durable poly insights (market selection,
  favorite-longshot bias, thin-market price reading, crowd calibration, wash-trade
  detection, Polymarket API recipes); everything else clean slate.
