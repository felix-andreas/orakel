# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **2026-07-27 run** (ops/runs/2026-07-27.toml). Slot 1 = ladder-rv only; 11 rows appended
  (ledger 132, scored 25). Market researcher filed nothing, correctly.
- **THE HEADLINE FLIPPED. WTI touched $85 and ladder-rv now LOSES to the market**:
  +0.000945 (21 rows) → **−0.0172** (25 rows). dip-to-85 alone is −0.4510, more than the total
  loss; everything else nets +0.0198. Tradeability 2/21 → 6/25 — **where there was liquidity we
  were wrong; where we were right there was no liquidity.**
- **Cause is structural, not miscalibration. STALE-FEED GATE ADOPTED.** WTI/metals trade
  22:00Z–21:00Z Mon–Fri; the feed was shut 28.8h while the book repriced 0.475 → 0.715 during
  exactly that closure. Our model moved −2.8 points from a vol lookback sliding across closed
  days — arithmetic that looks like a view. **64 of 95 outstanding rows were priced off a shut
  feed.** Weekend runs now emit nothing on WTI/gold/silver. `wiki/reference/stale-feed-gate.md`
  is the FOURTH way a quote misleads and the only one where OUR number is broken, not theirs.
- **The null/leg-sum check CLEARS** (asked since 07-25, slipped 3 runs). No null beats the
  market at either checkpoint we use, all seven assets, both anchors. The headline is not an
  artifact. But the board-CREATION anchor does lose to a null (85% of legs at 45–55c) — never
  use it. And **gold's window-open claim is withdrawn** (leg-sum gate takes −0.0189 → −0.0001);
  gold stays tradeable on the daily-checkpoint margin instead. **Stop quoting the pooled
  window-open edge — it reverses under the gate.**
- **Scoring now aggregates per MARKET.** Rows are not independent: 19 markets −0.0051 vs 25 rows
  −0.0173. Report both; the row number flatters and deflates alike.
- **Eleven ideas, 7 of 9 kills are "somebody already prices it well."** Filed as a hypothesis
  under test in decisions, no action attached. If the war-market idea dies too, the question
  becomes "is this the right pond" — Felix's, not mine.

## Medium-term (bootstrap phase)

- Ramp plan: idea SUPPLY is the binding constraint on slots, not capacity. One researcher
  a day with a ~70% day-1 kill rate can never fill 5 slots.
- Dashboard: live repo reads, no fallback copy (a stale copy served silently is worse than a
  visible gap), sha-pinned + concurrent reads (0.87s → ~0.4s). Deploy after any status or
  schema change.
- Fee model is real and priced (v2 policies). Buys LOSE outright (−0.22c) after fees.

## Long-term (durable principles)

- Constitution: observability first, spend logged, working window, model routing, no
  trading, single-writer CSV, R2-before-commit, commit+push every step.
- **Calibration ≠ tradeability ≠ fundability.** Three separate gates, and a variant can pass
  one and fail the next two: paired Brier (are we right), tape/fill (can we transact), and
  the break-even bound (is it worth the locked capital). `wiki/reference/break-even-win-rate.md`
  is the strongest artifact the firm has — a band that went 16/16 with t=+10.3 is
  uninvestable because 2.83 losses per 100 take it to zero.
- **Three ways a quoted price lies**, all now wiki pages: phantom midpoints (dead book),
  midpoint-is-not-a-fill (live book, but you trade at the bid), tape-gate (tight spread,
  listed depth, ZERO trades ever).
- Gamma's `closed` is a FILTER not an include-flag, in BOTH directions. Omitting it makes a
  resolution sweep structurally incapable of finding anything; including it makes an
  open-market check structurally incapable of finding anything. I made the first mistake
  this run and caught it only because slot 1 reported it.
- Inherited from poly: consensus beats individual signals; record exact model ids; agents
  can die silently mid-run — always audit folders before assuming loss.
- Scheduling: self-bind triggers keep MCP connectors; agent-created fresh-session triggers
  do not; a Routine's MODEL is Felix-only via the claude.ai UI. Min hourly.
- **Concurrency: never `git add -A` while agents run.** Stage explicit paths. The stop hook
  fires on every multi-agent run because it cannot tell agent-owned in-flight files from
  neglect — check `git status` against running agents before believing it.
- Health check recipe: dashboard 302 without Access headers / 200 with; snapshot worker
  `GET /`; `r2data verify` every manifest; worklogs current; no stale open inbox items.
  Access gotcha: service tokens need a policy with action "Service Auth", attached from the
  app's Policies tab.
