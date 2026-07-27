# Decision log

Append-only. Every structural change to the firm gets a dated entry: **what changed, why,
who decided**. Newest first.

---

## 2026-07-26 (late) — Gate 0 becomes a regression test: Kalshi's whole catalogue is one call

The market-researcher cycle run under Felix's "no already-efficient markets" directive filed
**no idea**, which is the correct outcome and the one the playbook now asks for. It worked up
the strongest candidate the scan produced — Polymarket's shipping-chokepoint boards resolving
on IMF PortWatch, ~$36M open, 19 resolved weekly ladders, a pure counting process on a free
government feed with zero taker fee — and killed it on three measured gates.

The board passed **every liquidity screen we own**: 1–2c spreads, 174/125 distinct wallets,
~$28k of seven-day taker flow on *both* sides of the leg we would trade, leg-sum 1.019. Worth
saying plainly, because it is the good news buried in a kill: **our liquidity gates are not
the binding constraint any more.** The three ways a quoted price lies are now screened for,
and boards clear them.

What killed it:

1. **Gate 0, measured rather than described** (the rule I wrote this morning after promoting
   an idea on a description). Kalshi's `KXHORMUZWEEKLY` declares **our exact PortWatch
   resolution URL**, trades 156k–446k contracts a week at 1c spreads, and is unbiased for the
   realised settlement: mean error +2.63, se 6.19, **t = 0.42** over n=9.
2. **The cross-venue fallback is also dead.** Polymarket priced the realised winner +4.6pp
   *higher* than Kalshi (se 3.8, t ≈ 1.2). If anything we are the sharper venue here — there
   is no spread to harvest in either direction.
3. **The one that generalises: the feed is not a fixed number.** PortWatch restates settled
   weeks by **−9% to +247%**. Rebuilding all 19 resolved boards from the live API reproduces
   the **wrong winning bucket on 7 of them (37%)**. For the week of 11–17 May, Kalshi settled
   15, Polymarket resolved 40–59 two days later, and the feed reads 52 today. No vintage
   archive exists. So the family is **unbacktestable**, which is a different and worse
   category than "efficient" — we could not have measured our own edge even if one existed.

**The capability this produced is worth more than the idea would have been.** Kalshi's entire
catalogue — **12,186 series with declared `settlement_sources` and `expiration_value`, the
exact settled integer** — comes back from **one unauthenticated call**. Gate 0 stops being an
argument and becomes a regression test: before any modelling, ask whether Kalshi already
declares the same resolution source, and if so compare their line to the settlement. Promoted
to `wiki/reference/sharp-line-screen.md`. `wiki/reference/first-print-vintages.md` gains a
mandatory companion gate: **rebuild ≥3 settled instances from the live feed and check they
match what the venue actually paid**, before modelling anything.

Screening those 12,186 series against everything we have considered found Kalshi covering
nearly all of it — Rotten Tomatoes (244 series), Netflix ranks, MrBeast views, GPU prices,
home values, reality-TV eliminations, chess, earthquakes, Emmys. **The one clean hole is
domestic box office**, against a deep Polymarket family ($17.1M on the Avatar opener, live
boards at $261k and $204k) resolving on The Numbers' *final* figures, explicitly "not studio
estimates". Recorded as a lead, explicitly unverified: its own gate 0 is BoxOfficePro's
forecast, which 403s us live but is archived in Wayback. Half a day, and it runs before a slot
is spent — not after, which is the mistake I made this morning.

**Two process failures, both mine to own.**

I sent the `/execution` route-reassignment message to the **wrong agent** — it reached the
market researcher, whose brief forbids touching `dashboard/`, and which correctly refused to
act on it and told me. The backtest agent therefore never got the instruction and added the
redirect I had tried to prevent; I removed it by hand. Agent ids returned from a parallel
spawn are not ordered the way the calls were written, and I assumed they were.

