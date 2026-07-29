# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **2026-07-29 run** (ops/runs/2026-07-29.toml). Slot 1 day 7: 12 rows (ledger 158). Market
  researcher killed post-count ladders. Watchlist 84 markets.
- **HEADLINE −0.0452 over 32 rows / −0.0133 over 20 markets.** WTI dipped to $80. Two
  markets carry all of it — dip-to-80 and dip-to-85, both **fully fillable** — while 18 of 20
  sit at or above zero. By horizon: 0–1d **−0.0001** (2/19 fillable), 1–3d −0.1211, 3–7d
  −0.1190, both nearly fully fillable. **Where we could trade we were wrong.**
- **MY exposure hypothesis was REFUTED.** I proposed the variant is structurally short
  downside touch; measured on 633 legs it **beats** the market on touched legs (t −1.99) and
  down legs trending into the barrier are its **best** bucket (t −4.66). What is real is a
  tail: the 8 worst legs of 633 are all `dip-to`, and our two losses are nested on one
  contract — **~1 draw, not 2**. **08-02 is a SIZING question, not a Brier one.**
- **Three calls made blind before Friday** (ops/decisions.md): the pricer split is
  **INCONCLUSIVE** — n≥30 clears in rows (37) and not in markets (19), markets is the unit we
  named in advance, and the board universe is exhausted so no schedule reaches 30; the 08-02
  decision may not rest on it. RV/IV anchors at the **emission time (~01:1xZ)**, not the
  prereg's 12:00Z, and is scorable from only two days so expect it underpowered. A
  **completeness gate** on the review: proceeds only when every outstanding row is resolved,
  else +1 day, once.
- **A resolution was silently dropped for two days** by an unquoted comma in its note (6
  fields, skipped as malformed). A resolution is a JOIN KEY — it removed every row on that
  market. The warning printed on every run and I grepped past it twice. Malformed
  *resolutions* are now a hard error with non-zero exit; malformed prediction rows stay a
  warning.
- Equity still can NEVER be predicted in the working window (zero overlap with Pyth RTH, both
  DST regimes). Gate suppresses it automatically; Felix item is no-reply-needed.

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
- **Read the tool's own warning lines.** Scoring printed "1 malformed skipped" on every run
  for two days while I grepped for the headline. A number that looks complete and is computed
  on less evidence is the failure mode this firm keeps rediscovering.
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
