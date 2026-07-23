# Dashboard

The human window into the firm (ARCHITECTURE.md §7): a Rust Cloudflare Worker
([workers-rs](https://github.com/cloudflare/workers-rs)) that server-renders the repo's
state as HTML. No external assets, and the core pages are JS-free: plain `<a>`
navigation, CSS-only tabs (hidden radio inputs + labels wired by `nth-of-type` pair
rules; `tabs` helper in `render.rs`) and a CSS-only burger menu on mobile (checkbox
hack). The `/dev` page is the one exception: it loads `/charts.js`, our hand-rolled
dependency-free SVG chart framework (`Chart.line` / `Chart.bar`, brush-zoom, tooltips,
ResizeObserver responsiveness, colors read from the CSS tokens at render time). It is
served by the Worker like `style.css` — still no external assets, no build step.

Styling is hand-written CSS at shadcn/ui (v4, zinc) fidelity, responsive down to phone
widths. Tailwind was considered and deliberately skipped: the build is pure
Rust/`worker-build` (no npm build step to add or break in proxy-restricted sessions),
and utility classes inside Rust `format!` strings are harder to maintain than one
tokenized stylesheet. If the UI ever outgrows this, revisit — the requirement is
shadcn-quality UI, not a particular toolchain. Layout philosophy: mostly flat content
under section headings with generous whitespace; cards only where boxed grouping earns
its border (see `section`/`subsection` vs `card` in `render.rs`).

## v1 skeleton: data is embedded at BUILD TIME

**Caveat:** this version does not fetch anything at runtime. Repo files are baked into the
wasm binary via `include_str!` — every deploy is a snapshot of the repo at build time, and
the dashboard goes stale until the next deploy. That is deliberate for the skeleton: the
CEO deploys after each run anyway, so the snapshot tracks the daily cadence.

Embedded files (paths are relative to `src/lib.rs`, so `../../ops/...` — specific known
files only, never globs):

| Page | Source files |
|------|-------------|
| `/` Operations | `ops/state.toml` (parsed with `toml`), `ops/decisions.md`, `ops/runs/README.md` |
| `/predictions` | `predictions/predictions.csv`, `predictions/resolutions.csv`, `predictions/scores.csv` (optional, see below) |
| `/inboxes` | `roles/{ceo,felix,market-researcher}/inbox/README.md` |
| `/wiki` | `wiki/index.md` |
| `/dev` | none — playground page; charts fetch `/dev/data/*.json` |
| `/dev/data/line.json`, `/dev/data/bar.json` | generated in `src/lib.rs` — deterministic example series (fixed-seed LCG, fixed start timestamp; never `Date::now`) |
| `/style.css` | `src/style.css` |
| `/charts.js` | `src/charts.js` |
| `/favicon.svg` | `src/favicon.svg` (sea shell, scheme-aware stroke via embedded `@media`) |

`predictions/scores.csv` does not exist until `scoring/` first runs, so `build.rs` stages
it into `OUT_DIR` (empty placeholder when missing) — the crate builds either way and picks
the file up automatically once it exists. Run manifests (`ops/runs/*.toml`) and real inbox
messages are **not** embedded (that would be a glob); the pages render a note instead.

The build timestamp in the footer is a compile-time constant emitted by `build.rs`
(`BUILD_TIMESTAMP`) — never `Date::now()` at runtime.

### Planned v2: live data

Replace the embedded snapshot with runtime reads, keeping the same pages:

- **GitHub API** (server-side `fetch` + Workers cache) for markdown/TOML/CSV — needs
  `GITHUB_TOKEN` as a Worker secret; enables run manifests, inbox messages, and full wiki
  browsing without redeploys.
- **R2 binding** (native, no credentials in code) on bucket `orakel` for big data:
  dataset snapshots, backtest outputs.
- Live Polymarket prices server-side; htmx partial swaps + ECharts for track-record
  charts.

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

`wrangler.toml`'s build command already falls back to `--no-opt` automatically when the
plain `worker-build --release` fails, so `npx wrangler deploy` works in both restricted
and unrestricted environments (just export `WASM_BINDGEN_BIN` first in restricted ones).
`--no-opt` only skips the binaryen size pass (~785 KiB unoptimized upload, ~295 KiB
gzipped — well under Workers limits).

Two gotchas baked into the config, do not undo them:

- **No `strip = true` in `[profile.release]`** — stripping removes the wasm
  `target_features` custom section and wasm-bindgen then fails with
  "externref table required for catch wrappers".
- **`ops/state.toml` is parsed as `toml::Table`**, not `toml::Value` — with the `toml` 1.x
  crate, `Value::from_str` parses a single value and rejects a document.

## Deploy

```sh
cd dashboard
CLOUDFLARE_API_TOKEN=... npx wrangler deploy
```

- The token needs the *Edit Cloudflare Workers* permission template. It is **not** in the
  repo; presence is tracked in `ops/state.toml` `[secrets]`.
- `wrangler.toml` runs `worker-build` as the build command, so `npx wrangler deploy` is
  the whole pipeline (wrangler is fetched by npx, no `package.json` needed).
- First deploy prints the `*.workers.dev` URL.
- Local preview: `npx wrangler dev --port 8787` then curl/browse `http://127.0.0.1:8787`
  (verified: all routes server-render; `workerd` is fetched from npm).

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

The Worker itself needs no secrets in v1 (nothing is fetched at runtime).

## Layout

```
dashboard/
├── Cargo.toml          # cdylib crate: worker, toml, pulldown-cmark
├── wrangler.toml       # name=orakel-dashboard, build via worker-build
├── build.rs            # BUILD_TIMESTAMP + optional-file staging (scores.csv)
└── src/
    ├── lib.rs          # router + pages + embedded data + /dev example data
    ├── render.rs       # esc/layout/section/card/tabs/table/markdown helpers (format!-based)
    ├── style.css       # shadcn v4 tokens (zinc, dark mode), tabs + burger, chart tokens
    ├── charts.js       # dependency-free SVG chart framework (Chart.line/Chart.bar)
    └── favicon.svg     # sea-shell icon, light/dark aware
```

Rendering is plain `format!` string building (CODING.md: procedural, no template engine).
Markdown → HTML via `pulldown-cmark`; `ops/state.toml` parsed with the `toml` crate; CSV
parsing is a trivial comma-split (our canonical CSVs never quote fields).
