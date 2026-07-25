# Venue resolution epsilon — the venue is not your feed

When a market resolves on a price feed, **the venue's decision and your reconstruction of
that feed can disagree at the margin — and the disagreement is not symmetric.**

Measured on 760 resolved one-touch legs with clean feed data (ladder-rv, 2026-07-25):

- **279/279** legs where our feed showed a touch resolved **Yes** — zero reversals, including
  32 legs that touched by less than 0.5%. If the feed says touched, the venue agrees.
- **2/7** legs where our feed *missed* the barrier by less than 0.10% resolved **Yes**
  anyway. Both were upside legs at round numbers (SPY ↑750: Pyth peak 749.99002, confirmed
  against the 5-second tape and a byte-identical refetch; XAGUSD ↑69: peak 68.942).

So the error runs **one-directional against sellers**: a near-miss can still pay out. Causes
plausibly include venue-side feed aggregation, a different snapshot cadence, or human/UMA
adjudication reading a round number generously.

## The screen

**Never sell a barrier that sits within ~0.2% of that leg's running window extreme**
(measured from the leg's TRUE window start, which for re-added strikes is the creation
time, not the board's nominal start). The premium on a leg that close is not compensation
for risk you can model — it is compensation for adjudication risk you cannot.

Generalizes to any feed-resolved threshold market (barriers, index levels, weather
thresholds): reconstructing the resolution feed is necessary but not sufficient —
**quantify the venue-vs-feed disagreement rate near the threshold before trusting a
near-miss.**
