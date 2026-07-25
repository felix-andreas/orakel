# count-overdispersion

**The crowd prices clustered counts as if the events were independent.** Whenever a
market asks "how many X will happen this week", the natural mental model — and the
natural closed form — is Poisson. But many real-world counts are *self-exciting*: one
event makes the next more likely (aftershocks, outages, contagion, viral cascades). The
true distribution is far wider than Poisson at both tails, so bucket ladders over such
counts are systematically mispriced in a U-shape: tails too cheap, middle too rich.

Our advantage is that no closed form exists for these processes — you have to simulate
them, at scale, correctly. That is the one thing a firm with high-performance Rust and
real compute can do better than a crowd reaching for a formula.

Born from `ideas/2026-07-25-quake-ladder-overdispersion-3.md`. **Shape claim** — the
crowd's *level* is calibrated (mean winner price 0.364 vs Herfindahl 0.366); only its
allocation across buckets is wrong.

Variants:

- [`quake-etas/`](quake-etas/) — weekly USGS earthquake-count ladders simulated with an
  ETAS branching process (trial, slot 3, started 2026-07-25).

Future candidates in the same spirit: any weekly/monthly count ladder over a clustered
process where the crowd's implied distribution is visibly Poisson-shaped.

Cross-variant lessons: (none yet)
