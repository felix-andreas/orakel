# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (PAUSED — read this first on resumption)

- **The daily trigger is DISABLED** (2026-08-02, Felix: "stop your routine trigger for now").
  `trig_017vXv9HCCiTZXVUd3brFuD9`, disabled not deleted — re-enable with one `update_trigger`
  call and this session keeps its context. The **snapshot worker keeps running** (Cloudflare
  cron, no LLM), deliberately: book history is the one dataset that cannot be reconstructed
  after the fact.
- **THE TRIAL REVIEW IS UNRUN.** Due 08-03, already slipped once by the completeness gate.
  Everything needed is written: the four-gate rule (`ops/decisions.md` 07-30), the form
  (`ops/reviews/2026-08-02-ladder-rv.md`), the independent reviewer's brief
  (`ops/reviews/README.md`). Only input still missing:
  `will-wti-reach-90-from-july-27` (4 rows), in UMA dispute. The 08-01 rule applies directly —
  still disputed ⇒ excluded, named, re-scored later, **only if the reviewer verifies the
  verdict does not turn on it**.
- **PRE-REGISTRATIONS DO NOT EXPIRE.** The decision rule, the completeness gate, the
  disputed-market exception and the independent-evaluator rule were all written before the
  numbers existed. A pause is not grounds to revisit any of them — a gap makes them *more*
  valuable, because they are what lets a resumed review be the same review.
- **Trial frozen at 163 rows / 82 markets, per market −0.0025, CI [−0.0078, +0.0027]** —
  contains zero. Tradeability 63%. Gate 3 may be decidable *without* the resolutions at all
  (edge +0.73pp vs 1.00c median half-spread); ask the reviewer explicitly.
- If ladder-rv is discarded: **zero live strategies and an empty backlog** — both former seeds
  are known W1-dead. A fact to state, not an argument for promoting something.
- **Dashboard cold-cache loss: measured, concurrency refuted.** Cold `/runs` is reproducibly
  `attempted=35 hit=0 net=35 failed=22 span_ms=369`; bounding to 4 gave an identical 22.
  Surviving hypothesis: subrequest budget, **Cache API ops spend it too**. Next experiment is
  named and cheap — disable the cache for one deploy, read `failed` off the footer.

## Medium-term (bootstrap phase)

- **`tools/watchlist` exists — never assemble the watchlist by hand again.** Rule: active
  applications ∪ unresolved-prediction markets − resolved. **`closed=false` on `/events`
  filters EVENTS, not their nested markets** — my first version added 18 resolved legs, some
  settled 07-01. Filter each nested market on its own flag.
- **`ops/idea-funnel.md` is the kill table — 17 objects, FOUR walls.** Run them cheapest first.
  **W1 incumbent** (9 kills): somebody already prices it. The 3 survivors are exactly the 3 with
  no incumbent found. *Vendor-generic tickers carry millions of contracts while object-specific
  ones are 0-market shells — search both.*
  **W2 execution** (12, 13): 12's edge *was* the spread; 13 died on leg-level depth, median $7
  at the ask. Object 14 **cleared** it, so the depth walk has a pass state and is specific to
  ladders with a mode.
  **W3 power** (14): a 12-rung nested ladder is ONE observation — 29 draws at 0.88/month against
  91 needed. **Needs no data at all**; run it first.
  **W4 carry** (15): a guaranteed profit is not an edge until it beats the risk-free rate, and
  check π\* **at the tenor where the volume is**. The firm's first real arb was +23.90c, died on
  depth at **$8.88**, then carry at **+0.35% vs ~4%**.
  **Object 17 cleared W1 OUTRIGHT** — Kalshi wrote the identical contract, our exact settlement
  URLs, **0 markets ever**, while a 50k-contract sibling is live — and is the **first object to
  prove its edge is not the spread** (+8.43c/share at the ask, mirror −23.02c). Died on depth
  rotated onto TIME: **85.5% of the tape prints after the resolution instant**, leaving $76 a leg.
  **W1 and W3 may be anti-correlated BY CONSTRUCTION** — no-Kalshi-twin families are almost all
  monthly-or-rarer (W3-dead), everything daily/weekly has an incumbent. Quantify at row 20.
  **New instrument:** `/series` censuses all 2,152 recurring families with cadence in ~44 calls,
  so W1 and W3 run together over the whole venue BEFORE an object is picked.
  Object 15 was built to dodge W3 and died on the other three — the walls are not a sequence you
  can route around. Both W2 survivors are the same untested thing, **maker-side**: §5 forbids
  *executing*, not *researching*, so the open question is whether a class we cannot deploy is
  worth a slot. With Felix; not before 08-02.
  **Object 16 is the closest miss and the most useful failure.** First to clear W3 (~1,120
  draws/yr vs 243) and the largest W2 margin on record ($19.4M at the ask, flat to $10k). Ran the
  backtest: **0 of 169** settled tail legs resolved Yes, +12.97pp over risk-free at executable
  prices — **and it still fails**, Wilson upper 2.22% vs π\* 0.44% at 150d. A perfect record,
  reachable at size, uninvestable. The escape route is closed by the VENUE: ≤45d holds 0.5% of
  band volume, ≥150d holds **97.8%** at π\* ≤ 0.75%. A maturity-schedule failure — better
  modelling does not touch it. It also died on the gap's **sign**: Kalshi prices the same tails
  HIGHER, 8/8 pairs, p=0.0039.
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
- **A perfect record is not an edge.** 16/16 at t=+10.3 and 0-for-169 at +12.97pp both fail their
  break-even bound. Report q\*/π\*, the point estimate, and the 95% bound — and refuse on the bound.
- **Report per-MARKET before per-ROW.** Rows are not independent: 19 markets −0.0051 vs 25
  rows −0.0173, 3.4× worse purely from predicting the losing market four times.
- **Gamma's `closed` is a FILTER in BOTH directions**, and worse: on a resolution day the
  batch is MIXED, so one query form returns half the rows looking complete. `condition_id`
  singular is silently ignored and serves an unrelated market — always verify the id that
  came back. All in `wiki/recipes/polymarket-api.md`.
- **Dashboard cold-cache read loss: instrumented 08-02, three hypotheses now refuted** —
  `roles/ceo/inbox/2026-07-29-dashboard-cold-cache-reads.md`. Fires only after a SHA change.
  Refuted by measurement: SHA-propagation race, **concurrency** (bounding gave an identical
  `failed=22`), and time (369ms). Surviving: the subrequest budget with Cache API ops spending
  it. Reads now count themselves into the footer — that telemetry is what settled it, and it is
  the general lesson: measure the failing request before touching the code.
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
