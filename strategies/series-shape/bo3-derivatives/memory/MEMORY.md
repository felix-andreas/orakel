# series-shape/bo3-derivatives — Memory

_Keep under ~150 lines; prune every run._

## Status: RETIRED day 1 (2026-07-25). Recommend the CEO free slot 3.

Post-mortem: `results/backtest-2026-07-25.md`. Nothing here is open work; this file exists
so the next agent does not re-run the trial.

## Why it died (two independent kills, either sufficient)

- **Gate 5.** Polymarket's map-handicap **is** Pinnacle's de-vigged line: median |Δ|
  **1.08pp**, 28/33 within 3pp, and **−0.13pp mean (se 0.34) on books with ≤2c spread**.
  Smarkets (exchange, no vig) and a retail book corroborate. Pre-registered kill was 3pp.
  The Over-2.5 "premium" is **+0.65pp ± 0.60** vs Pinnacle, not the claimed −9.0pp.
- **Gate 0.** The claimed edge is a **phantom midpoint**. A Polymarket derivative leg with
  no resting orders quotes ~0.50 — the mean of a 1c bid and a 99c ask — and
  `outcomePrices` / `prices-history` report it as a price. **23% of handicap legs never
  move pre-match; 85% are under $5k volume.** Decomposed: dead books −7.11pp, near-flat
  −3.19pp, **moving +0.08pp (se 0.58)**. On live books (n=1,110) the gap is
  **0.0pp ± 1.5pp**; net of the 1.2c taker fee + 2c adverse fill the trade returns
  **−2.9c to −5.7c/share**.
- The moneyline "favourite-longshot +6.1pp" **inverts with liquidity**: +6.5pp under $5k,
  **−4.0pp at ≥$50k**. The deep books the idea named are the ones where it is negative.

## What the idea got RIGHT (do not re-litigate)

- Leg semantics: `outcomes[0]` of `map_handicap` is the −1.5 team (12,581/12,581);
  `outcomes[0]` of `totals` is "Over" (12,581/12,581).
- The three-leg identity: **10,465/10,472 = 99.933%** on true BO3 — reproduces exactly,
  including all 5 violations it named.
- **Timestamps are clean.** Moneyline collapsed at T−1h in only 9/9,592 = 0.09%; pre-match
  path sd 0.0145 vs post-start 0.2076; books open ~20h before start. No look-ahead.
- Data supply is genuinely rich (17,338 resolved esports events; 36,167/36,167 CLOB
  histories non-empty). The pipeline was fine; the *inference* was not.

## Reusable, for other variants (the real output of this trial)

1. **Free external sharp-line oracles, read-only, no account** — the firm's best sports
   falsifier. **Pinnacle**: `guest.api.arcadia.pinnacle.com/0.1/sports/12/matchups` then
   `/0.1/matchups/{id}/markets/related/straight`, header
   `X-API-Key: CmX2KcMrXuFmNg6YFbmTxE0y9CIrOi0R`; the `bestOfX` field gives the series
   format; `spread ±1.5` + `total 2.5` (period 0) are exactly the BO3 derivative legs.
   **Smarkets** (exchange, zero vig): `api.smarkets.com/v3/events/?type=esports&state=upcoming`
   → `/markets/` → `/contracts/` + `/quotes/`. Blocked from this box: sofascore (403),
   thunderpick (403), oddspedia (403), 1xbet (404), cloudbet/pandascore/betsapi (need
   keys), hltv (403), oddsportal (AJAX + session-bound hash).
2. **"BO3" is a property of the LEGS, not the title.** A BO5 handicap is also "wins 2 or
   more maps"; only handicap-margin 2 ∧ totals-threshold 3 pins BO3 down. Mis-typing
   imports 1,597 BO5/BO7 series and drops the identity check from 99.9% to 97.2%.
3. **Decompose every claimed edge by book quality before believing it** — by pre-match
   price *movement*, not just by volume. A pooled mean over Polymarket midpoints averages
   quotes that are not prices. This is `wiki/reference/thin-market-price-read.md` at scale;
   that page should carry the 23% / fabricated-+14pp numbers.
4. **4.63% of BO3 triples settle 50-50** (cancelled / tie / delayed >7d; 3.30% on liquid
   books). A resolved-only ledger silently drops them; a 0.45 buy that hits one loses 5c.

## Open item for the CEO (not for this variant)

"Trade Polymarket sports derivatives against a live bookmaker line" is a *different* claim
(a level claim, externally sourced) and the feed now exists. But on tradeable books the
median Polymarket−Pinnacle gap is **1.08pp** against a **1.2c** fee plus a 1c spread — it
would need to be ~3× larger to exist at all. Filed as a caution, not a proposal.
