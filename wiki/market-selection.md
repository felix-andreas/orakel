# Market selection — where is edge findable?

Strategy ideas and applications should target markets where our comparative advantage —
building calibrated models fast, reading fine print carefully, and re-forecasting daily —
can beat the marginal trader. Distilled from poly's scored runs (2026-07).

## Select FOR

1. **Simulation-tractable generating processes** — counting processes (tweet counts),
   brackets/schedules (tournaments), physical time series (weather), mechanical indices
   (CPI rounding). If the resolution variable can be Monte-Carlo'd from public data, we
   can be calibrated where the crowd vibes.
2. **Thin-to-mid liquidity** (~$10k–$1M real volume). Deep books are efficient: poly's
   $40M BTC market converged to our number on its own — confirmation, not edge.
3. **Structural fine print** — rounding buckets, tie rules, revision policies, sibling
   families with coherence constraints. Careful reading beats casual traders. (poly's
   France market resolved on a tie-break every casual trader missed: model the tie-break,
   don't reduce "most goals" to "clear the leader".)
4. **Fast resolution** (days–weeks). Scoring is what compounds; a Dec-31 market teaches
   nothing until winter. For *trials* this is near-mandatory.
5. **Genuine uncertainty** (roughly 15–85% prices) — a 1% market is usually right for
   boring reasons.
6. **Stale or salience-anchored crowds** — sim-tractability alone isn't enough; mature
   recurring series with weekly-scored crowds are efficient at window-open (see
   [recurring-crowd-calibration](reference/recurring-crowd-calibration.md)). Look for
   prices that *haven't repriced* on fresh data, or that ran on narrative (poly's
   hottest-year market climbed 3 straight days on El-Niño headlines relevant to the
   *following* year — the fade was the biggest edge on the book).

## Select AGAINST

- **Speed-race mispricings** — if the inefficiency closes within minutes of a public
  print, bots own it and agent cadence structurally cannot compete. Measure before
  trialing: how fast does the target mispricing actually close on resolved instances?
  (temp-truncation kill, 2026-07-23: dead legs collapsed in 0–3 min; see
  [delayed-execution-test](reference/delayed-execution-test.md).)
- Deep, professionally-arbitraged books (major crypto levels, headline politics).
- Pure insider processes (M&A rumor resolution) — the decisive information is private;
  only worth it with a structural angle (e.g. wash-trade detection changing the read).
- Markets whose resolution source we can't access read-only.
- Calibrated recurring crowds at window-open — run the calibration test before spending
  a slot.
- **Proxy data against a primary-inputs crowd.** Before building any nowcast/model on a
  *proxy* feed, measure the crowd's implied σ from modal calibration on resolved
  instances (one day of work, no model needed). If the crowd's precision is only
  achievable by running the resolving index's own upstream inputs, a proxy-based model
  is dominated *before it is built* — however good the pipeline. (gistemp-era5 kill,
  2026-07-24: crowd σ 0.015 via GHCN-M+ERSST replication vs any ERA5-transfer floor of
  ~0.038.) Ask first: *who is the sharpest agent already in this market, and what data
  are they running?*
- **Glanceable within-window state.** The sibling of the screen above, and it kills
  nowcasts the same way: before modelling how a partly-realised window will finish, ask
  whether an ordinary trader can simply *look* at the state — an app, a live page, a
  daily list. If yes, your model must beat observation, not ignorance, and it loses.
  (Netflix weekly Top-10 probe, 2026-07-25: the whole information set a forecaster can
  use — previous week's global views plus 94-country ranks — publishes *before* the
  market opens, and no official data lands again until the resolving print, which looks
  ideal. But subscribers see the in-app daily Top 10 all week. A decay model fitted on
  264 weeks of Netflix's own TSVs picked the winner 23% / 42% of the time on shows /
  films against the Thursday market's 77% / 83%. Filed nowhere, killed in research.)
  The corollary is where to look instead: markets whose within-window state is **hidden
  from the amateur too** — a count that must be assembled, an estimate with error bars,
  an ordering over many public objects — so the competition is modelling skill rather
  than who bothered to look.

## Before you spend a slot

Run the cheap screens first, in this order — each has killed a real trial in one day:

1. **[Sharp-line screen](reference/sharp-line-screen.md)** — if a bookmaker or exchange
   prices this event, fetch their line. Minutes of work; killed a whole slot on 2026-07-25.
2. **[Phantom-midpoint split](reference/phantom-midpoints.md)** — decompose the claimed
   edge by whether the book actually moves. If it lives in the dead half, there is no edge.
3. **Speed screen** — how fast does the mispricing close? Minutes means bots own it.
4. **[Proxy-vs-primary](reference/published-ci-vs-printed.md)** — can the crowd run the
   resolving source's own inputs? Then a proxy model is dominated before it is built.

- **Objects a professional counterparty already prices or publishes.** The binding
  constraint on the 2026-07-25 simulation cycle was not modelling skill — it was *who else
  is here*. Three of four candidates died to one question: **golf** (DataGolf ships its
  complete live Monte Carlo — win/top5/top10/top20/cut per player — as free JSON **in the
  page source**, so read the source, not the UI), **MLB season ladders** (FanGraphs
  publishes free daily playoff odds from 20,000 sims), **tennis derivative ladders**
  (Polymarket's totals ladder *is* the Pinnacle line to +0.07pp). Run these two checks
  **before** the backtest, cheapest first: (1) does a bookmaker/exchange price this object
  ([sharp-line-screen](reference/sharp-line-screen.md))? (2) does a specialist publish the
  simulation free? A three-hour backtest run before them is three hours of manufactured
  edge. The corollary is the positive selection rule: **simulation edge survives where
  there is no professional counterparty at all** — global seismicity counts, obscure index
  ladders, venue-specific bucket families.

## The test

> Can a well-calibrated model, buildable from public data, know something this market's
> marginal trader doesn't — and will we find out (via resolution) fast enough to learn?
