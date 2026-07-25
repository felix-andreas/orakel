# series-shape/bo3-derivatives — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-25 — day 1 — **opus-5 (max)** — KILL

Ran the fixed day-1 order: gate 5 first, then gate 0, then the rest. Both produced kills.
Full numbers: `results/backtest-2026-07-25.md`. Data frozen to R2 before this commit
(`data/*.r2.json`, 6 manifests, all verified present).

**Gate 5 — obtained the external line the idea could not.** Three independent routes, no
accounts: **Pinnacle** guest arcadia API (`/0.1/sports/12/matchups` →
`/0.1/matchups/{id}/markets/related/straight`) publishes `spread ±1.5` and `total 2.5` on
`bestOfX: 3` matchups — precisely our two legs; **Smarkets** v3 (an exchange, so the mid
carries no vig at all); and a retail book via server-rendered HTML, cross-checked against a
**Stake.com** quote. Matched 34 live Polymarket BO3s to Pinnacle, re-oriented the handicap
onto Polymarket's `outcomes[0]`, de-vigged both by normalisation and by a power fit:
handicap median |Δ| **1.08pp** (28/33 within 3pp), totals **0.69pp**, moneyline **0.93pp**;
on books with ≤2c spread the mean Δ is **−0.13pp (se 0.34)**. Pre-registered kill was 3pp.
The idea's three claims measured against the sharp line: (A) +6.1pp → **−0.56 ± 0.34**,
(B) +9.5…+13.8pp → **−0.87 ± 0.67**, (C) −9.0pp → **+0.65 ± 0.60**.

**Gate 0 — found the artifact.** Independent harvest (17,338 resolved esports events vs the
idea's 16,959; 12,581 triples; 36,167/36,167 CLOB token histories, 0 failures). The idea's
semantics and timestamps are *correct* — leg typing 12,581/12,581, identity 10,465/10,472 =
99.933%, moneyline collapsed at T−1h in only 0.09% of cases, pre-match sd 0.0145 vs
post-start 0.2076. The error is elsewhere: **a Polymarket derivative leg with no resting
orders quotes a ~0.50 midpoint (the mean of a 1c bid and a 99c ask), and 23% of handicap
legs never move pre-match while 85% sit under $5k volume.** Decomposing the same sample by
book liveness: dead **−7.11pp**, near-flat −3.19pp, **moving +0.08pp (se 0.58)**. The
moneyline bias inverts with liquidity (+6.5pp under $5k → **−4.0pp at ≥$50k**), which
directly contradicts the idea's "the deep moneyline is miscalibrated by +6.1pp".

**Gates 1–4.** 1 passes (ledger reproduces). 2 moot — on live books the market's own price
already equals the realised rate, so there is nothing for a fitted map to beat. 3 fails
hard: net **−2.94c** (T−6h), **−3.26c** (T−6h→T−1h), **−5.67c** (T−24h→T−6h) per share
after the 1.2c sports taker fee and a 2c adverse fill; monthly-clustered t=−1.13 with a
Jan −14.3pp → May +4.2pp sign flip. 4 passes on supply (~5 events/day clear spread ≤5c and
top-of-book ≥$500) — and that is the point: **every book that passes gate 4 prices within
~1pp of Pinnacle.** Also measured: 4.63% of BO3 triples settle 50-50 (cancelled / tie /
delayed), which a resolved-only ledger silently drops.

**Structural bug found and fixed in our own path:** "BO3" must be typed from the LEGS
(handicap margin 2 ∧ totals threshold 3), not the title — a BO5 handicap is also "wins 2 or
more maps", and mis-typing imports 1,597 BO5/BO7 series, dropping the identity check from
99.9% to 97.2%.

**Output.** No prediction rows and no active applications: the gates say the signal is
noise, and rows we believe are wrong would pollute the track record rather than test
anything the 34-pair Pinnacle comparison has not already settled more sharply. One
`active = false` application records the best board (`lol-vit-g2` handicap: 1c spread,
$38.6k within 10c, passes every liquidity gate, Pinnacle 0.591 vs Polymarket 0.585).
**Recommend `status = "retired"` and freeing slot 3.**

Tooling note (CODING.md: name the language and why): Python rather than Rust for the whole
pipeline — the work is IO orchestration over ~1,500 Gamma pages plus 36k CLOB fetches, plus
small tabular stats, against a same-day deadline set by an 11:00 UTC match slate. `src/`
holds `harvest.py`, `gate0.py`, `gate5.py`, `compare.py`, `backtest.py`.
