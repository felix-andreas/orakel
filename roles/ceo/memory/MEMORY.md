# CEO Memory

_Keep under ~150 lines. Prune every run. Details go to worklogs / run manifests / wiki._

## Short-term (current run / immediate)

- **2026-07-26 run** (ops/runs/2026-07-26.toml). Slots: 1 = `barrier-touch/ladder-rv`
  (day 4, **13 rows appended**, ledger 108→121), 2 = **turned over twice in one morning** —
  `arena-rank/favourite-shrinkage` parked, `tomatometer/arrival-drift` promoted into the
  same slot within 3 hours and **retired on day 1 by gate 0**. slots_active back to 1.
- **THE BIG CORRECTION: our 2-of-21 reachability headline was about equity weeklies, not
  about the variant.** Slot 1 replayed the tape across all 70 predicted markets:
  BTC 100%, WTI 99%, silver 89%, gold 82%, **SPY/NVDA weeklies 38%**. Always split
  reachability by BOARD FAMILY before concluding anything about a variant. Gold is the
  warning case — best Brier edge, thinnest tape, 0/11 markets ever showed a bid at our mid.
- **New status `parked`** (trial | live | parked | retired). `retired` = a gate killed the
  thesis; `parked` = the thesis HELD and has no expression. Releases the slot, keeps the
  clock and evidence, carries `reopen_when` naming an observable condition. Dashboard had
  to be taught it — the stat strip read 2+0+5 against a total of 8 and the parked variant
  was invisible. **Any new status is a dashboard change; check the arithmetic reconciles.**
- **I promoted an idea on its DESCRIPTION of gate 0, not a measurement, and it was dead at
  filing time.** Kalshi is the primary venue for film-score ladders (1c spread vs our 18c,
  $7.19M vs $41k on the same film) with an unbiased line. Playbook now requires a named
  incumbent to be MEASURED before filing; unmeasurable → file `needs-gate-0`, not `backlog`.
  **Never spend a slot on an unmeasured incumbent.**
- **Sub-agents can outlive their parent.** Slot 2's Wayback harvester reported 90 min after
  the run closed, into an already-retired folder. Used it to close the audit the kill left
  open: the founding drift claim REPLICATED at 8x sample (-4.29 vs -4.14) but its explanation
  did NOT (thin-denominator ratio 1.26x, not 7.6x). **A correct observation with a wrong
  explanation that the market already prices is the failure mode to expect from here on** —
  our screens are good enough that being right is no longer sufficient. Always check for
  orphaned output before treating a variant folder as finished.
- **Kalshi publishes free hourly bid/ask HISTORY (`candlesticks`)** — the historical order
  book we lack. Our fillcheck reachability is a lower bound only because a resting bid nobody
  hit leaves no trace in a trade feed; a real quote history replaces the bound with a
  measurement. TOP wiki item next run.
- **Backlog was EMPTY this morning** (7 ideas in 4 days, 5 killed day 1, 0 available) — that
  is why slots 3–5 idle, not capacity. Ran a second market researcher; it filed the
  front-line first-passage idea. **Escalated to Felix: war-market domain ruling**
  (`roles/felix/inbox/2026-07-26-war-markets-scope.md`) — awaiting his yes/no before a slot.
- **Next run checklist:** (1) Felix's war-market ruling → promote or mark `discarded-scope`;
  (2) slot 1 day 5: the leg-sum/null-model re-check is STILL outstanding (2 runs now), plus
  `cmd_live` doesn't diffuse spot to a future window open (5.9% off for August), plus the
  skipped R2 archive freeze — don't let it skip twice; (3) 07-31: WTI+gold+silver July
  boards resolve → ~51 rows scored, the trial's real evidence; (4) archive WTIX6 when it
  lists (~Aug 20) — WTIQ6 is already delisted; (5) ~08-10: check whether the August arena
  boards priced, to reopen favourite-shrinkage.

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
