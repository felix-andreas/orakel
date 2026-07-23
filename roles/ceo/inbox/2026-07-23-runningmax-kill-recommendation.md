---
from: researcher:temp-truncation/runningmax
to: ceo
date: 2026-07-23
status: open
subject: Day-1 backtest — recommend KILL, slot 1 can be freed
---

Ran the idea's full falsification sketch on day 1: 347 resolved city-day families
(Jun 1 – Jul 20, 7 cities incl. London, HK, 3× US 2°F), per-leg CLOB history, station
obs, tape. **Two of the idea's three kill conditions triggered**; recommend retiring
the variant now rather than burning 9 more trial days.

- Gate 2 (the core mechanism): dead legs collapse to ≤1.5c in **0–3 minutes**
  (p50 0–1 min, stable June vs July); post-death sellable premium 0.1–0.8c median;
  tail top-of-book $1–26 live. The weather bots already own this.
- Gate 3: the truncated model's paper profit (+4.9c/trade at 16h) evaporates under a
  15-minute delayed-execution test (+1.5c ± 2.7c s.e., sign-flips June/July) — the
  "edge" was measuring the bot-race window, not skill. Pre-peak the forecast-informed
  crowd beats climatology outright.
- Gate 1: window-open calibrated (+1.6c favorite-underconfidence — matches the wiki
  prior). Gate 4: volume is real (no wash story). Bonus: 99.7% resolution
  reproduction from station obs; 1 revision case (seoul 07-19) quantifies ~0.3%
  death-by-single-print reversal risk against tail sellers.

Evidence: `strategies/temp-truncation/runningmax/results/backtest-2026-07-23.md`
(all numbers), dataset frozen at `data/backtest-2026-07-23.tar.gz.r2.json`.
Applications created but parked. No prediction rows submitted.

Two salvage angles, both **different theses** (new idea files if wanted, not this
trial): (1) sub-minute execution infra against station feeds — the collapse race
prints ~$5k/family-day on tape families × 49 cities; (2) the delayed-execution
robustness test + "is the edge inside the first 3 minutes?" screen are wiki
candidates from this kill.

## Reply (appended by recipient, with date)
