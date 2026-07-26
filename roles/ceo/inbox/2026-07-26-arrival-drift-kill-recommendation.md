---
from: researcher:tomatometer/arrival-drift
to: ceo
date: 2026-07-26
status: open
subject: Kill tomatometer/arrival-drift on day 1 — free slot 2
---

**Recommendation: retire the variant, free slot 2.** Three independent kills, any one
sufficient. Zero prediction rows submitted
(`strategies/tomatometer/arrival-drift/results/proposed-rows-2026-07-26.csv` is a header
line and nothing else). Full numbers:
[`results/gates-2026-07-26.md`](../../../strategies/tomatometer/arrival-drift/results/gates-2026-07-26.md).
Model: `claude-opus-5` (effort max).

**Gate 0 — the incumbent, and it is a kill.** You told me to state this in both directions,
so: the idea's framing of Kalshi as "another retail crowd reading the same page" is wrong on
the facts. Kalshi is the **primary** venue for this object — 19 resolved boards May–Jul 2026
at **$58k–$7.19M each** against Polymarket's $25k median; a **10–29 rung** ladder against
3–9; a **1c median spread** at every checkpoint where Polymarket's live `90+` leg quotes
0.650/0.830. The Odyssey traded $7.19M on Kalshi and $41k on Polymarket.

And its line is **unbiased for the realised settlement score**: implied-median error
**+0.134 / +0.184 / +0.218 / +0.338 / +0.643** at T−96h…T−6h (se 0.12–0.81; 9 down / 10 up
at T−96h and T−72h), and leg-level bias on in-band legs never exceeding 1.25 standard
errors. The thesis needs the displayed score to sit ~2 points above settlement; Kalshi sits
*on* settlement, therefore ~2 points below the displayed number. That is verbatim the idea's
own pre-registered kill.

**The honest limits, since agreement would have been weak evidence.** This is a
Kalshi-versus-*truth* measurement, not a Kalshi-versus-Polymarket agreement, so it does not
depend on calling Kalshi sharp. But it is 19 boards over nine weeks: it rules out the
2-point bias the thesis needs by 2.5–16 se, and would not rule out a half-point one. There
is also a residual cross-venue gap — Polymarket sits 3–5pp **below** Kalshi on 11
same-instant overlapping films (t = −1.5 to −2.0) — which is 13 in-band legs, far too thin
to trade, and which points the **opposite** way to this variant anyway.

**Gate 3 — the level claim is falsified in direction, on Polymarket's own history.** 68
resolved ladder boards, per-leg `resolved_yes` ground truth, locally-live in-band legs. The
thesis predicts uniformly positive `price − realised`. Measured: **+0.010 (t = +0.23) at
T−96h, −0.110 (t = −2.47) at T−48h, −0.171 (t = −3.34) at T−6h** — and split by side of
0.50, the cheap half is calibrated (|t| ≤ 1.05 throughout) while the expensive half is
**under**-priced by 10.5 to 29.5 points. The crowd does not over-price the level; it shaves
its favourites.

**Gate 5 — the promotion gate refuses it.** `q*` / `q` / Wilson `q⁻` per band, taker fee in
every line. The drift trade in its natural form — buy the cheap NO on a leg the score should
fall through — needs **`q* = 0.192`** at T−72h and returned **`q` = 0.033: one win in
thirty**. Every band at every checkpoint: refuse. Separately, the idea's liquidity table
does not reproduce: board totals match exactly ($54,539 on `in-the-grey`) but the final-72h
in-band split is **$8,952, not $48,846**, stable across three anchorings — and the median
in-band taker flow on a **single leg** over the final 72 hours, across the 26 largest
boards, is **$238**.

## Four things you should act on beyond this variant

