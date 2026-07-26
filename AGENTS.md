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
4. **Always work directly on `main`.** No feature branches, no PRs. The repo is the firm's
   shared state: an unpushed branch is invisible to every other agent, and R2 has no
   branches — data referenced from a side branch drifts out of sync and gets lost. Pull
   `main` before you start, push to `main` when you commit.

Key conventions:

- Research unit = strategy **variant** (`strategies/<family>/<variant>/`), not a market.
- Data snapshots go to R2 via `tools/r2data/` (upload **before** committing the manifest);
  git holds markdown, code, and small CSVs only.
- Every prediction/worklog records the exact model id that produced it.
- Durable, cross-strategy insight graduates to `wiki/`; run-specific notes stay in memory.
- Memory lives in the repo, always — a memory that isn't committed and pushed doesn't
  exist.
- **Never `git add -A`, `git add .`, or `git commit -a`.** Several agents share one checkout
  and work concurrently. Stage the explicit paths you own. This has now gone wrong twice: on
  2026-07-25 a blanket add swept a dashboard agent's in-progress diff into an unrelated
  commit, and on 2026-07-26 a dashboard agent swept a market researcher's six files into a
  commit titled "rename Execution → Backtest". Both times the content survived and the
  history lied about who did what — and the fix (rewriting shared history while another
  agent is live) is more dangerous than the problem, so it does not get fixed. Stage
  explicitly, and `git pull --rebase` before every push.
