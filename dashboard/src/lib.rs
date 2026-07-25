//! orakel dashboard — Cloudflare Worker (workers-rs), v3.
//!
//! Data path: LIVE reads at request time — repo files from the GitHub API
//! (src/live.rs, 60s Cache API caching) and hourly book snapshots from R2
//! (src/snapshots.rs). Every repo file the dashboard can render is ALSO
//! embedded at build time (build.rs → the pack in src/data.rs), so without the
//! GITHUB_TOKEN secret the whole dashboard still renders — from the build's
//! snapshot, flagged as such in the top bar. It never shows an error page.
//!
//! Information architecture (src/render.rs NAV):
//!   Overview     Dashboard · Daily runs · Execution
//!   Research     Strategies · Ideas · Predictions
//!   Data         Snapshots
//!   Firm         State · Decisions · Inboxes · Wiki
//!   Development  Charts · Endpoints
//! Detail routes hang off those: /strategies/<family>[/<variant>], /markets/<slug>,
//! /wiki/<page>.
//!
//! No external assets. Client JS is limited to: sidebar persistence + the
//! settings popover's theme/density choices (inline, in the shell), /charts.js
//! on pages that draw charts and /table.js on pages with a sortable table.

mod data;
mod dev;
mod execution;
mod firm;
mod live;
mod overview;
mod predictions;
mod render;
mod runs;
mod snapshots;
mod strategies;

use render::{crumb, Crumb, Freshness};
use worker::{event, Context, Env, Headers, Request, Response, Result, Router};

const CSS: &str = include_str!("style.css");
const CHARTS_JS: &str = include_str!("charts.js");
const TABLE_JS: &str = include_str!("table.js");
const FAVICON_SVG: &str = include_str!("favicon.svg");

/// Set by build.rs at compile time — the build stamp is always compile-time.
pub const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

fn static_response(body: &str, content_type: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("content-type", content_type)?;
    headers.set("cache-control", "public, max-age=300")?;
    Ok(Response::ok(body)?.with_headers(headers))
}