The backtest agent then ran a repo-wide `git add` and swept the researcher's six files into
its own commit, despite an explicit instruction in its brief not to. Same failure as
2026-07-25. Content is intact on `main` and verified file-by-file; only the commit message
lies. The researcher declined to rewrite shared history with another agent live, which was the
right call — the fix is more dangerous than the defect. Since briefs demonstrably do not
prevent this, the rule has been moved into **`AGENTS.md`**, which every agent reads before
anything else.

---

## 2026-07-27 — WTI touched $85 and the headline flipped: ladder-rv now LOSES to the market

Two markets resolved overnight, both YES, and we were wrong on both.

`will-wti-dip-to-85-in-july-2026` **touched**. We predicted no-touch four mornings running —
0.4937, 0.3520, 0.3928, 0.3650 — while the market went 0.525 → 0.410 → 0.415 → **0.715**. The
second, `will-wti-dip-to-90-...-from-july-25`, also touched, on a leg where we broadly agreed
with the market (0.9409 vs 0.9470).

**The variant's headline inverts: mean paired improvement −0.0172 over 25 rows, from +0.000945
over 21.** `dip-to-85` alone contributes **−0.4510**, which is larger than the total loss of
−0.4312 — every other row still nets +0.0198.

Three things follow, and the order matters because only the first is a defence and it is a
weak one.

**1. Scoring now aggregates per MARKET as well as per row, because rows are not independent.**
We predict the same market every morning, so one barrier touch is scored once per day it was
open. Those four `dip-to-85` rows are one event. Counted per market: 19 markets, mean
**−0.0051**. Counted per row: 25 rows, mean **−0.0173** — 3.4× worse, entirely because we
happened to predict the losing market four times. Both are negative; the flip is real either
way. `scoring/` gained a `market` level so the number of *events* a conclusion rests on is
visible next to the number of rows, and this cuts both ways — it would have deflated the
21/21 headline too.

**2. The 07-26 row is the research finding and it is not about counting.** That morning the
market had repriced to 0.715 and we stayed at 0.365 — a 35-point disagreement, and the market
was right. On 07-23 we were within 3 points of it. So the model did not merely have a
different view; **it failed to move when the market did.** Something priced that move in and
our pricer did not see it. Slot 1 has been told to explain that specifically, today.

**3. Reachability makes this worse, not better.** Tradeability went 2/21 → **6/25**, because
mid-board WTI legs at 0.4–0.7 are exactly the reachable ones. So the rows we could actually
have traded are the rows we lost on, and the wings we "beat the market" on remain the ones
nobody would trade with us. The two halves of the ledger are now cleanly separated: where
there is liquidity we were wrong, and where we were right there was no liquidity.

This is the caveat I have been repeating since the first scoring run — "21/21 was easy OTM
wings, one week, one regime" — arriving on schedule. It is also why the 07-31 resolution
matters more than ever: ~48 further rows on the July commodity boards, four days out, and the
trial review is 08-02. No slot decision today; the evidence that decides it lands on Friday.

Recorded by the CEO (claude-opus-5).

---

## 2026-07-26 (late) — Felix: don't research already-efficient markets; "Execution" is renamed to Backtest

Two directives, both correcting something the firm was getting wrong.

**1. "we shouldn't pick markets that a already efficient (like where will price of NVDA be)."**
This is the failure that has cost us the most, and our own data now names it. Four of our six
dead variants died because a professional already priced the object: `bo3-derivatives` against
Pinnacle, `satellites` against the market's own rank persistence, `quake-etas` against an
implied Fano factor of 1.362 versus an empirical 1.358, and `arrival-drift` today against
Kalshi — whose line is *unbiased for the realised settlement* on boards 3–300× the size of
Polymarket's. In that last case the underlying observation was true and replicated at 8×
sample; it was simply already in the price.

The rule going forward: **a liquid, heavily-traded board on an object professionals price is
not a research target.** Anything shaped like "where will <liquid asset> be on <date>" is out
unless a specific structural reason the crowd is wrong survives our own screens. What we want
instead are boards where the counterparty is *structurally* unable to be sharp — where no
professional cares, where the barrier is work rather than information, or where the crowd
reasons by narrative and the answer is arithmetic. That is also where our only durable
advantage (high-performance Rust, no deadline) actually applies.

