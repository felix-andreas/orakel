# climate-nowcast/gistemp-era5 — Worklog

One dated entry per run. Name the model that did the work.

---
## 2026-07-24 — day 1 (trial start): backtest-first → KILL recommendation

Model: fable (high) — claude-fable-5.

- Pulled frozen ERA5 + GISTEMP source snapshots (market researcher's R2 manifests,
  sha-verified). Built `src/pull_markets.py`: /public-search discovery → 36 Gamma
  events (28 resolved monthly bucket instances Apr-2024→Jun-2026 — more than the
  idea's ≥20 — plus live July, ranking events, annual), 197 legs, full CLOB
  prices-history (0 empty), 32 live books, 48 trade tapes. Built GISTEMP vintage
  archive from Wayback CDX: 50 unique captures of GLB.Ts+dSST.{txt,csv} 2024-03→
  2026-07 (`src/parse_vintages.py` → `data/gistemp_vintages.csv`). All raw frozen to
  R2 before manifest commit (`data/backtest-raw-2026-07-24.tar.gz.r2.json`, 6.0 MB).
- `src/backtest.py`: point-in-time nowcast (ERA5 3-day lag; GISTEMP = latest vintage
  ≤ t; fits on years < target; σ+bias frozen from pre-sample 2015–23 hindcast) at
  day-15/day-21/month-end/pre-print; gates 1–4. `src/capacity.py`: gate 5 from tape.
  Python not Rust: one-day statistical backtest, numpy/scipy regression + distribution
  work, no long-lived service; wiki recipes stay language-neutral.
- **THESIS BROKEN, saying it loudly (playbook): 3/5 kill conditions met.** Market
  beats model log-loss at ALL checkpoints (preprint 0.253 vs 1.085, model 2/28);
  modal calibration INVERTED at preprint (priced 0.836, realized 0.929 — crowd
  underconfident; implied crowd σ 0.014–0.018 vs our realized floor 0.038); delayed
  t+24h PnL −2.5c/trade, preprint 0/56. Gate 4: markets resolve on first print
  (0/28 mismatch), today's file would mis-grade 9/28 — backtest rebuilt on vintages.
  Gate 5 passes ($2.2k–43k late-window fundable flow). Full report:
  `results/backtest-2026-07-24.md`. CEO inbox message filed (kill rec, slot 2 free).
- No prediction rows submitted (backtest grades the live July divergence as the exact
  losing trade class). `applications/2026-07.toml` filed active=false, for-the-record
  pipeline output only. STRATEGY.md as-built; FAMILY.md cross-variant lessons
  (proxy-vs-primary-inputs screen; first-print discipline).
