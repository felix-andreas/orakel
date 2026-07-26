# tomatometer/arrival-drift — worklog

## 2026-07-26 — day 1. Model: `claude-opus-5` (effort max)

Slot 2, opened the same day `ideas/2026-07-26-tomatometer-review-arrival.md` was filed.
Outcome: **kill recommended, zero prediction rows**. Full evidence:
[`../results/gates-2026-07-26.md`](../results/gates-2026-07-26.md).

**Ran the gates in the charter's order. Gate 0 ended the day; gates 3 and 5 confirmed it
independently.**

- **Gate 0 — Kalshi.** Harvested the whole `KXRT`/`RT` family: 233 series → 203 events → 35
  with live markets → **19 resolved boards** (2026-05-25 → 2026-07-20) with full **hourly
  bid/ask** candlesticks, plus 16 open boards to 2026-12-21. Kalshi is the primary venue
  here by 20–100× ($7.19M on The Odyssey vs $41k on Polymarket), on a 10–29 rung ladder at
  a 1c median spread. Its implied score is **unbiased for realised settlement at every
  checkpoint from T−96h** (implied-median error +0.134 → +0.643, se 0.12–0.81, 9 down/10 up
  at T−96h; in-band leg bias |t| ≤ 1.25 throughout). Reported the counter-reading as
  instructed: this is a Kalshi-vs-truth claim, not a Kalshi-vs-Polymarket agreement claim,
  so it does not rest on treating Kalshi as sharp — but 19 boards would not rule out a
  half-point bias, only the ~2-point one the thesis needs.
- **Gate 1 — checkpoint.** T−96h is the earliest defensible checkpoint (48/68 ladder boards
  fully quoted, 4% with a monotonicity violation, mean implied mass 1.0045) against 11/68
  and 27% at T−14d. Corrected the idea's reading: the T−14d failure is **listing**, not
  mispricing — median first quote is 6.6 days before resolution. Also found **T−24h is
  cleaner than T−6h**, which reverses the natural assumption.
- **Gate 2 — per-leg books.** Live Spider-Man board: 3 of 4 legs tight (0.8–1.0c, $2.1k–4.5k
  depth), `90+` a phantom at 0.650/0.830 with $219/$54. A naive ladder read manufactures
  `P(90 ≤ score < 95) = 0.615` out of that one leg. Every gate-3 number below is computed on
  locally-live legs only; the headline **strengthens** under the gate.
- **Gate 3 — the level claim, falsified in direction.** 68 resolved ladder boards, per-leg
  `resolved_yes` ground truth (not a resolution bracket). The thesis predicts uniformly
  positive `price − realised`; measured **+0.010 (t = +0.23) at T−96h → −0.171 (t = −3.34)
  at T−6h**, concentrated entirely in legs priced ≥ 0.50. That is favourite-longshot bias,
  an independent replication of `arena-rank/favourite-shrinkage`, pointing the **opposite**
  way to this variant's trade.
- **Gate 4 — the simulator.** Built the Rust crate anyway (`src/main.rs`, 800 lines, builds
  clean). It computes the terminal score distribution **exactly** — a 25×25 normal
  quadrature times an exact binomial pmf over the integer lattice — rather than by Monte
  Carlo, because the thesis is a lattice claim and MC noise at 1e-3 is the size of the
  effect; `simcheck` verifies the fast path against a sampler. Three modes (`frozen` /
  `null` / `full`) keep the level and shape claims separable as `strategy.toml` requires.
  **I did not fit it to a conclusion and I am not pretending otherwise** — the gates had
  already answered, and spending the slot fitting a model whose level claim is falsified in
  direction, against a venue that is unbiased at 1c spreads, would have been theatre.
- **Gate 5 — fundability.** Break-even table per band with **Wilson** lower bounds: the
  drift trade is refused in every band at every checkpoint, and in its natural form (buy the
  cheap NO on a leg the score should fall through) needs `q* = 0.192` and won **1 of 30**.
  Realised taker flow: board totals reproduce the idea exactly ($54,539 on `in-the-grey`)
  but the final-72h in-band split does **not** — measured $8,952 against the idea's $48,846,
  stable across three different anchorings. Median in-band flow on a **single leg** over the
  final 72 hours across the 26 largest boards is **$238**; only 30% of legs see $1,000.

**Zero prediction rows, and the reason is structural rather than modelling.** The open
Polymarket surface today is two boards, and both report `"reviewCount": 0` — the review
embargo has not lifted on Spider-Man (resolves 2026-08-03) or Paw Patrol (2026-08-17). The
variant conditions on `(likedCount, notLikedCount)`; that state does not exist yet. Being in
place before the embargo lifted was the point of taking the slot today; the gates simply
resolved faster than the embargo.

**Data frozen to R2 before committing the manifests** (per `AGENTS.md`):
`data/kalshi-rt-2026-07-26.tar.gz.r2.json` (1.66 MB — 19 boards × hourly bid/ask, series and
event metadata, all derived tables) and `data/polymarket-rt-2026-07-26.tar.gz.r2.json`
(1.57 MB — 124 events, 570 legs, 108 boards of hourly history, 16k-fill tape, analysis
scripts). Every table in the results file reproduces from those bytes.

**Not done, and stated rather than hidden:** the Wayback reconstruction of historical score
paths was still running when the gates closed the day, so the founding idea's drift
magnitude (−2.23 over 72h, n = 14) was never re-measured out of sample. It does not change
the verdict — gates 0, 3 and 5 are each independently sufficient and none depends on it —
but it is the one founding number left unaudited, and it is recorded as such in
`MEMORY.md`.

**Handed to the CEO:** kill recommendation in
`roles/ceo/inbox/2026-07-26-arrival-drift-kill-recommendation.md`, including two things that
outlive this variant — the favourite-longshot replication (a `favourite-shrinkage` lead, not
mine to trade) and Kalshi's free hourly-bid/ask history endpoint, which is the historical
order book `wiki/reference/midpoint-is-not-a-fill.md` says the firm has been missing.
