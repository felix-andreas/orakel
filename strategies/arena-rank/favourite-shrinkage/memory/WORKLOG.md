# arena-rank/favourite-shrinkage — Worklog

One dated entry per run. Name the model that did the work.

---

## 2026-07-26 — day-3 fundable-band kill test (run early) · claude-opus-5, xhigh effort

Slot 2, day 2 of the trial. Ran the pre-registered day-3 test today because the cohort
checks 2026-07-31 and any trade would have had to be placed now.
Full study: `results/fundable-band-2026-07-26.md`.

**Outcome: the test PASSED, and zero rows were proposed.** All 7 migrated July
applications are `active = false`. CEO decision memo:
`roles/ceo/inbox/2026-07-26-favourite-shrinkage-band-test.md`.

What was done:

1. **Pulled the day-1 CLOB archive from R2** (`clob-prices-2026-07-25.tar.gz`, 2,565 token
   histories) and reproduced `gate_flb` exactly (+0.1108 OOS, t=+2.63, 9/10 months) before
   changing anything.
2. **Band split** (`src/band_split.py`): fundable 0.60–0.90 gain **+16.8pp (t +5.94)** /
   **+12.49c/trade** / **+457% annualised**, vs +4.9pp / +2.82c / +137% at 0.93–1.00.
   Instance-level (46 distinct instances): +17.2pp, t=+7.03. The gain is 3.4× larger in the
   fundable band — the test's premise ("edge only at 0.93–0.99") is false.
3. **Break-even bounds** (`src/robust.py`): the fundable band is the only one whose 95%
   lower bound on the favourite's win rate (0.846) clears its break-even (0.829). The
   0.93–1.00 band is 16/16 and needs 97.2% against a bound of 0.819 — **2.83 losses per 100
   wipe it out**. 10-fold month jackknife on the fundable band: +11.25c to +14.10c, t ≥ 4.05.
4. **Mandatory re-checks, both clean.** Leg-sum gate: the gain lives entirely in the
   leg-sum ≤1.05 half (+0.129 vs −0.184); 25/138 checkpoints exceed 1.05. Nulls (uniform,
   flat-0.90, and the new margin null) all lose to the market in every band. Phantom split:
   **137/138 checkpoints LIVE**, so the live-book headline equals the pooled headline.
5. **Fresh live state** (`src/live_state.py`): books + full Data API tape for all 7 boards,
   182 legs, 26,158 trades. Yesterday's `[book]` blocks were already stale — the Chinese
   favourite had moved 0.8275 → 0.7765 on *no new leaderboard data*, nosc-3 0.934 → 0.9831.
   Live leg-sums 0.971–1.028, all inside the gate. Fee read live: `tech_fees`, 0.04, taker-only.
6. **Executable-ask economics** (`src/live_analysis.py`): six boards sit at 0.935–0.983
   de-vigged, four quoting an **ask of 0.990** (+0.46c, +33% annualised, break-even 0.990,
   one loss per 100 wipes it). The α=1.75 rule hits its 0.995 clip on all six, so the
   "edge" is clip-minus-ask, not a model view.
7. **Fill evidence from the tape**, taker buys folded to Yes-equivalent as `tools/fillcheck`
   does: 7-day realised buying on our side is $58 (overall-sc-1), $209, $606, $668 on four
   boards against book depths of $7.2k–$25.9k. Only chinese ($33.2k) and nosc-3 ($11.1k)
   have real flow.
8. **The finding: a leaderboard-MARGIN screen** (`src/margin_null.py`,
   `src/persistence_screen.py`, `src/h2h_preliminary.py`, `src/chinese_persistence.py`).
   The one in-band board (chinese, 0.7997) sits at a **+3 margin between two Preliminary
   sub-3.8k-vote rows**. The migrated file's "persistence 0.976–0.982" is the top-30
   **pooled** figure; measured properly, sd(Δscore) is 5.87 (Preliminary) / 6.55 (<5k votes)
   vs 1.60–2.25 (established), gap-3–5 persistence is 0.846 (11/13) for both-<5k pairs, and
   Chinese company-level leadership at margin 0–3 persists 0.44–0.54 over 1–14 days.
   The historical cell matching this board exactly (incumbent, margin 0–3, priced 0.60–0.90,
   n=5/4 months): **market 0.800 → realised 0.800, gap +0.0pp**, our rule 0.951 (−15.1pp,
   the worst miss in the sample). Where the band gain *does* live: margin ≥4 with the
   incumbent (+11 to +13pp, n=27 and 56) or margin 0–3 with the market backing the
   challenger (+20.7pp, n=12).
9. **Archived the live resolving tables** (`src/archive_tables.py`, memory duty 1) — all
   four slices, `data_date = Jul 21, 2026`, unchanged since Jul 21 with 1–2 refreshes due
   before the check.
10. **STRATEGY.md** applicability rewritten to four clauses (margin added). Applications
    deactivated with per-file reasons; the stale 0.976–0.982 citation corrected in place.

Bugs found and fixed in my own work, worth not repeating: (a) I first measured persistence
on the *k-th distinct company* instead of the *k-th ranked model* — Anthropic owns ranks
1–4 so it owns places 1, 2 and 3; `resolve.winner` is the authority. (b) `"sc_" in
"text_overall_nosc_2"` is True, which silently pointed every no-style-control board at the
style-control-ON table.

Data frozen to R2 (uploaded before the manifests were committed):
`data/live-2026-07-26.json.gz.r2.json` (books + tape) and
`data/arena-tables-2026-07-26.tar.gz.r2.json` (today's resolving tables, raw + parsed).

Not done / handed to the CEO: the wiki pages (outside my folder — three candidates listed
in the memo), and the slot decision itself. August/September cohorts are listed but
**unpriced** (leg-sums 6.5–12.5), so there is no trade available for roughly two weeks.
