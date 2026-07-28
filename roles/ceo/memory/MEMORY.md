# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **2026-07-28 run** (ops/runs/2026-07-28.toml). Slot 1 = ladder-rv day 6; 14 rows appended
  (ledger 146, scored still 25 — nothing new resolved). Watchlist 56 → 100 markets.
- **THE STALE-FEED NUMBER GOT WORSE AND MORE PRECISE.** Slot 1 backfilled feed state for
  every row we have ever written, from the frozen candle archive rather than from memory:
  **68 of 132 rows (52%)** were priced off a shut feed, not yesterday's 64 of 95. Days 1–2
  emitted 20 stale equity rows nobody had counted. **Every equity row the firm has ever
  emitted was stale.** 07-25 corrected 4.5h → 4.9h.
- **Equity can NEVER be predicted inside the working window.** Zero overlap with Pyth RTH,
  by half an hour, in BOTH DST regimes (window 00:00–13:00Z summer / 01:00–14:00Z winter;
  RTH 13:30–20:00Z / 14:30–21:00Z). Not a bad hour — a structural impossibility. Filed to
  Felix as no-reply-needed; the gate already suppresses equity, so nothing is blocked. My
  recommendation is **don't widen the window**: equity weeklies were already the weakest
  family (38% reachable) and we would be spending his usage limit to chase it.
- **Ledger schema widened**: `pricer_version`, `feed_age_h`, `feed_open`. Scoring gained a
  **`pricer` aggregate level**, so Friday's split is a table row, not hand-work. 121 rows on
  `2026-07-23-touch-prob`, 11+14 on `2026-07-27-touch-prob-jump`.
- **Friday 07-31 needs the Wednesday and Thursday runs to happen.** The pricer split is
  confounded with feed state (every stale row is also an old-pricer row), so it can only be
  scored within `feed_open=1` — 50 rows vs 25 today, and 25 is below the n≥30 floor. Trigger
  verified armed for 07-29 01:07Z. 120 outstanding rows / 58 markets, identity 58/58 clean.
- RV/IV blend is **pre-registered, not switched** (`results/prereg-rv-iv-blend-2026-07-28.md`):
  the IV anchor sits above realized vol on **62 of 62** legs, and for a sell-only variant a
  higher σ destroys signals rather than creating them — RV 4 sell signals, blend 3, IV 1.

## Medium-term (bootstrap phase)

- **`tools/watchlist` exists — never assemble the watchlist by hand again.** Hand-assembly
  lost markets three times (mirrored after predicting; markets that came from predictions not
  applications; only the legs we predicted on rather than the whole applied-for board — 44
  legs). Rule is now executable: active applications ∪ unresolved-prediction markets − resolved.
- **`ops/idea-funnel.md` is the firm's kill table.** The three survivors are exactly the three
  objects where **no incumbent was found** — no exceptions in either direction. And my
  "idea supply is the binding constraint" was **wrong**: supply is ~2.2 objects/day. What is
  scarce is objects arriving with a *live tradeable board*. A second researcher fixes nothing.
- `slots_total = 5` is a **ceiling, not a target**. Filling a slot to look busy cost us
  `tomatometer/arrival-drift`, dead the day it was filed.
- Fee model is real and priced (v2 policies). Buys LOSE outright (−0.22c) after fees.

## Long-term (durable principles)

- Constitution: observability first, spend logged, working window, model routing, no
  trading, single-writer CSV, R2-before-commit, commit+push every step.
- **Calibration ≠ tradeability ≠ fundability.** Three gates; a variant can pass one and fail
  the next two. `wiki/reference/break-even-win-rate.md` is the firm's strongest artifact — a
  band that went 16/16 with t=+10.3 is uninvestable because 2.83 losses per 100 take it to zero.
- **Four ways a quoted price misleads**, all wiki pages: phantom midpoints (dead book),
  midpoint-is-not-a-fill (you trade at the bid), tape-gate (tight spread, zero trades ever),
  stale-feed-gate (**ours** is the broken number).
- **Report per-MARKET before per-ROW.** Rows are not independent: 19 markets −0.0051 vs 25
  rows −0.0173, 3.4× worse purely from predicting the losing market four times.
- **Gamma's `closed` is a FILTER in BOTH directions**, and worse: on a resolution day the
  batch is MIXED, so one query form returns half the rows looking complete. `condition_id`
  singular is silently ignored and serves an unrelated market — always verify the id that
  came back. All in `wiki/recipes/polymarket-api.md`.
- **The dashboard's SHA pinning has a propagation race**, fixed 07-28: `head()` learns a SHA
  from one GitHub replica, the content read hits one that hasn't seen it, and every read on
  the page fails at once. Measured: banner on the first request after a push, twice; 0 in 40
  post-fix. A failed pinned read now retries unpinned at `main`.
- Inherited from poly: consensus beats individual signals; record exact model ids; agents can
  die silently mid-run — always audit folders before assuming loss.
- Scheduling: self-bind triggers keep MCP connectors; agent-created fresh-session triggers do
  not; a Routine's MODEL is Felix-only via the claude.ai UI. Min hourly.
- **Concurrency: never `git add -A` while agents run.** Stage explicit paths. The stop hook
  fires on every multi-agent run because it cannot tell agent-owned in-flight files from
  neglect — check `git status` against running agents before believing it.
- Health checks: dashboard 302 without Access headers / 200 with; snapshot worker is
  `orakel-snapshot` **singular** (the plural typo returns a 404 that reads like an outage);
  `r2data verify <manifests>` takes explicit paths; worklogs current; no stale inbox items.
  Access gotcha: service tokens need a policy with action "Service Auth".
- TLS behind the agent proxy: **ureq's default rustls ignores `SSL_CERT_FILE`** and fails with
  `UnknownIssuer`. Use `attohttpc` with `tls-native`, the same stack `rust-s3` uses here.
