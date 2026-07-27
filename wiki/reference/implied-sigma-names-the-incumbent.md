# Fit the market's implied σ early — it names an incumbent you haven't found yet

> **In plain English:** before hunting for who else forecasts a thing, ask the market how
> *sure* it is. If the prices imply more confidence than the best model you could possibly
> build from public data, then somebody has already published the answer and the crowd is
> reading it. You have found the incumbent without finding the incumbent.

This is the cheapest screen we own, and it runs **before** the modelling rather than after.

## The measurement

1. Fit a distribution to the market's ladder at a checkpoint (lognormal, normal, whatever the
   family's resolution variable warrants). Read off its **implied σ**.
2. Build the best model the free public panel supports — and score it **in-sample**, on
   purpose. You want an upper bound on what the data can do, not an honest estimate.
3. Compare.

**If implied σ is materially tighter than your in-sample σ, stop.** The crowd knows something
your data does not contain. Prices do not become that confident from vibes; someone is
feeding them.

## Measured (box office weekend ladders, 2026-07-27 — killed)

| | σ |
|---|---:|
| market's implied lognormal σ at Friday noon | **0.120** |
| best model from the entire free panel, scored **in-sample** | **0.171** |

Head-to-head multiclass Brier on 32 resolved holdover ladders: **market 0.487, us 0.701**; we
won 8 of 32. The panel was not thin — 571 daily and 187 weekend charts, 437 film-weekends, six
covariates.

The gap was the tell. Chasing it produced the incumbent: **Shawn Robbins**, formerly chief
analyst at BoxOfficePro, posts a free point forecast for **every holdover weekend, by ordinal,
every Wednesday** — 61 issues since 2025-01-22, all public. His ~10% MAPE *is* the market's
implied σ, near enough exactly.

## Why this screen exists, and why the catalogue scan is not enough

`sharp-line-screen.md` says to check whether a venue prices the object, and Kalshi's
12,187-series catalogue with declared settlement sources is one unauthenticated call. Box
office passed that screen **perfectly**: 0 of 12,187 series, Pinnacle's "Entertainment" sport
returning `matchupCount: 0` and an empty `/leagues`, Smarkets carrying one annual winner
market. Every venue check said the field was clear.

**It wasn't.** The forecast lives in a PNG inside a Substack post. No venue scan, no
`settlement_sources` grep, no odds-API sweep could ever have found it — and the same has now
happened three times in different clothes:

| family | the incumbent | how they publish |
|---|---|---|
| golf | DataGolf | free model page |
| MLB | FanGraphs | free projections |
| **box office** | **a former trade analyst's Substack** | **a PNG in a weekly post** |

A hole in a venue catalogue says **no venue prices this**. It never says **nobody forecasts
this**. Those are different claims, and only the second one is the edge.

Implied σ catches all three, because it does not care *who* the incumbent is or *where* they
publish. It only asks whether the price is more confident than public data can justify.

## Rules

1. **Run it before you build.** It costs one ladder fit and one crude model. It is the last
   screen you want to discover after two days of pipeline work.
2. **Score your comparison model in-sample deliberately.** An honest out-of-sample number
   makes the gap look bigger than it is and you will kill live ideas. You want the most
   generous possible reading of your own data.
3. **A tight implied σ is a search instruction, not just a verdict.** Go and find the
   publisher — knowing they exist tells you roughly what to look for and how good it is.
4. **When implied σ is *wider* than your model supports, that is the interesting case** — it
   is the shape of a real edge, and it is what the rest of the wiki's gates then have to
   survive.

## See also

- [sharp-line-screen](sharp-line-screen.md) — the venue check, and its blind spot
- [recurring-crowd-calibration](recurring-crowd-calibration.md) — is this crowd already
  calibrated?
- [checkpoint-artifact](checkpoint-artifact.md) — the mirror image: when a *null* model beats
  the market, the board is unpriced rather than mispriced
- [break-even-win-rate](break-even-win-rate.md) — what survives once an edge does exist