/// First value of a query parameter.
fn query(req: &Request, key: &str) -> Option<String> {
    let url = req.url().ok()?;
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        // --- Overview ---
        .get_async("/", |_, ctx| async move {
            Response::from_html(overview::page(&ctx.env).await)
        })
        .get_async("/runs", |_, ctx| async move {
            Response::from_html(runs::page(&ctx.env).await)
        })
        .get_async("/execution", |req, ctx| async move {
            // v2 (the venue's real taker fee) is the default; v1 is fee-free and
            // must be asked for explicitly.
            let version = match query(&req, "fees").as_deref() {
                Some("v1") | Some("1") => 1,
                _ => 2,
            };
            Response::from_html(execution::page(&ctx.env, version, query(&req, "doc")).await)
        })
        .get_async("/execution/data/:set/:file", |_, ctx| async move {
            let set = ctx.param("set").cloned().unwrap_or_default();
            let file = ctx.param("file").cloned().unwrap_or_default();
            let version = file
                .strip_suffix(".json")
                .and_then(|s| s.strip_prefix('v'))
                .and_then(|s| s.parse::<u32>().ok());
            let Some(version) = version else {
                return Response::error("bad request", 400);
            };
            static_response(
                &execution::equity_json(&ctx.env, &set, version).await,
                "application/json; charset=utf-8",
            )
        })
        // --- Research ---
        .get_async("/strategies", |_, ctx| async move {
            Response::from_html(strategies::index(&ctx.env).await)
        })
        .get_async("/strategies/:family", |_, ctx| async move {
            let family = ctx.param("family").cloned().unwrap_or_default();
            Response::from_html(strategies::family(&ctx.env, &family).await)
        })
        .get_async("/strategies/:family/:variant", |req, ctx| async move {
            let family = ctx.param("family").cloned().unwrap_or_default();
            let variant = ctx.param("variant").cloned().unwrap_or_default();
            let doc = query(&req, "doc");
            Response::from_html(strategies::variant(&ctx.env, &family, &variant, doc).await)
        })
        .get_async("/ideas", |_, ctx| async move {
            Response::from_html(firm::ideas(&ctx.env).await)
        })
        .get_async("/predictions", |_, ctx| async move {
            Response::from_html(predictions::page(&ctx.env).await)
        })
        .get_async("/markets/:slug", |_, ctx| async move {
            let slug = ctx.param("slug").cloned().unwrap_or_default();
            Response::from_html(predictions::market(&ctx.env, &slug).await)
        })
        // --- Data ---
        .get_async("/snapshots", |_, ctx| async move {
            Response::from_html(firm::snapshots(&ctx.env).await)
        })
        .get_async("/snapshots/data/:date/:file", |req, ctx| async move {
            let date = ctx.param("date").cloned().unwrap_or_default();
            let file = ctx.param("file").cloned().unwrap_or_default();
            let Some(slug) = file.strip_suffix(".json") else {
                return Response::error("bad request", 400);
            };
            let url = req.url()?;
            snapshots::series_json(&ctx.env, &date, slug, url.as_str()).await
        })
        .get_async("/data/market-series/:file", |req, ctx| async move {
            let file = ctx.param("file").cloned().unwrap_or_default();
            let Some(slug) = file.strip_suffix(".json") else {
                return Response::error("bad request", 400);
            };
            let url = req.url()?;
            snapshots::market_series_json(&ctx.env, slug, url.as_str()).await
        })
        // --- Firm ---
        .get_async("/state", |_, ctx| async move {
            Response::from_html(firm::state(&ctx.env).await)
        })
        .get_async("/decisions", |_, ctx| async move {
            Response::from_html(firm::decisions(&ctx.env).await)
        })
        .get_async("/inboxes", |_, ctx| async move {
            Response::from_html(firm::inboxes(&ctx.env).await)
        })
        .get_async("/wiki", |req, ctx| async move {
            // Legacy deep links: /wiki?page=<path>
            match query(&req, "page") {
                Some(p) => Response::from_html(firm::wiki_page(&ctx.env, &p).await),
                None => Response::from_html(firm::wiki(&ctx.env).await),
            }
        })
        .get_async("/wiki/:a", |_, ctx| async move {
            let a = ctx.param("a").cloned().unwrap_or_default();
            Response::from_html(firm::wiki_page(&ctx.env, &a).await)
        })
        .get_async("/wiki/:a/:b", |_, ctx| async move {
            let a = ctx.param("a").cloned().unwrap_or_default();
            let b = ctx.param("b").cloned().unwrap_or_default();
            Response::from_html(firm::wiki_page(&ctx.env, &format!("{a}/{b}")).await)
        })
        // --- Development ---
        .get_async("/dev", |_, ctx| async move {
            Response::from_html(dev::charts(&ctx.env).await)
        })
        .get_async("/dev/endpoints", |_, ctx| async move {
            Response::from_html(dev::endpoints(&ctx.env).await)
        })
        .get("/dev/data/line.json", |_, _| {
            static_response(&dev::line_json(), "application/json; charset=utf-8")
        })
        .get("/dev/data/bar.json", |_, _| {
            static_response(&dev::bar_json(), "application/json; charset=utf-8")
        })
        // --- static assets ---
        .get("/style.css", |_, _| {
            static_response(CSS, "text/css; charset=utf-8")
        })
        .get("/charts.js", |_, _| {
            static_response(CHARTS_JS, "text/javascript; charset=utf-8")
        })
        .get("/table.js", |_, _| {
            static_response(TABLE_JS, "text/javascript; charset=utf-8")
        })
        .get("/favicon.svg", |_, _| {
            static_response(FAVICON_SVG, "image/svg+xml")
        })
        .run(req, env)
        .await
}

// ---------------------------------------------------------------------------
// Shared page plumbing
// ---------------------------------------------------------------------------

/// Freshness indicator for the top bar. `live` is the AND of every read the
/// page made; the stamp is the repo's last commit time when live, otherwise
/// the build stamp (which is exactly what the embedded pack is from).
pub async fn freshness(env: &Env, live: bool) -> Freshness {
    // No reachable HEAD ⇒ no live reads happened, whatever the caller thinks.
    let head = live::head_commit_date(env).await;
    let live = live && head.is_some();
    Freshness {
        live,
        stamp: render::fmt_ts(&head.unwrap_or_else(|| BUILD_TIMESTAMP.to_string())),
        build: BUILD_TIMESTAMP.to_string(),
    }
}

/// Standard page: breadcrumbs + body, with the freshness resolved.
pub async fn shell(env: &Env, active: &str, crumbs: Vec<Crumb>, live: bool, body: &str) -> String {
    let fresh = freshness(env, live).await;
    render::layout(active, &crumbs, body, &fresh)
}

/// Shown at the top of a page whose data came from the build-time pack.
pub fn snapshot_banner() -> String {
    format!(
        "<div class=\"banner\">{} Live GitHub reads are unavailable — showing the repo snapshot embedded at build time ({}). Set the GITHUB_TOKEN worker secret to read main at request time.</div>",
        render::icon("clock"),
        render::esc(BUILD_TIMESTAMP)
    )
}

/// Breadcrumb trail helper: section → page [→ detail…].
pub fn trail(parts: &[(&str, &str)]) -> Vec<Crumb> {
    parts.iter().map(|(label, href)| crumb(label, href)).collect()
}

/// A JSON string literal, safe to inline inside a `<script>` block.
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