**2. "execution would be real execution. i think we rather want a backtest page."** He is
right and the point is substantive, not cosmetic. `CONSTITUTION.md` makes real trading a hard
line, so a surface called "Execution" claims something the firm does not do; everything on it
is a replay of stored signals against stored prices. The dashboard route becomes `/backtest`
(old paths redirect). The **repo directory `execution/` does not move** — it is referenced
from `ARCHITECTURE.md`, this log, several `strategy.toml` success guidelines and the CEO
playbook, and the rename is a presentation change only. He also called the page overloaded and
asked for a full rework, which is dispatched.

Recorded by the CEO (claude-opus-5). The selection rule graduates to `wiki/market-selection.md`
once the market-researcher cycle currently editing that file has finished.

---

## 2026-07-26 — tomatometer/arrival-drift killed on day 1 by the gate I should have run before promoting it

I promoted the Tomatometer idea into slot 2 within three hours of it being filed, on the
strength of its own description of gate 0: that Kalshi runs 233 Rotten Tomatoes series but is
"another retail crowd reading the same page". The day-1 researcher measured that description
and it was wrong on the facts. Kalshi is the **primary** venue for this object — 19 resolved
boards at $58k–$7.19M against Polymarket's $25k median, a 10–29 rung ladder against 3–9, a 1c
median spread where Polymarket's live `90+` leg quotes 0.650/0.830, and The Odyssey traded
$7.19M there against $41k here. Its implied score is **unbiased for the realised settlement**
at every checkpoint from T−96h. The thesis requires the displayed score to sit ~2 points above
settlement; Kalshi therefore already sits ~2 points below the displayed number, which is
verbatim the kill the idea had written for itself.

Two independent confirmations, either sufficient alone. **Gate 3:** on 68 resolved ladder
boards with per-leg ground truth, `price − realised` runs +0.010 (t=+0.23) at T−96h to
−0.171 (t=−3.34) at T−6h — the level claim is falsified in *direction*, and the expensive
half is under-priced by 10.5–29.5pp, which is favourite-longshot bias pointing the opposite
way to this trade. **Gate 5:** the natural form of the trade needs `q* = 0.192` and won 1 of
30, and the idea's liquidity table does not reproduce — board totals match exactly, but the
final-72h in-band split is $8,952 against a claimed $48,846, with median single-leg in-band
flow of $238 over 72h.

`slots_active` back to 1. Variant retired, folder kept as the post-mortem.

**The process failure is mine, and the fix is cheap.** Naming a candidate incumbent and
characterising it is not running the sharp-incumbent screen — it is deferring it, and the
deferral cost a slot-day. Added to `roles/market-researcher/PLAYBOOK.md`: if an idea names
any venue, model or public tool that might already price the object, the measured comparison
goes **in the idea file**, and if the data cannot be got the idea is filed as `needs-gate-0`
rather than `backlog`. I will not spend a slot on an unmeasured incumbent again. An idea filed
honestly as unverified is worth more than one filed confidently as clear.

Worth being clear that the day was not wasted, because this is what day-1 kills are for — six
of our eight variants have now died on day 1 and every one produced something durable. This
one produced four things, and the first is significant beyond the variant:

1. **Kalshi publishes a free hourly bid/ask history** (`candlesticks`). That is the historical
   order book `wiki/reference/midpoint-is-not-a-fill.md` says we have been missing — our
   fillcheck reachability numbers are a *lower bound* precisely because a resting bid nobody
   hit leaves no trace in a trade feed. A real quote history would replace the bound with a
   measurement. Top wiki item for the next run.
2. A **favourite-longshot replication in an unrelated family** — `arena-rank/favourite-shrinkage`'s
   mechanism appearing in film-score ladders, with one band clearing `q⁻ > q*` while a 15-for-15
   band still fails. `wiki/reference/break-even-win-rate.md` proving itself on fresh data the
   day after it was written.
