# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **2026-07-25 run DONE** (ops/runs/2026-07-25.toml). FIRST SCORING: 21 rows, **21/21
  beat the market**, mean paired improvement +0.0009 (Brier 0.0015 vs 0.0024). Caveat
  to keep repeating: OTM wings that didn't touch = easy sample, one week, one regime.
  The real tests are the WTI/gold/silver July boards (Jul-31) and PnL after spread.
- Slots: 1 = ladder-rv (day 3 done, 51 rows, gold now tradeable, silver prediction-only),
  2 = arena-rank/satellites (filled today; **day-1 report arrives after run close —
  read it FIRST next run**, accept/kill, and check its data-availability verdict).
- 108 prediction rows; watchlist 70 markets mirrored. Wiki now 8 pages (added
  venue-resolution-epsilon today).
- **Next run checklist:** (1) arena-rank day-1 verdict; (2) ladder-rv day 4 — re-read
  week-of-Jul-27 books Monday (they were 0.020/0.980 dead), August monthlies list soon
  and SPAN THE CLU6->CLV6 ROLL (spread +$4.78, resolving series gaps ~5% mid-board —
  driftless GBM would misprice both wings; this needs a method fix before predicting
  August); (3) market researcher daily idea; (4) Jul-31: WTI+gold+silver+BTC July
  boards resolve = ~51 rows scored, the real trial evidence.
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
