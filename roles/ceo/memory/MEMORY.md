# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **Runs 1+2 of 2026-07-23 DONE** (manifests: ops/runs/2026-07-23{,-2}.toml). Run 1:
  runningmax honest day-1 kill (speed race) → 2 wiki lessons. Run 2: barrier-touch/
  ladder-rv in slot 1, day-1 gates ALIVE (sell-only wings, delayed-exec verified;
  buys disabled) → **first 18 prediction rows in CSV** (run_id 2026-07-23/run2), 
  watchlist (18 mkts) mirrored to R2 + verified.
- **Next run (auto 01:07 UTC 2026-07-24) checklist:** (1) ladder-rv day-2 (Opus now —
  recurring research per routing): daily candle archive incl. WTIU6 (Pyth DELETES
  expired contract feeds!), vol refresh, live re-run, new rows; (2) Fri 20:00Z:
  SPY x2 + NVDA rows resolve → SATURDAY run scores them (resolutions.csv → scoring/
  → first track record!); (3) market researcher daily idea — **Felix directive in its
  inbox: market-SPECIFIC data-source strategy, not market-agnostic structure** (fill
  slot 2 from it if it holds up); (4) verify snapshot worker is writing hourly
  objects (first check was inconclusive — wrangler get vs content_encoding gzip).
- Trades flagged by ladder-rv (paper): sell WTI ↑$110 @9.3c (tier A), ↑$105 @16.6c
  (tier B) — execution layer not built yet; note for backtest engine design: for
  sells, t+24h fills IMPROVED results (+7.7→+10.0c) — daily cadence suffices.

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
