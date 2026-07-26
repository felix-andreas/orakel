# tomatometer/arrival-drift — memory

**Status: KILLED 2026-07-26, day 1. Zero prediction rows.** Read
[`../results/gates-2026-07-26.md`](../results/gates-2026-07-26.md) before reviving anything
here; this file is the operational residue.

## The three numbers that ended it

1. **Kalshi implied-median error vs realised settlement, n = 19 boards:** +0.134 (T−96h),
   +0.184 (T−72h), +0.218, +0.338, +0.643 (T−6h); se 0.12–0.81; 9 down / 10 up at T−96h.
   In-band leg bias never exceeds 1.25 se. Kalshi already contains the drift.
2. **Polymarket `price − realised` on locally-live in-band legs, 68 ladder boards:**
   +0.010 (t = +0.23) at T−96h → −0.171 (t = −3.34) at T−6h. **Wrong sign for the thesis**,
   and concentrated entirely in legs priced ≥ 0.50 (−0.105 t = −2.09 at T−72h; −0.262
   t = −7.77 at T−24h). It is favourite-longshot, not a shifted centre.
3. **Break-even on the drift trade at T−72h:** buy the cheap NO, `q* = 0.192`, `q = 0.033`
   (1 of 30), `q⁻ = 0.006`. Every band refused at every checkpoint.

## Reusable API facts

- **Kalshi historical book, free and unauthenticated:**
  `GET https://api.elections.kalshi.com/trade-api/v2/series/{series}/markets/{ticker}/candlesticks?start_ts=&end_ts=&period_interval=60`
  → hourly `yes_bid` / `yes_ask` open-high-low-close **in dollars**, plus `volume_fp` and
  `open_interest_fp`, for the market's whole life. Strictly better than Polymarket's
  midpoint-only `prices-history`. **`start_ts`/`end_ts` are epoch seconds; an out-of-range
  window returns `{"candlesticks":[]}` with HTTP 200** — that reads as "no data" and cost me
  twenty minutes. Compute the epoch, do not eyeball it.
- **Kalshi discovery:** `/series?category=Entertainment` → 2,524 series, of which **233**
  match `KXRT*`/`RT*`. Most are legacy shells with **no markets left**; the live family is
  the umbrella series **`KXRT`** with 64 events (`KXRT-SPI`, `KXRT-ODY`, …). Path is
  `/events?series_ticker=` → `/markets?event_ticker=`. `/markets?series_ticker=` returns
  `[]` for the per-film legacy series. Rate limit is real — 0.3s spacing, back off on
  `too_many_requests`.
- **Kalshi tape:** `/markets/trades?ticker=&limit=&cursor=` works and paginates;
  `min_ts`/`max_ts` did not filter for me.
- **Kalshi market fields carry the settlement source**: `settlement_sources[].url` on the
  *event* gives the exact `rottentomatoes.com/m/<slug>` — a free film → RT-slug map. The
  *series*-level source is generic; read it off the event.
- **Rotten Tomatoes live state:** `curl` with a browser UA against
  `https://www.rottentomatoes.com/m/<slug>`, then grep `"criticsScore":{...}` for
  `likedCount` / `notLikedCount` / `ratingCount` / `reviewCount`. Works today.
- **Polymarket:** `endDate` is **not** the resolution instant in this family; the real
  10:00 ET instant is only in the leg `description`. `liquidityNum` is null on 485 of 556
  modern legs.

## Semantics you must not get wrong

- Polymarket **"at least X" = `score ≥ X`**. Kalshi **"Above X" = `score ≥ X+1`**. So
  Polymarket `X+` ≡ Kalshi `Above (X−1)`. Verified on rules text both sides and on The
  Odyssey's settlement (Kalshi `Above 94` YES / `Above 95` NO; Polymarket `95+` YES /
  `96+` NO; both ⇒ 95).
- The Tomatometer is `round(100 · liked / (liked + notLiked))` — plain half-up rounding on
  the integer lattice. A `95+` leg at `n = 350` is `liked ≥ 331`.
- Both venues settle at **10:00 AM ET Monday after wide release** (14:00Z EDT / 15:00Z EST),
  but **not necessarily on the same Monday**: `The Invite` settled 2026-06-29 on Polymarket
  and 2026-07-13 on Kalshi. Match on the instant, never on the film.

## Traps found

- **`how-to-make-a-killing-rotten-tomatoes-score`**: `≥56` resolved NO while `≥57` resolved
  YES, on a $646k board with $190k of notional on the broken leg. Almost certainly a
  mid-life re-label. Exclude it from any study; treat it as live counterparty risk.
- **Family-level phantom pass rates lie.** The family scores 0.6% dead legs; the live
  Spider-Man board is 25% phantom (`90+` at 0.650/0.830, $219/$54 depth). Gate every leg.
- **T−6h is dirtier than T−24h** (mono violations 3% → 10%, implied mass 1.003 → 1.027) —
  deep-OTM legs go one-sided at the end. T−24h is the cleanest checkpoint here.
- **Leg-sum is not a vig test on a cumulative ladder** (it is a survival function). Use the
  implied bucket mass from adjacent-strike differences. On the 40 *bucket*-era boards the
  raw leg-sum **is** the right test.

## The lead I did not take

The favourite-longshot result in gate 3 is an independent replication of
`arena-rank/favourite-shrinkage` in a family with no shared crowd, mechanism or resolution
source. At T−72h the 0.70–0.90 band gives `q* = 0.8225`, `q = 0.9667` (29/30),
`q⁻ = 0.8333` — **clears**. n = 30 on one band. That belongs to `favourite-shrinkage`, and
is worth handing over rather than re-deriving.

## Open question if anyone revives this

The historical **score paths** (Wayback reconstruction of `likedCount`/`notLikedCount` per
capture) were still being harvested when the gates closed the day, so the drift's *magnitude*
was never re-measured out of sample on more than the idea's own 14 films. It does not change
the verdict — gates 0, 3 and 5 are each sufficient and none of them depends on it — but it is
the one number in the founding idea that remains unaudited. If you revive this, measure that
first, and note the idea's own warning that Wayback capture density is biased toward big
releases.
