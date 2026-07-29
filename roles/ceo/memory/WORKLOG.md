# CEO Worklog

One dated entry per run. Name the exact model id that did the work.

---
## 2026-07-27 — the headline flipped, and then got explained

Model: claude-opus-5. Manifest: `ops/runs/2026-07-27.toml`. 2 subagents, 540k tokens.

WTI touched $85. We had called no-touch four mornings running while the market walked to 0.715.
ladder-rv's mean paired improvement went +0.000945 → **−0.0172**, and the reachability split
made it worse rather than better: the mid-board legs we lost on are the ones we could actually
have traded, while the wings we "beat" remain untradeable.

Then slot 1 explained it, and the explanation is the most useful thing the variant has produced.
The 07-25 and 07-26 runs read the **same spot, sigma and remaining sessions** — the WTI feed is
shut from Friday 20:59Z to Sunday 22:00Z, and the book repriced during exactly that window. Our
only movement was a vol lookback sliding across closed days: arithmetic wearing the costume of a
view. **64 of 95 outstanding rows were priced the same way.** Adopted the stale-feed gate with
suppression and changed the cadence — weekend runs now emit nothing on WTI, gold or silver. Five
honest days beat seven with two of them quoting Friday.

The null/leg-sum check finally ran and **cleared**, which is the reason today is a setback rather
than a collapse: the evidence base is not a checkpoint artifact. It cost one claim — gold's
window-open margin does not survive a leg-sum gate — and I have withdrawn the pooled window-open
number I had been repeating in my own summaries.

Also added a per-market level to scoring, because four rows on one barrier touch are one event
and the ledger was counting them as four. Per market −0.0051, per row −0.0173. Both negative;
only the size was a counting artifact, and it would have deflated the good headline too.

Market researcher filed nothing and was right to: box office passed every venue check perfectly
and died to the market's own implied sigma, which named an incumbent no catalogue scan could
reach — a man with a free Substack. Two wiki pages out of the day, `stale-feed-gate` and
`implied-sigma-names-the-incumbent`.


## 2026-07-26 — slot 2 turned over twice; both of my errors are in the manifest

Model: claude-opus-5. Run manifest: `ops/runs/2026-07-26.toml`. 5 subagents, 1.22M tokens —
the highest-spend run so far, and the extra researcher was unplanned.

**Before spawning anyone**, found `arena-rank/satellites` (retired yesterday) still owned 7
ACTIVE applications while its successor owned none. Migrated them. Caught only because the
playbook forces the watchlist to be built from active applications, which made me enumerate.
Rebuilt the watchlist as active applications UNION every market with an unresolved
prediction — the union matters, because an applications-only build drops the July commodity
boards, which are the 51 rows that score on 07-31.

**Slot 2, twice.** `favourite-shrinkage` passed its pre-registered band test decisively and
proposed zero rows, so I introduced `parked` and released the slot — the thesis held, there
was just no board. Then I promoted the Tomatometer idea into the same slot within three
hours, and its day-1 researcher killed it on gate 0. Kalshi is the primary venue for that
object with an unbiased line; the idea was dead when filed, and I had promoted it on its own
*description* of the screen rather than a measurement. Playbook fixed.

**Slot 1** appended 13 rows and revised our own headline: the 2-of-21 reachability result was
equity weeklies and sub-3c wings, not the variant. Split by board family it is BTC 100%, WTI
99%, silver 89%, gold 82%, weeklies 38%.

**Two mistakes of mine**, both in `[failures].mine` rather than buried: a resolution sweep
structurally incapable of finding anything (Gamma's `closed` is a filter, not an
include-flag), and the unmeasured-incumbent promotion. A third, smaller: introducing `parked`
broke the strategies page arithmetic until I taught the dashboard about it.

Three wiki pages written (break-even-win-rate, sharpen-only-what-persists, tape-gate) and two
API traps recorded. One escalation to Felix on whether war markets are in scope.


## 2026-07-23 — first CEO run (promotion day)

Model: fable (high). Felix promoted this session to CEO. Dashboard round 2 shipped
(flat layout, mobile burger, favicon, /dev charts) and deployed; self-bind daily
trigger armed (01:07 UTC). Market researcher (fable/high) scanned 26.5k markets,
filed temp-daily-max-truncation-lag; slot 1 trialed temp-truncation/runningmax;
researcher (fable/high) ran the full falsification suite on 347 resolved families
day 1 and recommended KILL (dead legs collapse 0-3 min = bot speed race; delayed-
execution test erased model edge). Kill accepted, variant retired, slot freed,
delayed-execution test + speed-race screen graduated to wiki. 0 prediction rows
(correctly withheld). Manifest: ops/runs/2026-07-23.toml.

---
## 2026-07-23 — second CEO run (Felix-requested)

Model: fable (high). Market researcher run 2 filed hit-price-ladder-rv (one-touch
ladder relative value; speed-screened by construction). Slot 1 refilled with
barrier-touch/ladder-rv; day-1 researcher ran all gates on 13 resolved boards:
ALIVE sell-side (+10.0c/trade delayed t+24h, se 1.6; buys -7.3c -> disabled;
gate-0 251/255 with Pyth-feed-deletion trap found and contained). Appended the
firm's FIRST 18 prediction rows (WTI ladder full, SPY/NVDA wing sells; trial
status); mirrored 18-market watchlist to R2 (verified readback) — snapshot worker
collecting from next :07. 3 rows resolve Friday -> Saturday run scores them.
Manifest: ops/runs/2026-07-23-2.toml.

