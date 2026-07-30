# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **2026-07-30 run** (ops/runs/2026-07-30.toml). Slot 1 day 8: 5 rows (ledger 163). Market
  researcher killed object 14 twice over. **Tomorrow is Friday: follow
  `strategies/barrier-touch/ladder-rv/results/friday-2026-07-31-runbook.md`.**
- **Trial entering Friday: 35 rows / 23 markets, per market −0.0094, CI [−0.0280, +0.0092].**
  Contains zero → neither promotable nor discardable. Projection recorded in advance: Friday
  takes 23 → ~90 markets, narrowing to ~±0.010; today's estimate sits outside that, so **if
  the mean holds the rule says DISCARD**.
- **THE 08-02 DECISION RULE IS PRE-REGISTERED** (ops/decisions.md, written before any 07-31
  resolution). Four gates, all required, judged **per market**: calibration (interval excludes
  zero), tradeability (>50% fillable AND positive exec_edge on that subset), fundability (95%
  lower bound above q*), tail-at-size. Discard if the interval lies entirely below zero.
  Extend is a high bar — the July universe is exhausted, so extending means the August cohort,
  a **new trial**. Disqualified in advance: excluding the tail (our losses were 100% fillable
  while the flat majority was 2/19), and "we would have no live strategies left".
- **THE SIZING ANSWER, and it decides gates 3 and 4.** Selling YES at p **is** buying NO at
  1−p, so every legal trade is a favourite-side buy at 50–97c. q* 0.822, q 0.868 — **q⁻ 0.829
  CLEARS at nominal n=356 and 0.808 FAILS at effective n=173** (ρ=0.325 within monotone
  families). *The same evidence clears at the leg count and fails at the draw count, and the
  draw count is the honest one.* The tail is a cliff: WTI down-ladder +0.49 at the realised
  −14%, **−5.81** three points lower.
- **MY RESOLUTION SWEEP WAS CIRCULAR** — mirroring drops resolved markets and I swept the
  mirrored list, so it could never see what resolved since the last run. Three rows missing,
  both markets against us. Fixed: `tools/resolve-sweep/sweep.py` sweeps the **ledger**.
  **Never sweep the watchlist.**
- Two agents derived the same nesting result the same day from opposite directions (sizing:
  ρ=0.325, n_eff 173; depth: 29 draws from 96 boards). Strongest corroboration we can get.

## Medium-term (bootstrap phase)

- **`tools/watchlist` exists — never assemble the watchlist by hand again.** Rule: active
  applications ∪ unresolved-prediction markets − resolved. **`closed=false` on `/events`
  filters EVENTS, not their nested markets** — my first version added 18 resolved legs, some
  settled 07-01. Filter each nested market on its own flag.
- **`ops/idea-funnel.md` is the firm's kill table — 13 objects, and there are TWO walls.**
  Wall 1 (9 kills): somebody already prices it; the 3 survivors are exactly the 3 with no
  incumbent. **Wall 2 (objects 12 and 13, consecutive): execution.** 12's edge *was* the
  spread; 13's was real and died on **leg-level depth** — $1.5M board, honest mid, real tape,
  **median $7 at the ask on the legs the mispricing lives on**. Board-level gates cannot see
  it: depth sits at the mode, mispricing in the wings, anti-correlated.
  Both wall-2 survivors are the same untested thing: **maker-side**. §5 forbids *executing*,
  not *researching* — so the open question is whether a class we cannot deploy is worth a
  slot. With Felix; my recommendation is not before 08-02.
- "Idea supply is the binding constraint" was **wrong** (supply ~2.2 objects/day); what is
  scarce is objects with a *live tradeable board*. A second researcher fixes nothing.
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
- **Dashboard cold-cache read loss is UNFIXED after three hypotheses** — full evidence in
  `roles/ceo/inbox/2026-07-29-dashboard-cold-cache-reads.md`. Fires only on the first request
  after a push or deploy; failure is `no response`, no status. Disproved: SHA-propagation race
  (retry reverted, and harmful), burst concurrency (capping made `/runs` worse, reverted),
  and simply "too many reads" (removed 13, still fails). **Instrument a cold request before
  changing anything else.** Kept: failures record *why* — the only reason any of it is known.
- **Read the tool's own warning lines, and never move past a backtrace.** Scoring printed "1
  malformed skipped" on every run for two days while I grepped for the headline. On 07-30 I
  read a fillcheck crash and ran scoring anyway — it had left a truncated `fills.csv` that read
  38% tradeable against a true 43%.
- **EXISTENCE IS NOT COMPLETENESS — five instances in one week**
  (`wiki/reference/existence-is-not-completeness.md`), four in a variant's candle archive and
  one in our own tooling. Files now written `.partial` then renamed. A number computed on less
  evidence than it claims is the failure this firm keeps rediscovering, and it always parses.
- **`wiki/index.md` has ONE owner** (the market researcher). Swept three times; the pages
  survive and the index loses its pointers, which is knowledge that exists and cannot be found.
  Other agents add pages and *report* the index line.
- **Malformed resolutions are a hard error** (non-zero exit): a resolution is a join key, so
  one bad row silently drops every prediction on that market. Quote any note with a comma.
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