3. `how-to-make-a-killing` resolved **incoherently** — `≥56` NO and `≥57` YES, with $190k on the
   broken leg. A venue-integrity data point.
4. `endDate` is **not** the resolution instant in this family; up to 15h of checkpoint drift.

The researcher also flagged, unprompted, that it never audited the founding −2.23-point drift
measurement because the Wayback harvest was still running — and that its Rust crate, though
verified against a 10⁶-draw sampler to TV 0.0057, was never fitted to a conclusion. Saying so
plainly instead of dressing a dead thesis in a backtest is exactly the behaviour I want.

Decided by the CEO (claude-opus-5); analysis by the slot-2 day-1 researcher (claude-opus-5, max).

---

## 2026-07-26 — New status `parked`; arena-rank/favourite-shrinkage passes its kill test and loses its slot anyway

Slot 2 ran its pre-registered day-3 band test a day early — correctly, since the cohort
checks 07-31 and any trade had to go on now. **It passed decisively.** The favourite-longshot
gain concentrates exactly where the variant committed it must, in the fundable 0.60–0.90
band: +16.8pp over n=74 across 10 months, t=+5.94, +15.2% return on locked capital, 95%
lower bound on the win rate 0.846 against a 0.829 break-even. It survives a leg-sum gate and
a 10-fold month jackknife.

And it proposed **zero rows**, because the mechanism has no expression in the cohort it was
handed. Six of seven July boards sit at 0.935–0.983 with four quoting an ask of 0.990 — pay
99c to win 1c, where one loss per hundred wipes the band out. The seventh is in band and
fails a screen the variant did not have this morning: at a 0–3 leaderboard margin with the
crowd backing the incumbent, the crowd is already right (n=5, market 0.800 → realised 0.800,
our rule 0.951 — the largest model error in the sample). August and September are listed but
**unpriced**, leg-sums 6.5–12.5, i.e. phantom ~0.5 on empty books. Nothing to trade for
roughly two weeks.

**Introduced `parked` as a variant status** (`trial | live | parked | retired`;
`strategies/README.md` documents it). `retired` means a gate killed the thesis and the folder
is a post-mortem. `parked` means the thesis held and has no expression: the boards it needs
are unlisted, unpriced, or outside the band it committed to. A parked variant releases its
slot and stops counting as active work — a slot that cannot trade for two weeks is a slot the
firm is pretending to use, and `ops/state.toml` claiming `slots_active = 2` would have been a
lie to the only human reading it. It keeps its trial clock and its evidence, and
`reopen_when` names the observable condition (an Aug/Sep board with leg-sum ≤ 1.05, favourite
in 0.60–0.90, passing the margin screen, from ~08-10) so reopening is checkable rather than
remembered. All 182 legs stay in the watchlist, so the snapshot worker accumulates the
evidence to reopen on whether or not anyone is watching.

`slots_active` 2 → 1. This is the first variant to leave a slot without being wrong.

Two durable pages written from it, both of which generalise well past this variant:

- **`wiki/reference/break-even-win-rate.md`** — the best artifact this firm has produced. A
  band that went 16/16 with t=+10.3 is uninvestable because it needs a 97.2% win rate and
  2.83 losses per 100 trades take it to zero. Report `q*` (break-even), `q`, and the 95%
  lower bound; refuse when the bound is below `q*`. This is now the standard promotion gate
  for any favourite-side trade, and it retires cents-per-trade as a ranking metric: cents
  ranked the bands 4:1, RoLC 5.2:1, and the bound ranked them tradeable / not / not.
- **`wiki/reference/sharpen-only-what-persists.md`** — a favourite-longshot correction inside
  a recurring ranking cohort is conditional on the ranking persisting; measure persistence on
  the resolution variable's own archive, at the granularity the board resolves on. Includes
  the pooled-statistic trap: the losing application cited a 0.976–0.982 persistence figure
  that was real but computed on established 50k-vote rows, quoted for a pair in a
  6.5-sd sub-population where it is 0.846. Sibling of `published-ci-vs-printed.md`.

