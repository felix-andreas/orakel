# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **2026-07-31 run** (ops/runs/2026-07-31.toml). Last predicting day; boards closed 21:00Z
  tonight. Slot 1 day 9: 4 rows (ledger 167). Market researcher killed object 15.
- **SATURDAY 08-01 IS THE SCORING RUN.** Follow `results/friday-2026-07-31-runbook.md`, which
  slot 1 dry-ran and **corrected today — four steps were wrong, two failing silently with exit
  0.** Before anything: **move `data/events` aside** so `discover` refetches (it caches on file
  existence — the SIXTH existence-is-not-completeness instance). `tape`/`wash` take **space**-
  separated args; `discover`/`live` take **one comma**-separated arg; the wrong form matches
  nothing and exits 0. Judge `selftest` by exit code. The Saturday BTC pass needs its own
  completeness block (Friday's omits BTCUSDT).
- **THE EDGE IS SMALLER THAN THE SPREAD, and this does not depend on Friday.** Nominal margin
  **+0.73pp** against a median half-spread of **1.00c**; selling at the bid gives q* 0.8316 vs
  q⁻ 0.8289 — failing by 0.27pp at nominal n, zero fee, before any correlation argument.
  Break-even half-spread 0.73c against a median 2.0c-wide book. n_eff ∈ **[118, 173]**, failing
  across the range. Kelly at the 95% lower bound is **negative**.
- **THE REVIEW IS EVALUATED BY AN INDEPENDENT AGENT, not me and not slot 1** (decided 07-31,
  before any board closed). Form: `ops/reviews/2026-08-02-ladder-rv.md`, thresholds verbatim,
  PASS/FAIL/UNEVALUABLE per gate. Brief: `ops/reviews/README.md`. **Slot 1 supplies analysis,
  never verdicts** — it handed me one twelve hours after the rule and I did not take it, because
  accepting an unfavourable verdict establishes that it gives verdicts at all.
- Ask the reviewer explicitly: **is gate 3 decidable without Friday's resolutions?** If so, the
  outcome was set by the book rather than the forecast, and nine days of prediction measured the
  wrong question.

## Medium-term (bootstrap phase)

- **`tools/watchlist` exists — never assemble the watchlist by hand again.** Rule: active
  applications ∪ unresolved-prediction markets − resolved. **`closed=false` on `/events`
  filters EVENTS, not their nested markets** — my first version added 18 resolved legs, some
  settled 07-01. Filter each nested market on its own flag.
- **`ops/idea-funnel.md` is the kill table — 15 objects, FOUR walls.** Run them cheapest first.
  **W1 incumbent** (9 kills): somebody already prices it. The 3 survivors are exactly the 3 with
  no incumbent found. *Vendor-generic tickers carry millions of contracts while object-specific
  ones are 0-market shells — search both.*
  **W2 execution** (12, 13): 12's edge *was* the spread; 13 died on leg-level depth, median $7
  at the ask. Object 14 **cleared** it, so the depth walk has a pass state and is specific to
  ladders with a mode.
  **W3 power** (14): a 12-rung nested ladder is ONE observation — 29 draws at 0.88/month against
  91 needed. **Needs no data at all**; run it first.
  **W4 carry** (15): a guaranteed profit is not an edge until it beats the risk-free rate. The
  firm's first real arb was +23.90c, died on depth at **$8.88** total, then on carry at **+0.35%
  annualised vs ~4%**.
  Object 15 was built to dodge W3 and died on the other three — the walls are not a sequence you
  can route around. Both W2 survivors are the same untested thing, **maker-side**: §5 forbids
  *executing*, not *researching*, so the open question is whether a class we cannot deploy is
  worth a slot. With Felix; not before 08-02.
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
- **EXISTENCE IS NOT COMPLETENESS — SIX instances in one week**
  (`wiki/reference/existence-is-not-completeness.md`), four in a variant's candle archive and
  one in our own tooling, and on 07-31 `discover` caching on `p.exists()` with no completeness
  guard. Files now written `.partial` then renamed. A number computed on less evidence than it
  claims is the failure this firm keeps rediscovering, and it always parses.
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
