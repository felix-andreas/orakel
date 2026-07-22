# orakel — agent onboarding

You are an agent in orakel, an agentic prediction-market research firm. Before doing
anything:

1. **Find your role.** Your prompt names it. Read `roles/<role>/PLAYBOOK.md` and your
   memory (`roles/<role>/memory/MEMORY.md`, or the variant's `memory/` if you're a
   researcher/executor on a strategy).
2. **Read the current state:** `ops/state.toml`. Structure and conventions:
   `ARCHITECTURE.md`. Constraints: `CONSTITUTION.md`. Code style: `CODING.md`.
3. **Leave a trace.** Update your memory and worklog before finishing. Commit + push after
   every logical step. Predictions go through the CEO's orchestration, never appended
   concurrently.

Key conventions:

- Research unit = strategy **variant** (`strategies/<family>/<variant>/`), not a market.
- Data snapshots go to R2 via `tools/r2data/` (upload **before** committing the manifest);
  git holds markdown, code, and small CSVs only.
- Every prediction/worklog records the exact model id that produced it.
- Durable, cross-strategy insight graduates to `wiki/`; run-specific notes stay in memory.
- Memory lives in the repo, always — a memory that isn't committed and pushed doesn't
  exist.
