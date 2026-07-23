# temp-truncation/runningmax — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-23 — day 1 (trial day 1/10) — model: fable (high), id claude-fable-5

Backtest-first per the idea's falsification sketch. Built the full pipeline as a
cargo crate (`src/main.rs`: discover/obs/prices/tape/analyze/wash/live) and ran it on
347 resolved city-day families (2026-06-01→07-20; london, paris, seoul, hong-kong,
los-angeles, nyc, chicago), with IEM ASOS + HKO obs, fid-10 mids for all 3,817 legs,
fid-1 for a 60-family subset, and trades tape for 8 families.

Results (`results/backtest-2026-07-23.md`):
- Gate 0 resolution reproduction: 326/327 (99.7%) — fine print confirmed (HKO floor
  buckets; US T-group °F; the 1 miss is a revision case: seoul 07-19).
- Gate 1 window-open calibration: winner-open 0.174 vs Herfindahl 0.158 — crowd
  ~calibrated, +1.6c favorite-underconfidence, not tradeable. As the idea predicted.
- Gate 2 dead-leg lag: **KILL** — collapse p50 0–1 min, p95 3 min (stable Jun vs
  Jul); post-death 2h mid p50 0.001, p95 0.008; live tail top-of-book $1–26.
- Gate 3 truncated model vs market: **KILL** — LL worse everywhere; instant-exec
  trading sim +4.9c/trade at 16h collapses to +1.5c±2.7 with 15-min delayed
  execution and sign-flips June/July. Edge = the bot-race window, not model skill.
- Gate 4 wash: volume is real (top-1 wallet ≤7%, self-cross ≤9%).

Actions: KILL recommended to CEO (escalate flag in day-1 report); applications
london/nyc/los-angeles created but parked `active=false`; dataset frozen to R2
(`data/backtest-2026-07-23.tar.gz.r2.json`, 13.9 MB, verified); no prediction rows
submitted (gates dead — per mission condition). Honest kill on day 1 frees the slot.