Decided by the CEO (claude-opus-5); analysis by the slot-2 researcher (claude-opus-5, xhigh).

---

## 2026-07-25 — Dashboard reads are pinned to a commit SHA and issued concurrently

Felix: "it feels much slower now. it was fast before." He was right, and the first
diagnosis was wrong. The cause was not the fallback removal — it was that the
`GITHUB_TOKEN` worker secret landed **today**. Before that, `token(env)` returned `None`
and every read short-circuited straight to the compiled-in pack: twenty in-memory string
scans, zero I/O. The moment the token existed, those same twenty reads became twenty
*sequential* HTTPS round trips.

Measured before the fix: latency scaled linearly with the number of reads — `/decisions`
(1 read) 0.24s, `/strategies` (~10) 0.62s, `/` (~20) 0.87s warm and **2.9s** whenever the
60-second cache clock lapsed.

Two changes, both in the read layer:

1. **Pin content reads to a commit SHA.** `live::head()` resolves `main`'s SHA (memoised
   per isolate for 60s — that TTL is now the dashboard's only freshness knob) and every
   file and tree read goes to `?ref=<sha>`. The URL then names an immutable blob, so it
   caches for a day instead of a minute. A push produces new URLs; old entries just go
   unused. The every-60-seconds cliff is gone by construction, not by tuning.
2. **Issue independent reads concurrently** (`data::read_all`, `futures::join!`). A page's
   reads never decide each other — only the tree must land before we know which variant
   and run manifests to fetch — so every page is two waves rather than N steps. The
   per-variant and per-run loops were the worst offenders and are now single batches.

Result: `/` **0.87s → 0.41s**, every other page 0.21–0.38s, and latency no longer scales
with read count. All 17 routes verified 200 with correct content afterwards.

Not done, deliberately: fetching the whole repo as one tarball, or mirroring it to R2 and
reading through the binding. Both would shave another ~0.15s and both add a moving part;
at 0.4s the page is no longer the bottleneck. Revisit only if the repo grows enough that
the tree read itself gets slow. Decided by Felix, implemented by the CEO (claude-opus-5).

---

## 2026-07-25 — Dashboard has no fallback copy of the repo: it reads `main` or shows an error

The Worker compiled every renderable repo file into its own binary (~170 files, ~1.0 MiB)
and served that whenever a GitHub read failed. Felix called it: we don't need it, show an
error instead. Removed.

The reason it had to go is not the megabyte, it is the failure mode. A dashboard that
silently swaps in a build-time copy during an outage does not look broken — it looks like
a working dashboard showing numbers that happen to be old. Every number on it is a claim
about the firm's current state, so an invisible fallback is a machine for producing
confident wrong answers. A visible gap is strictly better than a plausible stale one, and
that is the same principle as the fillable-count decision above: prefer the honest hole.

- `main` at request time is the only source of truth. A failed read yields empty text.
- The top bar reads **`cannot read repo`** instead of a timestamp, and a red banner names
  the cause: no token, 401 (token revoked), 403 (rate limit or scope), 5xx (upstream).
- The banner is rendered once in `render::layout` from the freshness state, not by each
  page — a page cannot forget it. That deleted 15 per-page banner call sites.
- Transient failures (network, 5xx) are retried once. A retry costs one subrequest and no
  staleness, unlike serving an old copy; 401/403/404 are definitive and never retried.
- Bundle: 2390 KiB → 1306 KiB raw, 781 KiB → 521 KiB gzipped (−45%). Repo edits also no
  longer trigger a Worker rebuild, since nothing outside `src/` is compiled in.

Verified with the token removed: every route still returns 200, renders its empty state
plus the error banner, and leaks **zero** repo content (grep for `barrier-touch` on the
no-token render: 0 hits). With the token, 76 loads across every route showed one failure —
the first request against a freshly deployed version. It is rare, it is now visible by
design, and a reload clears it. Decided by Felix, implemented by the CEO (claude-opus-5).

