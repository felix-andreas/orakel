# Dashboard

The human window into the firm (ARCHITECTURE.md §7): a Rust Cloudflare Worker
([workers-rs](https://github.com/cloudflare/workers-rs)) that server-renders the repo's
state as HTML. No external assets, no framework, no build step beyond `worker-build`.

## v3 — information architecture

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
| `/strategies/<family>` | FAMILY.md, the family's variants with status, family + variant scoring |
| `/strategies/<family>/<variant>` | STRATEGY.md, strategy.toml facts (status, trial clock, labels, success guideline), applications, results/worklog documents, the variant's predictions and scoring |
| `/strategies/<family>/<variant>?doc=<path>` | one of that variant's markdown documents (`results/*.md`, `memory/*.md`, `STRATEGY.md`) |
| `/markets/<slug>` | every prediction for one market over time (our probability vs market price vs the hourly R2 midpoint), the resolution, and per-row scoring |
| `/wiki/<page>` | one wiki page (`/wiki?page=` still works) |

`market_slug`, `family` and `variant` are links everywhere they appear — the prediction
log, run steps, applications, scoring aggregates and the snapshot book table.

There is **no page title**: the breadcrumb in the top bar is the title. The top bar
carries only things that do something — breadcrumbs, a light/dark toggle (persisted in
`localStorage`, respected by the charts) and the data-freshness stamp.

## Layout language

Four bands, in this order, and nothing else:

1. a row of compact **KPI cards** — label, big number, delta chip, one line of context;
2. a **dominant panel** (≈2/3) beside a **secondary list panel** (≈1/3);
3. **paired panels** — usually one chart, one dense table with inline mini-bars;
4. a compact table of recent activity.

Components live in `src/render.rs` and are deliberately few: `kpi_row`, `panel`, `table`,
`items`/`row`, `badge`/`chip`, `minibar`, `prose`. `src/style.css` holds the tokens (one
type scale, a 4px spacing rhythm, hairline borders, tabular figures on every number) and
both themes as first-class token blocks. **Labels are sentence case everywhere — there is
no `text-transform: uppercase` in the stylesheet.** When a rendered document's own `# `
heading would repeat the breadcrumb or panel title, it becomes the panel title instead of
being printed twice (`render::md_title` / `markdown_body`).

## Data: live reads, complete embedded fallback

- **GitHub API** (`src/live.rs`): file bodies via the Contents API
  (`Accept: application/vnd.github.raw+json`), all directory listings derived from ONE
  recursive Trees API call, plus the HEAD commit date for the "last updated" stamp.
  Responses are cached ~60s in the Workers Cache API keyed on the API URL (404s too).
  Requires the `GITHUB_TOKEN` worker secret (fine-grained PAT, read-only *Contents* on
  felix-andreas/orakel).
- **Embedded repo pack** (`build.rs` → `src/data.rs`): every renderable repo text file
  (`ops/`, `predictions/`, `ideas/`, `wiki/`, `strategies/` markdown + TOML + CSV, and
  `roles/*/inbox/*.md`) is concatenated into ONE file staged in `OUT_DIR` and pulled in
  with a single `include_str!` — ~95 files, ~290 KiB. Without the PAT the **whole**
  dashboard still renders from that snapshot (every page, including runs, strategies and
  ideas, which v2 could only show as empty states); the top bar says `snapshot` instead of
  `live` and a banner names the build. `data::tree()` falls back to the pack's file list,
  so directory discovery works offline too.
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
├── build.rs            # BUILD_TIMESTAMP + the embedded repo pack
└── src/
    ├── lib.rs          # router, page shell plumbing, static assets
    ├── render.rs       # layout (sidebar, breadcrumbs, theme toggle) + components
    ├── data.rs         # pack + live reads, CSV Table, repo discovery, dates
    ├── live.rs         # GitHub API with Cache API caching
    ├── snapshots.rs    # R2 book snapshots and series endpoints
    ├── overview.rs     # / dashboard, /execution
    ├── runs.rs         # /runs narrative
    ├── strategies.rs   # /strategies, family, variant, ?doc=
    ├── predictions.rs  # /predictions, /markets/<slug>
    ├── firm.rs         # /state /decisions /inboxes /wiki /ideas /snapshots
    ├── dev.rs          # /dev, /dev/endpoints, example JSON
    ├── style.css       # tokens + components, light and dark
    ├── charts.js       # SVG chart framework
    └── favicon.svg
```

Rendering is plain `format!` string building (CODING.md: procedural, no template engine).
Markdown → HTML via `pulldown-cmark`; TOML via the `toml` crate; CSV is a comma-split with
**column access by name** (`data::Table`), so an added column never shifts a page's data.

### Still planned

- Execution/backtest output on `/execution` once the engine lands (the nav slot and the
  page are already reserved).
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
`WASM_BINDGEN_BIN` first in restricted ones). Unoptimised output with the embedded pack is
~1.5 MiB raw / ~575 KiB gzipped — well inside the Workers limit.

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
  `npx wrangler secret put GITHUB_TOKEN`. Until it is set every page renders the embedded
  build-time pack and says so. NB: agent sessions cannot verify a PAT — the session proxy
  answers `api.github.com` with its own 403, so test the token from the deployed Worker or
  locally outside a session.
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