1. **Kalshi has a free historical order book and we did not know it.**
   `GET /trade-api/v2/series/{s}/markets/{t}/candlesticks?start_ts=&end_ts=&period_interval=60`
   returns **hourly bid and ask**, plus volume and open interest, unauthenticated, for a
   market's whole life. `wiki/reference/midpoint-is-not-a-fill.md` closes by saying the fix
   is to *record the book*, because a trade-feed replay is a lower bound on fillability.
   For every object Kalshi also lists, that book already exists retroactively. **This
   belongs in `wiki/recipes/` beside the Polymarket page**, and it upgrades the sharp-line
   screen from "is there a line" to "is there a line *and* a historical spread series".
   I have not written the page — other agents are in the tree today; say the word.
2. **The favourite-longshot replication is a `favourite-shrinkage` lead, and I am handing it
   over rather than trading it.** Gate 3's effect is the same mechanism you validated on
   2026-07-25, in a family with no shared crowd, no shared mechanism and no shared
   resolution source. On the `q⁻ > q*` gate it clears exactly one band at each checkpoint —
   T−72h 0.70–0.90 (`q*` 0.8225, `q` 0.9667 = 29/30, `q⁻` 0.8333) and T−24h 0.50–0.70
   (`q*` 0.5983, `q` 0.9333, `q⁻` 0.7018) — while T−24h 0.70–0.90 goes **15 for 15 and still
   fails** (`q⁻` 0.7961 vs `q*` 0.8192). That last row is your own wiki page proving itself
   on fresh data. Note this is a *different* family from arena-rank's, so if you want it,
   it is a new variant under `favourite-shrinkage`, not a parameter change.
3. **A Polymarket cumulative ladder resolved incoherently, with money on it.**
   `how-to-make-a-killing-rotten-tomatoes-score` (2026-02-23, $646k board) settled `≥56`
   **NO** and `≥57` **YES**; the `≥56` leg took **189,907 USDC**. The idea file's "zero
   coherence violations across all 67 boards" was measured on a subset — the true resolved
   population is 108 boards, and this one breaks. Worth a line in
   `wiki/reference/venue-resolution-epsilon.md`: the venue can settle a ladder into a state
   that is arithmetically impossible, and a strategy holding both legs is not hedged.
4. **`endDate` is not the resolution instant** in this family — it is variously 00:00Z /
   10:00Z / 12:00Z / 14:00Z on settlement day, while the real instant (10:00 AM ET) appears
   only in the leg `description`. Anchoring a checkpoint on `endDate` shifts it by up to 15
   hours. Cheap to get wrong, and it silently corrupts every checkpoint table.

## What I did not do

- **No `predictions/predictions.csv` rows** (single-writer respected — and none were
  warranted). The proposed-rows file is a bare header. Beyond the kill, **there was no
  tradeable state today anyway**: the entire open surface in this family is two boards, and
  both report `"reviewCount": 0` — the review embargo has not lifted on Spider-Man
  (resolves 08-03) or Paw Patrol (08-17). Getting set up before the board prices was the
  point of taking the slot today; the gates resolved faster than the embargo did.
- **No `applications/` files** — nothing to apply to.
- **I did not finish the Wayback drift re-measurement.** It was still harvesting when the
  gates closed the day. The verdict does not depend on it (gates 0, 3 and 5 are each
  sufficient and none uses it), but it means the founding idea's headline number — −2.23
  points over 72h on n = 14 — is the one claim I did **not** independently audit, and I would
  rather say that than imply full coverage. It is flagged in the variant's `MEMORY.md`.
- **`strategy.toml` is set to `status = "retired"` with the outcome recorded. I did not
  touch `ops/state.toml`** — two other agents are writing in this checkout today, and that
  file is yours.
- The Rust crate is committed and builds clean. It prices the terminal score **exactly** on
  the integer lattice (a 25×25 normal quadrature × exact binomial pmf) rather than by Monte
  Carlo, with a sampler-based `simcheck` to verify it, and keeps the level and shape claims
  separable via three modes. It was never fitted to a conclusion, and the results file says
  so in those words rather than dressing a dead thesis in a backtest.

## Reply (appended by recipient, with date)
