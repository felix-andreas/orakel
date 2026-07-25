# Decision log

Append-only. Every structural change to the firm gets a dated entry: **what changed, why,
who decided**. Newest first.

---

## 2026-07-25 — Venue fees found, verified, and priced into every policy (v2)

The market researcher discovered Polymarket charges real taker fees, undocumented in our
wiki: `fee = shares × rate × p × (1−p)`. The execution engine had `fee_bps = 0`
everywhere, so the matrix I had just reported was too generous. Corrected same day.

Verified three independent ways (not taken on the wiki's word): the published docs; each
market's own `feeSchedule` on Gamma across 600 markets; and a fit against ~2,300 real
executed fills, which additionally **ruled out** the plausible `min(p, 1−p)` form.
Established facts: charged **per taker fill, on entry and on exit, never at resolution**
(so a held position pays once, a round trip twice); **makers pay zero**; and — the piece
nobody had right — **gold, silver, WTI, SPY and NVDA are `finance` at 0.04**, read off
each market rather than guessed. Sports was 0.03 before 2026-07-10, which matters for
any future sports backtest.

Eight `-v2` policies were created rather than edited (DESIGN.md §5), and v1 re-runs
bit-identical — the proof that fees are the only difference.

**All three conclusions I reported survive**, with two changes worth stating: fees take
8–25% of gross PnL; the sell/buy split sharpens from (+7.75c / +0.47c) to
(**+7.20c / −0.22c**), i.e. the naive buy book is now an outright money-loser, which
independently vindicates ladder-rv's decision to disable buys; and while `harvest` keeps
the top rank, its lead over `sniper` collapses by 74% (213pp → 54pp) because it pays the
fee twice. A ranking that survives is not the same as a conclusion that is unchanged.

---

## 2026-07-25 — Execution layer built; watchlist mirroring moved to run start

Built the execution simulator (`execution/`): eight named policies, two signal sets, the
capital-lockup accounting rule (annualized return on locked capital, not cents/trade),
conservative fills (never at mid), and a refusal to name winners below n=30.

First matrix (details in `execution/results/SUMMARY.md`) produced three findings:
**(a)** filtering is the single biggest lever — `mirror`→`gate` roughly doubles
annualized return on strictly fewer trades; **(b)** the sell-side house finding
replicates independently (sells +7.75c/trade vs buys +0.47c on the naive policy);
**(c)** the two headline metrics genuinely disagree — `sniper` wins cents/trade,
`harvest` wins annualized return because it holds 3 days instead of 10 — which is the
design's own argument reproduced on data.

**And one sobering result: on our OWN live predictions, seven of eight policies take
zero trades.** After a 3c spread our 21 scored predictions had under 5c of *executable*
edge: they were 2–7c wings whose "edge" was measured against a midpoint that is not a
tradeable price. Being right 21/21 and having nothing to trade are compatible states,
and the firm now measures both.

Operational change (CEO playbook step 3): the R2 watchlist is now rebuilt from **active
applications at the START of every run**, not from predictions at the end. Root cause of
the missing books: the watchlist grew 18→40 markets 26 minutes *after* the run that
produced 18 of the 21 signals, so the hourly snapshot worker had never seen them. Fixing
the order makes future signal sets book-complete at zero cost.

Also corrected a 10× arithmetic error in DESIGN.md §3's worked example (the formula and
the engine were always right; the prose was not) — caught by the implementing agent.

---

## 2026-07-25 — arena-rank: thesis killed, mechanism kept (variant split)

`arena-rank/satellites` day-1 falsification killed its founding thesis on gate 2: the
anchor-calibrated order-statistic simulation lost to the satellite crowds (log-loss
1.244 vs 0.504, better in 1/10 cohort-months), and the portfolio-correlation effect
calibrated to zero. Root cause is now a wiki rule: the leaderboard publishes CIs about
LATENT skill (±5.9) while the market resolves on the PRINTED rank, whose realised 7-day
sd is 1.23 — using published bars as σ over-disperses and fades favourites.

