# Dashboard

The human window into the firm (ARCHITECTURE.md §7): a Rust Cloudflare Worker
([workers-rs](https://github.com/cloudflare/workers-rs)) that server-renders the repo's
state as HTML. No external assets, no framework, no build step beyond `worker-build`.

## v4 — information architecture

The sidebar is the map of the firm. Groups collapse and remember their state in
`localStorage` (per section); the group holding the active route is always expanded, and
without JS every group stays open.

| Section | Pages |
|---|---|
| Overview | **Dashboard** `/` · **Daily runs** `/runs` · **Backtest** `/backtest` · **Paper book** `/execution` |
| Research | **Strategies** `/strategies` · **Ideas** `/ideas` · **Predictions** `/predictions` |
| Data | **Snapshots** `/snapshots` |
| Firm | **State** `/state` · **Decisions** `/decisions` · **Inboxes** `/inboxes` · **Wiki** `/wiki` |
| Development | **Charts** `/dev` · **Endpoints** `/dev/endpoints` |

Detail routes hang off those and light up their parent nav item:

| Route | What it shows |
|---|---|
| `/backtest?tab=history` \| `?tab=method` | the other signal set / the accounting rules, as real addresses (default tab = no parameter) |
| `/backtest?fees=v1` | the same rows priced with **no venue fee** (superseded, kept for attribution) |
| `/backtest?doc=summary` \| `?doc=design` | the engine's own write-up / the accounting rules, rendered whole |
| `/execution` | the **paper book**, tabbed: Holdings (default) · Plan `?tab=plan` · Shares & NAV `?tab=shares`. The plan's whole selection is in the URL (`?s=`, `?m=`, `?p=`) |
| `/execution?fees=…`, `/execution/data/…` | **302** to the `/backtest` equivalent — the backtest was called Execution until 2026-07-26 and `fees` is a parameter the book never uses |
| `/strategies/<family>` | the family, **tabbed**: Overview (its plain-English line, its variants) · How it works (FAMILY.md) · Predictions (every variant's rows + family/variant scoring) |
| `/strategies/<family>/<variant>` | one strategy, **tabbed**: Overview (`explainer`, facts, applications) · How it works (STRATEGY.md) · Results (`results/*.md`) · Predictions · Logs (WORKLOG + MEMORY) |
| `/strategies/…?tab=<key>` | the tab, as a real address. `how-it-works` \| `results` \| `predictions` \| `logs`; the default tab carries no parameter, so the bare URL is canonical |
| `/strategies/<family>/<variant>?doc=<path>` | legacy deep link — **302** to the tab that now holds the document (`results/x.md` → `?tab=results#x`) |
| `/markets/<slug>` | every prediction for one market over time (our probability vs market price vs the hourly R2 midpoint), the resolution, and per-row scoring |
| `/wiki/<page>` | one wiki page (`/wiki?page=` still works) |

`market_slug`, `family` and `variant` are links everywhere they appear — the prediction
log, run steps, applications, scoring aggregates and the snapshot book table.

There is **no page title**: the breadcrumb in the top bar is the title. The top bar
carries only things that do something — breadcrumbs, the data-freshness stamp and (below
900px) the burger.

**Every setting lives in one place**: a **settings popover** at the bottom-left of the
sidebar, built on the native Popover API (`popover` attribute + `popovertarget`), so
opening, light-dismiss, Esc and focus handling are the browser's. It holds theme
(**light / dark / system**, system being the default — with no explicit choice there is no
`data-theme` attribute at all and the stylesheet's `prefers-color-scheme` block decides),
a **density** switch (comfortable / compact, which retunes four spacing tokens at once),
expand/collapse all sidebar sections, and links out. Theme and density are persisted in
`localStorage` and applied by a tiny inline script before first paint; charts re-render on
both. Without JavaScript the popover still opens and says the controls need JS — the page
then follows the system scheme, which is the default anyway.

Below 900px the top bar is the **only** header. The sidebar stops being a band: its
wordmark row is dropped and the nav opens as an overlay hanging off the bar
(`position: fixed`), filling the **dynamic** viewport (`100dvh - var(--topbar-h)`) so
mobile browser chrome can neither clip it nor leave it short; the settings control stays
pinned to its bottom. With the menu open the breadcrumb hands its slot to the `orakel`
wordmark — navigation has replaced the page's context, so the app's name is what belongs
there. One band, 48px, in every state.

## Layout language

**Cards are the exception, not the container** (PRINCIPLES.md). There is no generic
bordered "panel" component: a page is a headline stat strip and then **sections** — a
heading, a hairline, content — separated by space. Lists are hairline-separated rows, so
five things read as five lines rather than five boxes. **No bordered surface may contain
another one**; the only bordered surfaces left are things that float (settings popover,
chart tooltip) or warn (the snapshot banner). This is asserted programmatically in
verification: no element with a full border + radius, ≥120×44px, may contain another.

A detail page with more than one kind of content gets a **secondary bar** — `render::tabbar`
in `layout`'s subbar slot, a sibling of the top bar on the same gutters, sticky under it at
three quarters its height. Every tab is a real link (`?tab=<key>`, default tab = no
parameter) and the server renders that tab's content, so a fresh load, a bookmark and the
back button all behave; nothing is hidden client-side. **A tab that would be empty is not
rendered.** The active tab is marked by weight and a rule under it, never by dimming the
others; the bar scrolls sideways when five tabs do not fit at 375px, so the page never
does. The breadcrumb names the tab (`… / ladder-rv / Results`) and the tab's own content
carries no repeated heading.

The **lede** (`.lede`) is the plain-English paragraph that describes a thing — a strategy's
`explainer`, a family's `> **In plain English:**` opener, which is lifted out of FAMILY.md
rather than printed twice. It is larger than body text, set to a ~72-character measure, and
sits above every number. When it is missing the page says which required field is missing
instead of quietly showing less.

Components live in `src/render.rs` and are deliberately few: `stat_grid`/`stat_line`,
`section`/`section_foot`, `tabbar`, `table`/`table_sortable`, `items`/`row`, `badge`/`chip`,
`minibar`, `notes`, `doc`, `prose`. `src/style.css` holds the tokens (one type scale, a
4px spacing rhythm, hairline borders, tabular figures on every number), both themes as
first-class token blocks, and the four density tokens (`--gap`, `--pad-page`, `--cell-y`,
`--rowsp`) that `[data-density="compact"]` retunes. **Labels are sentence case everywhere
— there is no `text-transform: uppercase` in the stylesheet.** When a rendered document's
own `# ` heading would repeat the breadcrumb or section title, it becomes the section
title instead of being printed twice (`render::md_title` / `markdown_body`).

## Data: live reads, no fallback

- **GitHub API** (`src/live.rs`): file bodies via the Contents API
  (`Accept: application/vnd.github.raw+json`), all directory listings derived from ONE
  recursive Trees API call. Requires the `GITHUB_TOKEN` worker secret (fine-grained PAT,
  read-only *Contents* on felix-andreas/orakel).
- **Reads are pinned to a commit SHA, and issued concurrently.** Two properties carry the
  page's speed, and both matter:
  1. `live::head()` resolves which commit `main` is at (memoised per isolate for 60s —
     that TTL *is* the dashboard's freshness knob). Every file and tree read then goes to
     `?ref=<sha>`, so its URL names one immutable blob and can cache for a day. Before
     this, every entry expired on a 60-second clock whether or not anything had changed,
     so the first visitor each minute re-fetched the whole page from GitHub.
  2. Independent reads go out together (`data::read_all` / `futures::join!`). A page's
     reads do not decide each other — only the tree has to land before we know which
     variant and run files to fetch — so it is two waves, not twenty steps.

  Measured on `/` (~20 reads): **0.87s → 0.41s** warm, and the 2.9s cache-miss spike is
  gone. Latency used to scale linearly with the number of reads; now it barely moves
  (`/decisions`, 1 read, is 0.22s; `/`, 20 reads, is 0.41s).
- **No fallback copy.** Until 2026-07-25 `build.rs` also concatenated every renderable
  repo file (~170 files, ~1.0 MiB) into the binary, and the Worker served that whenever
  GitHub was unreachable. It was removed: an outage then looked like a working dashboard
  quietly showing outdated numbers, which is the worst failure this thing can have — and
  the pack was 45% of the bundle. `main` at request time is now the only source of truth.
  A failed read yields empty text, the top bar reads `cannot read repo` instead of a
  timestamp, and a red banner names the cause (no token / 401 / 403 rate limit / 5xx).
  Transient failures (network, 5xx) are retried once — a retry costs one subrequest and no
  staleness, unlike serving an old copy. 401/403/404 are definitive and never retried.
- **KNOWN, UNFIXED: pages lose reads on the first request after a push.** The
  "Some of this page is missing" banner appears over content that is perfectly fine,
  naming a few files that exist and are committed. Measured 2026-07-28: it fires on the
  **first request after a push** and in **none of 120 requests made outside that window**.
  The firm pushes constantly during a run, which is when Felix is most likely to be looking.

  Two hypotheses were tried and **both are disproved** — recorded so nobody spends the
  afternoon again:

  | tried | result |
  |---|---|
  | SHA-propagation race (`head()` learns a commit one replica hasn't got), patched with an unpinned retry at `main` | **No.** The failure reason is `pinned no response, unpinned no response` — no status at all, so the subrequest never happened. A rejected ref returns a status. The retry also *costs* two subrequests per failure, and was reverted. |
  | Burst concurrency, patched by capping in-flight reads at 6 | **No, and it made things worse** — `/runs` went from 3 lost files to 6 while `/` stayed at 2. Do not bound `read_all`; see its doc comment. |

  What survives: a **per-request subrequest budget** that only a cold cache can exhaust.
  Cache hits don't spend it; a push changes the SHA, so every pinned URL changes and every
  read on the page becomes a real subrequest at once.

  **The fix is fewer reads per page, not different reads.** `/runs` reads every manifest
  and `/` reads ~20 files, and both grow as the firm accumulates history — so this gets
  worse on its own. Candidates: read only the N most recent manifests, or derive more from
  the single Trees call. Needs a dashboard cycle.

  Failures record *why* (`path (pinned no response)`), which is the only reason any of the
  above is known. Keep that.
- **R2 binding `ORAKEL`** (`src/snapshots.rs`): hourly
  `snapshots/books/<YYYY-MM-DD>/<HH>.json.gz` objects. Binding gets return the stored
  gzipped bytes verbatim (the `content_encoding=gzip` metadata is HTTP-layer only), so
  bytes are sniffed for the 0x1f,0x8b magic and gunzipped (flate2 rust_backend). Two
  series endpoints, both Cache-API'd for 5 minutes:
  `/snapshots/data/<date>/<slug>.json` (one day) and `/data/market-series/<slug>.json`
  (last three days, used by `/markets/<slug>`).

The build timestamp in the footer is a compile-time constant from `build.rs`
(`BUILD_TIMESTAMP`) — never `Date::now()` at runtime.

## Charts

`src/charts.js` is our own dependency-free SVG framework, served like `style.css`:

```js
Chart.line(el, { series: [{ label, points: [{t, v, label}], mode, color }] }, opts)
Chart.bar (el, { bars: [{ label, v, tone }] }, opts)
// opts: { min, max, x: "time" | "index", yPrecision }
```

- **eight** series colours (`--chart-1..8`), because the backtest page draws one line per
  policy and there are eight policies;
- multi-series with per-series `mode` (`"line"` / `"dots"`) and a pinned palette `color`,
  so a series that is absent at render time never recolours the others (the legend is
  server-rendered from the same tokens, and would otherwise disagree);
- `x: "index"` for rank-ordered charts (the dashboard's model-vs-market panel);
- brush-zoom, double-click reset, nearest-point tooltips across all series;
- bars carry a tone (`ok`/`bad`) and negative values draw below a zero baseline;
- colours and fonts are read from the CSS custom properties at render time, and charts
  re-render on resize, on system scheme changes **and** on the theme toggle
  (`MutationObserver` on `data-theme`).

## Source layout

```
dashboard/
├── Cargo.toml          # cdylib crate: worker, toml, pulldown-cmark, serde_json, flate2
├── wrangler.toml       # name=orakel-dashboard, build via worker-build, R2 binding
├── build.rs            # BUILD_TIMESTAMP (nothing else — no embedded pack)
└── src/
    ├── lib.rs          # router, page shell plumbing, static assets
    ├── render.rs       # layout (sidebar, breadcrumbs, theme toggle) + components
    ├── data.rs         # live reads, CSV Table, repo discovery, dates
    ├── live.rs         # GitHub API with Cache API caching
    ├── snapshots.rs    # R2 book snapshots and series endpoints
    ├── overview.rs     # / dashboard
    ├── backtest.rs     # /backtest (tabbed), its ?doc= views and the equity-curve JSON
    ├── book.rs         # /execution — the paper book: Holdings, Plan, Apply
    ├── runs.rs         # /runs narrative
    ├── strategies.rs   # /strategies, family + variant (tabbed), ?doc= redirects
    ├── predictions.rs  # /predictions, /markets/<slug>
    ├── firm.rs         # /state /decisions /inboxes /wiki /ideas /snapshots
    ├── dev.rs          # /dev, /dev/endpoints, example JSON
    ├── style.css       # tokens + components, light and dark
    ├── charts.js       # SVG chart framework
    ├── table.js        # click-to-sort for table.data.sortable
    └── favicon.svg     # the sea-shell mark — render.rs inlines the same path
```

The **wordmark** is that mark plus the word `orakel`, nothing else. `favicon.svg` has to
hard-code its stroke colour (a browser tab icon inherits nothing); the inlined copy in
`render.rs` uses `currentColor` so it follows the theme. Change one, change the other.

Rendering is plain `format!` string building (CODING.md: procedural, no template engine).
Markdown → HTML via `pulldown-cmark`; TOML via the `toml` crate; CSV is parsed with
RFC4180 quoting and **column access by name** (`data::Table`), so an added column never
shifts a page's data — and `execution/results/summary.csv`, whose `fee_model` column is a
sentence full of commas, lines up with its header.

### The backtest page

Called **Backtest**, not Execution: the firm places no orders and never will
(CONSTITUTION.md §5), so a surface named after execution claimed something we do not do.
Every number on it is a replay of stored signals against stored prices. **The repo
directory keeps its name** — `execution/` is the engine's home and is referenced from
ARCHITECTURE.md, `ops/decisions.md`, several `strategy.toml` success guidelines and the
CEO's playbook — and every file path the page prints is the true one
(`execution/results/summary.csv`). The rename is presentation only.

`/backtest` reads `execution/results/summary.csv` (one row per signal set × policy ×
policy version) and `execution/README.md` (the policies' plain-English characters),
concurrently. It never recomputes a metric.

**One tab per signal set, plus method** — because "which policy is best" is meaningless
without saying *on whose signals* (DESIGN.md §2), and our two sets return opposite
verdicts. The tab order is the order a reader's questions arrive:

| Tab | Question it answers |
|---|---|
| **Our own signals** (default) | *Does any of this trade at all?* On `orakel-live`, **7 of the 8 policies take zero trades** — the most important result we have, and the reason this tab is the landing page. Being calibrated and being tradeable are different properties. |
| **Historical signals** `?tab=history` | *Where there ARE trades, which policy wins?* The ranked matrix over `ladder-rv-hist`, the fee before/after per row, and the equity curves. |
| **How it works** `?tab=method` | The capital-lockup rule, the fee model, and what the engine refuses to conclude. |

- The deciding metric is **annualized return on locked capital**, never cents per trade
  (DESIGN.md §3); both are shown and the two leaders disagreeing is the finding.
- **v1 is fee-free, v2 charges the venue's real taker fee.** v2 is the default and carries
  no parameter, the version is a visible switch that survives a tab change, each ranked row
  prints the other version's number and the gap in percentage points, the cost model is
  quoted in the engine's own words, and choosing v1 raises a banner.
- Sample size sits beside every number and the `n < 30` rule is respected: underpowered
  rows are shown, labelled and **not ranked** (DESIGN.md §7).
- **No t-statistic is shown.** Our own wiki says it answers the wrong question — the null
  that matters is the break-even win rate, not zero
  (`wiki/reference/break-even-win-rate.md`) — and SUMMARY.md says these particular ones are
  optimistic because repeated signals on a market share an outcome. Both facts are stated
  in the caveats instead; the engine's t-stats remain in `?doc=summary`.
- Caveats are generated from the same CSV (synthetic fills, one regime, sub-daily holds
  annualising by ~500×, `patient`'s dropped 24h-later observations, peak deployment above
  bankroll), so they cannot drift from the numbers.
- Equity curves come from `execution/results/<set>/<policy>-v<n>.json` via
  `/backtest/data/<set>/v<n>.json`, one `Chart.line` series per policy. **They are drawn
  only where more than one policy traded** — a single line from $1,000 to $1,002 is not a
  comparison, and vertical space is the scarcest thing on the page. Because the tab *is*
  the signal set, the old set `<select>` is gone.

### The paper book

`/execution` (`src/book.rs`) is the live counterpart of the backtest: not *what would have
happened*, but *what we hold now and what a policy would do about it next*. It is **paper**
— no wallet, no order, no venue (CONSTITUTION.md §5) — and the page says so in a banner on
every tab, in the nav label and in the breadcrumb, not only in a comment.

| Tab | Question it answers |
|---|---|
| **Holdings** (default) | *How much money is there, how much is committed, what is open, what is it worth?* Cash, free cash and committed as separate figures (`free = cash − Σ collateral`, so a fully deployed book cannot look cash-healthy), then one row per position, then the ledger. |
| **Plan** `?tab=plan` | *What would this policy do about today's signals, and what would it cost?* Configure → plan → apply; target positions minus the open ones, `terraform plan`-style. |
| **Shares & NAV** `?tab=shares` | *Several people pay into one book — who owns what, and what is a share worth?* NAV at both marks and the gap between them, shares outstanding, NAV per share ("Last"), the per-investor register and every contribution and redemption with the rate it was struck at. |

**Markets are shown as the ladders they are.** A slug like
`will-bitcoin-dip-to-42pt5k-in-july-2026-821` is one rung of a board — one asset over one
window, listed as a whole ladder of price levels — and 53 of them is not 53 ideas, it is
four boards. `parse_rung`/`boards_of` turn a slug into `BTC ≤ 42.5k` plus the plain-English
question behind it, group by (asset, window) and sort by barrier so the downside and upside
rungs read as one continuum. The label leads and the slug stays underneath, everywhere a
market is named on the page: nobody recognises a market by its slug, but the slug is what
the URL, the CSVs and the ledger rows carry.

- **Every position is marked twice.** At the CLOB midpoint and at the price we could
  actually get out at (bid for a long, ask for a short). A midpoint is not a fill
  (`wiki/reference/midpoint-is-not-a-fill.md`: 21/21 beat the market, 2/21 had a
  counterparty at the scored price), so the mid column is labelled an upper bound and the
  gap between the two columns is the most informative number on the page. Where the live
  book cannot price a position, `predictions/fills.csv`'s observed prices stand in, named.
- **Return on locked capital, never cents per trade** (`execution/DESIGN.md` §3), with the
  annualized figure beside it and "too young to annualize" under a day.
- **The ledger is the authority.** Cash is `starting_cash + Σ cash_delta`; if the ledger's
  collateral movements and `positions.csv` disagree the page raises a red banner and says
  to believe the ledger.
- **The empty book is a first-class view**, because it is today's real state and will be
  for a while: what the book is, that it is paper, that nothing is open, and a link to the
  Plan tab.
- **Plan is a pure function** and its whole selection lives in the URL (`?s=family/variant`
  repeated, `?m=slug` repeated, `?p=policy-file`), so a plan is linkable, reproducible and
  correct under the back button. The form is a plain GET form — no JavaScript.
- **The plan reads as configure → plan → apply.** Three numbered sections and a step strip
  that carries the *state* of each one (`fade-v2 · 1 strategy · 53 markets — all defaults,
  nothing to change` / `no changes — every one of 53 candidates was refused` / `nothing to
  append`), each a real anchor. It is allowed to exist only because it answers the page's
  question before any scrolling; a progress indicator carrying no state would be the
  decoration PRINCIPLES.md forbids.
- **The defaults are the plan.** Strategies and markets collapse behind a summary line
  saying what the default resolved to, so the reader can see at a glance that they need not
  touch either; the policy is the one control that is always open, because it is the choice
  that usually matters. A whole board is selected in one click — a link, so it is
  bookmarkable and the back button undoes it.
- **Gates are the engine's, in the engine's order** — `candidates()` mirrors
  `execution/engine/src/sim.rs::simulate` line for line (side, min edge, edge percentile,
  spread, depth, fundability, per-market cap, sizing, slippage, taker fee), so a candidate
  refused here would be refused there. Two live-only departures, both deliberate: a
  delayed policy (`patient`) acts on **the latest prediction that is already
  `delay_hours` old** rather than the newest one, since live the entry observation is the
  current book; and a candidate is refused when the book already holds **the other side**
  of that token, because a plan whose rows cancel each other out is not applicable.
- **A plan that proposes nothing is the important screen**, not a blank one: every refused
  candidate is listed with the rule that stopped it and the number that failed, tallied by
  rule at the top. On our own live signals seven of the eight policies take zero trades —
  this is where that stops being a statistic.
- **v1 policies raise a banner.** They are fee-free and exist only so the v1→v2 delta reads
  as the cost of the fee; the default is `fade-v2`, or `portfolio.toml`'s bound policy.
- **A plan is never silently truncated.** If it needs more collateral than there is free
  cash it is shown in full, with a red banner saying it does not fit.
- **Apply cannot write, and does not pretend to.** The dashboard is a read-only Worker with
  no commit path by design, so Apply renders the exact `ledger.csv` rows the plan implies
  plus the command to append them, and says why that is the better shape (the ledger is the
  audit trail). There is no fake success state. Rows carry a `plan_id` — an FNV-1a digest
  of the selection, the policy and the prices — so an applied trade traces back to the plan.

#### Shares, NAV and the "Last" price

Several people can pay into one book, so ownership is counted in **shares**, not in each
person's dollars (`portfolio/README.md`). NAV is `cash + Σ unrealised P&L` — cash already
includes money committed as collateral, because posting collateral commits money rather than
spending it, so nothing is double counted.

- **Issuance and redemption are struck at the conservative liquidation mark**, never the
  midpoint: the bid on a long, the ask on a short. With one account the mark is a reporting
  nicety; with several people it decides who gains at whose expense in both directions —
  issue at an inflated NAV and the new investor is diluted, redeem at one and the remaining
  holders fund it. The midpoint NAV sits beside it labelled as the optimistic bound and
  **the gap is displayed**, per position as well as in total, because that gap is the size
  of the transfer the wrong mark would make.
- **A position with no live two-sided book is counted, not hidden.** Its conservative figure
  falls back to a midpoint, which makes it not conservative, and the page says how many.
- **`shares_delta` is never recomputed.** It is taken from `investors.csv` exactly as
  written — it is the evidence for how many shares somebody got — and the check
  (`amount ÷ nav_per_share`) is shown against any row that disagrees rather than quietly
  repairing it.
- **The empty register is a first-class view**: what a share is, that the first contribution
  strikes at 1.0000 by definition, the exact first row that would be written, and a worked
  example of what issuing at a flattered NAV would cost the new investor. Nothing on the page
  accepts money and the page says so.
- **The opening balance has no owner, and the page says so.** `starting_cash` is a notional
  placeholder nobody paid in, so NAV is 10,000.00 against 0 shares; taken literally with the
  1.0000 first-contribution rule, the first contributor is handed the whole balance. A banner
  states the conflict with the live numbers and names the two one-line fixes. Raised with
  Felix in `roles/felix/inbox/2026-07-26-shares-opening-balance.md`.
- **The arithmetic is unit-tested** (`cargo test --lib`, `src/book.rs` `mod tests`): two
  contributions struck at different NAVs and a redemption, asserting that issuing at the true
  NAV moves nothing between holders, that the register sums to the book, and that the
  conservative NAV is below the midpoint one. These are pure functions over parsed tables, so
  they need no Worker.

### Still planned

- Live Polymarket prices server-side, beyond the hourly R2 snapshots.
- R2-backed backtest dataset browsing (manifests → tables/charts).

## Build

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build

cd dashboard
cargo check --target wasm32-unknown-unknown   # fast verification
worker-build --release                        # full pipeline → build/worker/shim.mjs
```

`worker-build` downloads pinned `wasm-bindgen`/`esbuild`/`wasm-opt` binaries on first run.
In proxy-restricted agent sessions the GitHub-release downloads (`wasm-bindgen`, binaryen's
`wasm-opt`) are blocked with 403; `esbuild` comes from npm and works. Workaround (verified
in such a session):

```sh
# version must exactly match the wasm-bindgen lib version in Cargo.lock
cargo install wasm-bindgen-cli --version 0.2.126
export WASM_BINDGEN_BIN=wasm-bindgen     # worker-build honors {NAME}_BIN overrides
worker-build --release --no-opt          # --no-opt skips the blocked wasm-opt
```

`wrangler.toml`'s build command already falls back to `--no-opt` automatically, so
`npx wrangler deploy` works in both restricted and unrestricted environments (export
`WASM_BINDGEN_BIN` first in restricted ones). Since the embedded pack was dropped the
deployed bundle is ~1.31 MiB raw / ~521 KiB gzipped, down from ~2.39 MiB / ~781 KiB —
well inside the Workers limit, which is on the compressed size.

Which downloads are blocked is not fixed: in an agent session on 2026-07-25 only
`wasm-bindgen` needed the workaround — with `WASM_BINDGEN_BIN` exported, worker-build's
`wasm-opt` download went through and the optimised pipeline ran end to end (1.59 MiB raw /
636 KiB gzipped at the time, when the pack was still embedded). Export `WASM_BINDGEN_BIN`
and let the fallback decide the rest.

Two gotchas baked into the config, do not undo them:

- **No `strip = true` in `[profile.release]`** — stripping removes the wasm
  `target_features` custom section and wasm-bindgen then fails with
  "externref table required for catch wrappers".
- **`ops/state.toml` is parsed as `toml::Table`**, not `toml::Value` — with the `toml` 1.x
  crate, `Value::from_str` parses a single value and rejects a document.

Local preview: `npx wrangler dev --port 8787`, then curl or browse
`http://127.0.0.1:8787`. `wrangler dev` simulates an empty R2 bucket, so `/snapshots`
renders its empty state and `/markets/<slug>` draws the prediction dots without the
midpoint line — use `--remote` to exercise the real bucket.

## Deploy

```sh
cd dashboard
CLOUDFLARE_API_TOKEN=... npx wrangler deploy
```

- The token needs the *Edit Cloudflare Workers* permission template. It is **not** in the
  repo; presence is tracked in `ops/state.toml` `[secrets]`.
- **Live repo reads need the `GITHUB_TOKEN` worker secret**:
  `npx wrangler secret put GITHUB_TOKEN` (**set 2026-07-25**, live reads verified). Without
  it every page renders its error banner and no content at all. `secret put` deploys a
  new version by itself — no rebuild needed to turn live reads on — but allow a few seconds
  before the change shows.
- **Verifying live reads** (the reliable check, since the word `live` also appears in
  page content): the top-bar indicator is `dot-ok</span>live · <stamp>` and `stamp` is the
  **HEAD commit time of `main`**, while the footer carries the build time. Live reads are
  proven when the stamp is *newer* than the footer's build stamp — a request-time read is
  the only way the Worker could know about a commit made after it was built.
- Agent sessions can partly verify a PAT after all: the session proxy blocks *repo*
  endpoints (`/repos/...` → its own 403 `"GitHub access is not enabled for this session"`,
  not GitHub's), but `/user` and `/rate_limit` pass through, so the token's identity and
  `X-OAuth-Scopes` are readable in-session. Contents/Trees access itself still has to be
  tested from the deployed Worker.
- `wrangler.toml` runs `worker-build` as the build command, so `npx wrangler deploy` is
  the whole pipeline (wrangler is fetched by npx, no `package.json` needed).
- **A deploy can ship the PREVIOUS build and report success.** Seen 2026-08-02: two
  consecutive deploys printed `Uploaded` with a fresh Version ID while the build step said
  `Finished in 0.06s` — cargo considered the wasm artifact fresh and rebuilt nothing, so the
  live Worker kept serving the old code. It cost an hour of concluding that correct fixes
  were wrong.

  **The only reliable invalidation is `rm -rf build target/wasm32-unknown-unknown`.**
  Everything cheaper was tried and does not work: `touch src/*.rs` prints
  `Compiling orakel-dashboard` and takes ~25s while still shipping the old wasm (it rebuilds
  the *host* target); `cargo clean -p orakel-dashboard` removed 505 files and the next build
  still finished in **0.06s**; deleting `build/` alone regenerates it from the stale artifact.

  **The footer build stamp is the check, and it is the only honest one.** `BUILD_TIMESTAMP`
  comes from `build.rs`, which re-runs only on a genuine rebuild — during the stale period the
  live page reported a stamp **seven days old** while wrangler reported a fresh Version ID.
  After deploying, fetch any page and confirm the stamp is now.
- First deploy prints the `*.workers.dev` URL.

### Cloudflare Access (do before sharing the URL)

The dashboard is private. Front it with Cloudflare Access (Zero Trust) so only Felix can
reach it:

1. Zero Trust dashboard → Access → Applications → *Add application* → Self-hosted.
2. Application domain: the worker's `workers.dev` subdomain (or a custom domain routed to
   the worker).
3. Policy: Allow → Include → Emails → Felix's email. Access sends a one-time PIN; no other
   identity provider setup needed.
4. Until Access is configured, the workers.dev URL is public — it exposes repo state
   (no secrets, but keep the window short).
