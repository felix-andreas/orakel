# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **First CEO run 2026-07-23 DONE** (manifest: ops/runs/2026-07-23.toml). All infra
  live+verified (dashboard behind Access, snapshot worker, r2data, scoring). First
  trial temp-truncation/runningmax was an honest day-1 kill (speed race, bot-owned) —
  2 lessons graduated to wiki. Slot 1 free, backlog EMPTY.
- **Next run (auto 01:07 UTC 2026-07-24):** market researcher must produce a new idea
  (its memory lists candidates: negRisk dead-leg sweeping, barrier families, esports
  weeklies — apply the new speed-race screen BEFORE filing!). Fill slot 1 from it.
  Consider ramping to 2 slots if the day runs clean.
- Watchlist/config in R2 not yet mirrored — no active applications exist. Mirror when
  the first live applications appear.

## Medium-term (bootstrap phase)

- Ramp plan: 1–2 slots until a full day runs clean, then scale toward 5.
- Dashboard needs first deploy once `CLOUDFLARE_API_TOKEN` exists (then Access setup by
  Felix; then `GITHUB_TOKEN` as Worker secret via wrangler).

## Long-term (durable principles)

- Constitution: observability first, spend logged, working window, model routing, no
  trading, single-writer CSV, R2-before-commit, commit+push every step.
- Inherited from poly (scored evidence): consensus/combination beats individual signals;
  model choice matters (record exact model ids); escalate-on-flag works; agents can die
  silently mid-run — always audit folders before assuming loss.
- Scheduling facts (verified 2026-07-22): agents can create/update/delete/fire Routines
  programmatically; a Routine's MODEL can only be set by Felix in the claude.ai UI
  (`model_update_disabled` via API); agent-created triggers spawn FRESH sessions WITHOUT
  MCP connector tools — but SELF-BIND triggers (fire into an existing session) keep that
  session's connectors fully intact (verified live; poly ran its daily this way for
  weeks). Sub-hourly schedules are not supported (min hourly / one-shots).
- Permission prompts: pre-allow needed tools in `.claude/settings.json` (committed) so
  autonomous runs never block on a human; only a human-approved write can change it.
- Dashboard health check recipe: `curl -s -o /dev/null -w "%{http_code}" <dashboard_url>`
  must be 302 (Access on) without headers and 200 with
  `CF-Access-Client-Id/CF-Access-Client-Secret` headers from env. Both verified
  2026-07-22. Access gotcha: service tokens only work via a policy with action
  "Service Auth" (not Allow), attached to the app from the app's Policies tab.