---
## 2026-07-24 — daily run (first automatic firing)

Model: fable (high). Market researcher answered Felix's market-specific directive
with gistemp-monthly-nowcast; slot 2 trialed climate-nowcast/gistemp-era5 — killed
same day by its own 5-gate backtest (28 instances; crowd replicates GISTEMP primary
inputs at sigma 0.015, our proxy floor 0.038; model built BETTER than promised and
still lost). Kill accepted; first-print-vintages + proxy-vs-primary screens
graduated to wiki. ladder-rv day 2 (opus/xhigh): 39 rows appended (57 total),
watchlist 40 mkts mirrored, buys stay disabled, metals prediction-only, 5 tier-B
sell signals. 20 rows resolve 20:00Z today -> tomorrow scores the first track
record. Manifest: ops/runs/2026-07-24.toml.

---
## 2026-07-25 — daily run (first scoring)

Model: opus-5. FIRST TRACK RECORD: 18 markets resolved, 21 rows scored, 21/21 beat the
market (mean paired improvement +0.0009; our Brier 0.0015 vs market 0.0024) — small
absolute effect on deep OTM wings, thesis directionally confirmed. ladder-rv day 3
(opus-5 xhigh): 51 rows, resolution verification 20/20 vs venue, venue-epsilon
asymmetry discovered -> wiki + method screen, metals backtest (gold earned, silver
denied), weekly board family found, empty-book boards correctly skipped. Market
researcher (opus-5 max) filed arena-rank-satellites; slot 2 filled, day-1 research in
flight at close. Manifest: ops/runs/2026-07-25.toml.

---
---
## 2026-07-28 — daily run (infrastructure, and two corrections to myself)

Model: claude-opus-5. Nothing resolved (0 new; the two closed rows were already
scored). Slot 1 day 6: 14 rows appended under the gate, 79 of 93 legs suppressed
(ledger 146). Market researcher killed mention markets — the first family to clear
every positive screen at once, dead because the crowd's +5pp bias IS the spread
(both trade directions lose at executable prices simultaneously). It passes the tape
gate better than anything we have screened, which retires the idea that liquidity is
our binding constraint.

Three things I had asserted turned out to be wrong, and all three were found by
building the measurement rather than arguing about it. (1) The stale-feed damage is
68 of 132 rows, not 64 of 95, and every equity row the firm has ever emitted was
priced off a shut feed — which is structural: the working window and Pyth equity RTH
have ZERO overlap, missing by half an hour in both DST regimes. (2) "Idea supply is
the binding constraint on slots", carried in memory for three runs, is wrong by ~4x;
what is scarce is objects arriving with a live tradeable board. (3) The hand-built
watchlist had been under-covering every applied-for board for six days — 44 legs
never snapshotted — which I only saw once tools/watchlist made the rule executable.

Also: ledger widened with pricer_version / feed_age_h / feed_open and scoring gained
a pricer aggregate level, so Friday's split is a table row. Dashboard: two
hypotheses for Felix's missing-content banner tried and both disproved (reverted);
what survives is a subrequest budget only a cold cache can exhaust, and the one
change worth keeping was making failures record WHY. Manifest: ops/runs/2026-07-28.toml.

---
## 2026-07-29 — daily run (the measurements went against the claimants, me twice)

Model: claude-opus-5. WTI dipped to $80; headline −0.0172/25 rows → −0.0452/32
(−0.0133 over 20 markets). Slot 1 day 7: 12 rows (ledger 158). Market researcher
killed post-count ladders on a screen we had no name for — leg-level depth, a
$1.5M board with a median $7 resting at the ask on exactly the legs the edge
lives on.

I was wrong twice and both were caught by measurement rather than argument. I
proposed the variant is structurally short downside touch; on 633 legs it beats
the market on touched legs and down-trending legs are its BEST bucket. The real
problem is a tail — the 8 worst legs of 633 are all dip-to, and our two losses
are nested on one contract, so ~1 draw not 2. That makes 08-02 a sizing question,
not a Brier one. And the watchlist tool I shipped yesterday was adding
already-resolved legs, because `closed=false` on /events filters events, not the
markets inside them; yesterday's "44 legs" claim is annotated as wrong.

Slot 1 flagged a one-row discrepancy rather than trusting its own count, and it
was right: a resolution had been silently dropped since 07-27 by an unquoted
comma in its note. A resolution is a join key, so it removed every row on that
market — and the warning printed on every run while I grepped past it. Malformed
resolutions now fail hard.

Three calls made blind before Friday: the pricer split is inconclusive (n≥30
clears in rows, not in markets, and markets is the unit we named in advance);
RV/IV anchors where we actually trade rather than at the prereg's 12:00Z; and a
completeness gate on the 08-02 review. Also a second wall in the funnel — objects
12 and 13 died to execution rather than an incumbent, and both leave only a
maker-side construction. Manifest: ops/runs/2026-07-29.toml.

