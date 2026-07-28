---
date: 2026-07-28
status: open
from: CEO
decision_needed: only if you want equity boards in the trial at all
blocking: nothing — the trial continues either way
---

# The working window and US equity hours never overlap

Not "rarely". Never, by half an hour, in both DST regimes:

| | working window (§3, Berlin 02:00–15:00) | Pyth equity RTH | overlap |
|---|---|---|---|
| summer | 00:00–13:00 UTC | 13:30–20:00 UTC | **none**, gap 0.5h |
| winter | 01:00–14:00 UTC | 14:30–21:00 UTC | **none**, gap 0.5h |

SPY and NVDA boards resolve on Pyth prices sampled during US regular trading hours. The
daily trigger fires at 01:07 UTC. So **there is no hour of the current cadence at which an
equity board can be legally predicted** — the resolving feed is always shut when we look.

This isn't hypothetical, and it's worse than we thought yesterday. Slot 1 backfilled the
feed state of every row we have ever written, from the frozen candle archive rather than
from memory: **68 of 132 rows (52%) were priced off a shut feed**, and **every equity row
the firm has ever emitted was stale**. Yesterday's write-up said 64 of 95; the correction
is upward, and days 1–2 turn out to have emitted 20 stale equity rows nobody had counted.

Today the stale-feed gate suppressed all 22 equity legs automatically. That is the system
working. But it means the equity applications currently produce nothing, every day, forever.

## What I've already done (no decision needed)

Nothing changes in the trial. The gate handles it: equity legs are suppressed at emission,
the boards stay in the watchlist so their books keep being recorded, and the evidence base
for the 08-02 trial review rests on WTI, gold and silver — which sit in a 22:00Z→21:00Z
Mon–Fri session and *are* open at 01:07 UTC (verified today: 0.1h stale, feed open).

## The decision that is yours

**Only if you want equity in scope at all: the weekday window would have to extend to about
17:00 Berlin** (giving 13:00–15:00Z inside RTH). That is a constitution §3 change, and §3
exists to protect your interactive usage limits — so it costs you something real.

**My recommendation: don't.** Equity was already the weakest family we have. Reachability
by board type was BTC 100%, WTI 99%, gold 82%, **equity weeklies 38%** — so we would be
widening your window to chase the family least likely to be tradeable, on the strength of
zero valid rows. If WTI and the metals earn a promotion on Friday, revisit it then with
evidence instead of hope.

If you agree, no reply is needed — the gate already produces this outcome and I'll record
it as settled at the 08-02 trial review. Reply only if you want the window changed.
