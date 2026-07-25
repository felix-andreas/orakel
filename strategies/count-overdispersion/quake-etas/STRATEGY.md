# count-overdispersion/quake-etas

> Thesis (from `ideas/2026-07-25-quake-ladder-overdispersion-3.md` — read it fully):
> weekly USGS seismicity-count ladders are priced against an implicitly Poisson
> distribution. Over 1,385 weeks of catalogue the M5.5+ weekly count has mean 9.458 and
> variance 47.825 — **Fano 5.06**, i.e. 2.25× wider than Poisson. That maps onto the
> traded buckets as a U-shape: both tails ~1.6× too cheap, the middle up to 1.6× too
> rich. **Shape claim** — the crowd's level is calibrated; its allocation is not.
>
> Traded **at window-open only**: mid-week repricing is measured dead (70% of it lands
> within one hour of a quake), so intraday is a speed race we would lose.

## Method

DAY-1 STATE. The simulation is ETAS (Epidemic-Type Aftershock Sequence): a self-exciting
branching point process — background events thinned from a Poisson process, each spawning
offspring with Omori-law timing and Gutenberg-Richter magnitudes — at ~10⁶ simulated
weeks per board, plus a magnitude-revision layer and an integrated parameter posterior.
No closed form exists for a branching process with an Omori kernel; that absence is the
edge.

**Known gate-0 subtlety:** 21/21 M6.5+ boards reproduce exactly, but 5/20 M5.5+ boards
miss by one event — 2.01 events/week sit at exactly M5.5 and magnitudes are revised ±0.1
post-hoc, which moves a whole bucket. The revision layer is not optional.

## Applicability

A market fits when: it is a bucket ladder over a count of clustered events, the resolving
catalogue is public and reproducible, books are live (this family: 0/314 dead legs), and
the fundable legs (≥3c after the 0.05 fee) are where the overdispersion actually bites.

## How to run

(to be written with the first scripts in `src/` — Rust; this is the compute-heavy variant
the family exists for)

## Evidence

- (day-1 results land in `results/`)

## Changelog

- 2026-07-25 — created from the idea; slot 3 trial started.