Worth being precise about one thing, since it came up: we do **not** mirror the repo to
R2. The bucket holds hourly market book snapshots written by `workers/snapshot`, and
`tools/r2data` blobs. The thing just deleted was a copy compiled into the Worker binary.

---

## 2026-07-25 — Scoring reports tradeability next to calibration; the first batch was 2/21 fillable

Our first scored batch was 21 predictions, all `barrier-touch/ladder-rv`, and all 21 beat the
market on paired Brier. That headline was reported without checking the one thing that makes
it mean anything: `market_price` is a CLOB **midpoint**, and a midpoint on a wing leg is the
average of a near-zero bid and a fat ask. It is not necessarily a price anyone will give you.

Built `tools/fillcheck` (Rust, `attohttpc` behind the agent proxy like `r2data`), which
replays Polymarket's public trade feed for every market we predicted on and records the best
price a counterparty was demonstrably reachable at on each side, in windows of 1h / 24h /
life. `scoring/` now joins the result and prints `n_fillable` and `exec_edge` on every
aggregate.

The answer: **21/21 beat the market, 2/21 were reachable, 1/21 within the first hour.** The
one liquid row (`will-wti-dip-to-90`, $34k volume) is the row where we had essentially no
edge — 0.8263 against a market at 0.82 — and it contributed 11% of the batch's improvement.
The other 89% sits in SPY/NVDA weekly wings where `will-spy-reach-760` was scored at a 2.55c
midpoint against a best-ever bid of 0.12c.

What changed, and why:

- **`scoring/` will no longer print a Brier improvement without a fillable count beside it.**
  Reporting calibration as if it were money is the single easiest way for this firm to fool
  itself, and it already happened once.
- **Promotion decisions turn on `exec_edge`, not `improvement`.** `ladder-rv`'s
  `success_guideline` is amended; its 2026-08-02 review uses the executable number.
- **Weekly equity ladders are demoted to research-only** for `ladder-rv` — still predicted
  on, never counted in a headline without their own fill evidence. The monthly WTI/gold/
  silver boards resolving 2026-07-31 are the trial's real evidence.
- Durable rule: `wiki/reference/midpoint-is-not-a-fill.md`. Evidence:
  `strategies/barrier-touch/ladder-rv/results/executable-price-audit-2026-07-25.md`.

Honest limit, recorded so nobody over-reads it: `fillcheck` sees trades, not orders, so a
resting bid nobody hit is invisible to it. 2/21 is a lower bound. The real fix is recording
the book at prediction time (`bid`/`ask`/`depth_usd` columns, sourced from the snapshot
worker), which is now the top infrastructure item.

This independently corroborates the execution engine from the other direction: on
`orakel-live` signals, seven of eight execution policies took zero trades. Two methods, one
conclusion — this variant's demonstrated edge lives where the liquidity isn't. Decided by
the CEO (claude-opus-5).

---

## 2026-07-25 — Dashboard switched from build-time snapshot to live repo reads

Felix provisioned `GITHUB_TOKEN` in the environment, so it was set as the `orakel-dashboard`
Worker secret (`wrangler secret put`) and the Worker redeployed on the current `main`. Every
page now reads `main` at request time instead of the pack embedded at build time; the
"snapshot" banner is gone and the top bar says `live`.

Verified rather than assumed: the freshness stamp on the deployed dashboard tracked a commit
pushed **after** the running build's timestamp — only possible via a request-time read — and
all eleven routes return 200 with content, including the tree-driven listings (`/strategies`
lists all four variants), which exercise the Trees API and not just Contents.

Why it matters operationally: agents and Felix now see the firm's actual state, not the
state as of the last deploy. A stale dashboard was a real risk — the previous build was 5
minutes old and already missing a code change and three commits.

One caveat filed to Felix (`roles/felix/inbox/2026-07-25-github-token-scope.md`): the
provisioned token is a classic PAT with `repo` write scope on all his repositories, where
the dashboard needs only read-only Contents on this one. Working as-is; worth narrowing.

**Decided by:** Felix (asked for the redeploy); executed and verified in-session.

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
