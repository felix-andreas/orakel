# series-shape/bo3-derivatives

> **RETIRED on day 1 (2026-07-25). The thesis is false.** Full post-mortem:
> [`results/backtest-2026-07-25.md`](results/backtest-2026-07-25.md).
>
> Original thesis (from `ideas/2026-07-25-esports-series-shape-2.md`): take the deep
> esports moneyline as the level; trade the thin map-handicap and totals legs whose
> implied series-score distribution is mis-allocated — moneyline favourite-longshot
> (+6.1pp), a convex transfer amplifying it 2.5–3.3× on the sweep leg, and an independent
> Over-2.5 premium (8/8 months, t=−5.16). **Shape claim.**
>
> **What actually happened.** Gate 5 (run first, as the cheapest kill) obtained the
> external line the idea's author could not: Polymarket's map-handicap tracks **Pinnacle**
> to a median **1.08pp** (28/33 within 3pp; **−0.13pp mean on books with ≤2c spread**),
> corroborated by the **Smarkets** exchange and a retail book. The pre-registered kill was
> "agrees within 3pp". Gate 0 then found the mechanism of the false edge: **a Polymarket
> derivative leg with no resting orders quotes a ~0.50 midpoint — the mean of a 1c bid and
> a 99c ask — and 23% of these legs never move pre-match, 85% have under $5k of volume.**
> Restricted to books with a real price, the market→realised gap is **0.0pp ± 1.5pp**
> (n=1,110); net of the 1.2c taker fee and a 2c adverse fill the trade returns
> **−2.9c to −5.7c/share**.

## Method (as built, day 1)

Gate-first. No forecasting model was ever needed — the gates answered the question.

1. **`src/gate5.py` — the external line.** Pinnacle guest arcadia
   (`/0.1/sports/12/matchups` → `/0.1/matchups/{id}/markets/related/straight`, public
   guest key, no account) serves `spread ±1.5` and `total 2.5` on `bestOfX: 3` matchups —
   *exactly* our two legs. Smarkets v3 (`/events/?type=esports&state=upcoming` →
   `/markets/` → `/quotes/`) serves an exchange back/lay mid, i.e. no vig at all. De-vig
   by normalisation **and** by a power fit; the conclusion must not depend on which.
2. **`src/compare.py`** matches Polymarket events to bookmaker matchups on start time +
   team-name tokens, **re-orients the handicap onto Polymarket's `outcomes[0]`** before
   differencing (an orientation error here would fake the entire result, so both sides are
   printed), and reports Δ sliced by book quality.
3. **`src/harvest.py`** — independent Gamma harvest: date-windowed offset paging
   (`closed=true&tag_id=64`, weekly `end_date_min`/`end_date_max` windows × offset, since
   plain offset caps at 2000) → 17,338 resolved esports events; leg typing from
   `sportsMarketType` with every `description` retained verbatim; CLOB `prices-history`
   per token anchored on `gameStartTime − 36h` at `fidelity=10` (36,167/36,167 non-empty).
4. **`src/gate0.py`** — price-free artifact checks: description-derived semantics, the
   three-leg identity, survivorship, supply.
5. **`src/backtest.py`** — checkpoint extraction (T−24h/−6h/−1h/−15m; *last observation at
   or before* t, never interpolated forward) and the decomposition of every headline
   number **by book liveness and by leg volume**. That decomposition is the whole result.

**Two method facts worth carrying forward.**
*"BO3" is a property of the legs, not the title*: a BO5 handicap is also "wins 2 or more
maps", so the format is pinned only by handicap-margin 2 ∧ totals-threshold 3. Getting
this wrong imports 1,597 BO5/BO7 series and collapses the identity check from 99.9% to
97.2%. And: *decompose every claimed edge by book quality before believing it* — a pooled
mean over Polymarket midpoints is a mean over quotes, and some of those quotes are not
prices.

## Applicability

None. `applications/` holds one `active = false` entry recording the most liquid board the
variant would have traded, and the measured reason it does not.

## Evidence

- [`results/backtest-2026-07-25.md`](results/backtest-2026-07-25.md) — day-1 kill, all six
  gates with numbers.
- [`data/gate5-pinnacle-2026-07-25.csv`](data/gate5-pinnacle-2026-07-25.csv) — the 34
  matched Polymarket-vs-Pinnacle pairs, in git because it is the load-bearing table.
- `data/*.r2.json` — frozen: 17,338 events, 12,581 triples, 10,471 checkpoint records,
  36,167 CLOB token histories, the raw Pinnacle snapshot.

## Changelog

- 2026-07-25 — created from the idea; slot 3 trial started; **killed the same day** on
  gate 5 (external line, three independent books) and gate 0 (phantom midpoints on dead
  books). Recommend `status = "retired"`, slot 3 freed.
