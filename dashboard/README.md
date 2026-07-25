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
| Overview | **Dashboard** `/` · **Daily runs** `/runs` · **Execution** `/execution` |
| Research | **Strategies** `/strategies` · **Ideas** `/ideas` · **Predictions** `/predictions` |
| Data | **Snapshots** `/snapshots` |
| Firm | **State** `/state` · **Decisions** `/decisions` · **Inboxes** `/inboxes` · **Wiki** `/wiki` |
| Development | **Charts** `/dev` · **Endpoints** `/dev/endpoints` |

Detail routes hang off those and light up their parent nav item:

| Route | What it shows |
|---|---|
| `/execution?fees=v1` | the same matrix priced with **no venue fee** (superseded, kept for attribution) |
| `/execution?doc=summary` \| `?doc=design` | the engine's own write-up / the accounting rules, rendered whole |
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
  recursive Trees API call, plus the HEAD commit date for the "last updated" stamp.
  Responses are cached ~60s in the Workers Cache API keyed on the API URL (404s too).
  Requires the `GITHUB_TOKEN` worker secret (fine-grained PAT, read-only *Contents* on
  felix-andreas/orakel).
- **No fallback copy.** Until 2026-07-25 `build.rs` also concatenated every renderable
  repo file (~170 files, ~1.0 MiB) into the binary, and the Worker served that whenever
  GitHub was unreachable. It was removed: an outage then looked like a working dashboard
  quietly showing outdated numbers, which is the worst failure this thing can have — and
  the pack was 45% of the bundle. `main` at request time is now the only source of truth.
  A failed read yields empty text, the top bar reads `cannot read repo` instead of a
  timestamp, and a red banner names the cause (no token / 401 / 403 rate limit / 5xx).
  Transient failures (network, 5xx) are retried once — a retry costs one subrequest and no
  staleness, unlike serving an old copy. 401/403/404 are definitive and never retried.
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

- **eight** series colours (`--chart-1..8`), because the execution page draws one line per
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
    ├── execution.rs    # /execution, its ?doc= views and the equity-curve JSON
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

### The execution page

`/execution` reads `execution/results/summary.csv` (one row per signal set × policy ×
policy version) and `execution/README.md` (the policies' plain-English characters). It
never recomputes a metric.

- The headline is **annualized return on locked capital**, not cents per trade
  (DESIGN.md §3) — both are shown, and the two leaders disagreeing is the finding.
- **v1 is fee-free, v2 charges the venue's real taker fee.** The version is a visible
  switch, each row prints the other version's number and the gap in percentage points,
  the cost model is quoted in the engine's own words, and choosing v1 raises a banner.
- Sample size sits under every headline number, and the engine's `n < 30` rule is
  respected: underpowered rows are shown, labelled and **not ranked** (DESIGN.md §7).
- Caveats are generated from the same CSV (synthetic fills, one regime, `patient`'s
  dropped 24h-later observations, peak deployment above bankroll), so they cannot drift
  from the numbers.
- Equity curves come from `execution/results/<set>/<policy>-v<n>.json` via
  `/execution/data/<set>/v<n>.json`, one `Chart.line` series per policy.

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