But one mechanism survived with better statistics than the original claim: the crowds
are **underconfident in their own favourite** (+9.2pp vs de-vigged price at T−7d, se
1.9pp, t=4.77, 9/10 months), and sharpening their distribution gains +0.111 log-loss
OOS (t=+2.63; at T−7d t=+7.49, 10/10 months).

Decision per our taxonomy (different approach → new variant, not a version bump):
retire `satellites` with its post-mortem, create `arena-rank/favourite-shrinkage`
(`supersedes = "satellites"`) carrying only the surviving evidence. The slot clock is
NOT reset — day 1 is spent. **A kill test is pre-registered for day 3**: the
favourite-longshot gain must concentrate in a fundable 0.60–0.90 band; if it exists only
on 0.93–0.99 favourites, return on locked capital after spread cannot justify a slot and
the variant retires. The retired simulation's forward prediction rows were deliberately
NOT logged — we do not put a dead mechanism's calls into the track record; day 2
produces shrinkage-based rows for the same cohort, still ahead of the 07-31 resolution.

---

## 2026-07-24 — Model routing: Opus 5 everywhere (Felix)

Opus 5 released; Felix directs: use it wherever Fable was used, at **max** effort, and
**xhigh/high** for the roles that already ran Opus. Rationale carried over from the
original split — idea generation and day-1 falsification are the highest-leverage
decisions (each bad call burns a slot), so they get the deepest thinking; recurring
daily research and execution are more mechanical. Fable is retired from routing. Note:
prediction rows and worklogs must now record `opus-5` (+ effort) as the producing model
— the model column keeps separating method-edge from model-edge.

---

## 2026-07-22 — CEO instantiated (Felix's instruction)

The scaffolding session is promoted to the CEO: it becomes the CEO's persistent session,
woken daily by a self-bind trigger at 01:07 UTC (03:07 German summer time — inside the
working window year-round). Felix chose self-bind over fresh-session mode because it
keeps all MCP connectors (verified live earlier today; fresh sessions from agent-created
triggers lose them). Model routing per constitution §4: subagents on Fable run at high
effort only; Opus subagents may run extra-high. First CEO run starts immediately:
market researcher scan → first strategy idea → fill research slot 1.

## 2026-07-22 — Founding (Felix + scaffolding session)

The firm is founded as the successor of `poly`, redesigned around lessons from its ~2-week
run. Founding decisions, agreed between Felix and the scaffolding agent:

- **Research unit = strategy variant**, not market. poly's per-market research (3
  researchers/market) produced correlated one-shot papers, n=2-3 per method, and its
  `strategies/` promotion path never fired once.
- **Family → variant → application** taxonomy with the params-plus-small-local-changes
  membership rule; split variants rather than over-generalize. Versions = name postfix +
  `supersedes` field.
- **5 research slots**, ≥10-day trials judged on scored evidence (guideline: ≥15 scored
  predictions across ≥3 markets beating the market baseline + backtests on resolved
  markets). CEO decides promote/discard/extend.
- **Roles with own memory + inboxes**: CEO (orchestrates, never researches), market
  researcher (daily scan → one idea/day), researchers (per slot), executors (per live
  variant). Felix is a role with an inbox.
- **One daily CEO trigger owned by Felix**; CEO spawns everything else and may create
  further triggers inside the working window (weekdays 02:00–15:00, weekends 02:00–08:00
  Europe/Berlin).
- **No hard token cap initially**; spend logged per run. Model routing: Fable
  (high/xhigh) for market research + initial research, Opus for recurring research +
  execution.
- **Git = index, R2 = bytes** (poly committed 70 MB of snapshots into git). Upload-before-
  commit, immutable content-addressed keys.
- **Execution layer from day one** (paper only): versioned execution policies with signal
  combination folded in, PnL-backtestable. Real trading stays a Felix-only decision.
- **Dashboard**: dynamic Rust app on Cloudflare Workers, private via Cloudflare Access,
  htmx + ECharts, deployed from agent sessions via wrangler.
- **Wiki seeded** with a curated handful of durable poly insights (market selection,
  favorite-longshot bias, thin-market price reading, crowd calibration, wash-trade
  detection, Polymarket API recipes); everything else clean slate.
