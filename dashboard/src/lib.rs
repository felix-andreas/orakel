//! orakel dashboard — Cloudflare Worker (workers-rs), v3.
//!
//! Data path: LIVE reads at request time — repo files from the GitHub API
//! (src/live.rs, 60s Cache API caching) and hourly book snapshots from R2
//! (src/snapshots.rs). **There is no fallback copy of the repo.** Earlier
//! builds compiled every renderable file into the Worker and served that when
//! GitHub was unreachable; the result was that an outage looked like a working
//! dashboard quietly showing outdated numbers, which is the worst failure this
//! thing can have. A read that fails now renders as an error, with the reason,
//! and the top bar says `cannot read repo` instead of a timestamp.
//!
//! Information architecture (src/render.rs NAV):
//!   Overview     Dashboard · Daily runs · Backtest · Paper book
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

mod backtest;
mod book;
mod data;
mod dev;
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
    // Resolve which commit we are serving ONCE, before any page reads a file,
    // so every read on this request is pinned to the same SHA and none of them
    // can trigger a refresh mid-page. See live::begin_request.
    live::begin_request(&env).await;
    Router::new()
        // --- Overview ---
        .get_async("/", |_, ctx| async move {
            Response::from_html(overview::page(&ctx.env).await)
        })
        .get_async("/runs", |_, ctx| async move {
            Response::from_html(runs::page(&ctx.env).await)
        })
        .get_async("/backtest", |req, ctx| async move {
            // v2 (the venue's real taker fee) is the default; v1 is fee-free and
            // must be asked for explicitly.
            let version = match query(&req, "fees").as_deref() {
                Some("v1") | Some("1") => 1,
                _ => 2,
            };
            Response::from_html(
                backtest::page(&ctx.env, version, query(&req, "tab"), query(&req, "doc")).await,
            )
        })
        .get_async("/backtest/data/:set/:file", |_, ctx| async move {
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
                &backtest::equity_json(&ctx.env, &set, version).await,
                "application/json; charset=utf-8",
            )
        })
        // `/execution` is the live paper book — holdings, plan and apply
        // (src/book.rs). The BACKTEST was called "Execution" until 2026-07-26,
        // which claimed something the firm does not do (it places no orders,
        // CONSTITUTION.md §5), so it moved to /backtest and left this address
        // free. Old backtest deep links still have to work, and they are
        // identifiable: `?fees=` is a backtest parameter and the book will never
        // use it. So the route disambiguates on it — that branch stays forever,
        // everything else is the book. 302, not 301: a permanent redirect is
        // cached by browsers for a very long time and this rename is a week old.
        // (The leaf `/execution/data/…` below is pure backtest and redirects
        // unconditionally.)
        .get_async("/execution", |req, ctx| async move {
            let mut url = req.url()?;
            if url.query_pairs().any(|(k, _)| k == "fees") {
                url.set_path("/backtest");
                return Response::redirect_with_status(url, 302);
            }
            Response::from_html(book::page(&ctx.env, &url).await)
        })
        .get_async("/execution/data/:set/:file", |req, ctx| async move {
            let set = ctx.param("set").cloned().unwrap_or_default();
            let file = ctx.param("file").cloned().unwrap_or_default();
            let mut url = req.url()?;
            url.set_path(&format!("/backtest/data/{set}/{file}"));
            Response::redirect_with_status(url, 302)
        })
        // --- Research ---
        .get_async("/strategies", |_, ctx| async move {
            Response::from_html(strategies::index(&ctx.env).await)
        })
        .get_async("/strategies/:family", |req, ctx| async move {
            let family = ctx.param("family").cloned().unwrap_or_default();
            let tab = query(&req, "tab");
            Response::from_html(strategies::family(&ctx.env, &family, tab).await)
        })
        .get_async("/strategies/:family/:variant", |req, ctx| async move {
            let family = ctx.param("family").cloned().unwrap_or_default();
            let variant = ctx.param("variant").cloned().unwrap_or_default();
            // Legacy deep link: `?doc=<path>` rendered one of the variant's
            // documents on its own page. Those documents now live in tabs, so
            // the old address redirects to the tab that holds the document
            // (and to its anchor, when the tab holds several).
            if let Some(rel) = query(&req, "doc") {
                let (tab, anchor) = strategies::doc_target(&rel);
                let mut url = req.url()?;
                let q = if tab.is_empty() { None } else { Some(format!("tab={tab}")) };
                url.set_query(q.as_deref());
                url.set_fragment(if anchor.is_empty() { None } else { Some(&anchor) });
                return Response::redirect(url);
            }
            let tab = query(&req, "tab");
            Response::from_html(strategies::variant(&ctx.env, &family, &variant, tab).await)
        })
        .get_async("/ideas", |_, ctx| async move {
            Response::from_html(firm::ideas(&ctx.env).await)
        })
        .get_async("/ideas/:slug", |_, ctx| async move {
            let slug = ctx.param("slug").cloned().unwrap_or_default();
            Response::from_html(firm::idea_page(&ctx.env, &slug).await)
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
        .get_async("/inboxes/:role/:stem", |_, ctx| async move {
            let role = ctx.param("role").cloned().unwrap_or_default();
            let stem = ctx.param("stem").cloned().unwrap_or_default();
            Response::from_html(firm::inbox_page(&ctx.env, &role, &stem).await)
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
/// page made; the stamp is the repo's last commit time. There is no fallback
/// source, so when a read fails the page is showing an incomplete picture and
/// has to say so — `reason` carries what GitHub actually told us.
pub async fn freshness(env: &Env, live: bool) -> Freshness {
    let head = live::head(env).await;
    let reason = match &head {
        Ok(_) if live => None,
        // HEAD is reachable but some read on this page was not, so the cause is
        // per-file. Name the files: "part of this page" is not a diagnosis, and
        // it cost real time to track down the one request in ~30 that hit it.
        Ok(_) => {
            let failed = live::failed_paths();
            Some(match failed.len() {
                0 => "A file this page needs could not be read from the repository.".to_string(),
                1 => format!("Could not read {} from the repository.", failed[0]),
                n => format!(
                    "Could not read {n} files from the repository: {}.",
                    failed.join(", ")
                ),
            })
        }
        Err(e) => Some(e.clone()),
    };
    Freshness {
        live: live && head.is_ok(),
        stamp: head.map(|h| render::fmt_ts(&h.date)).unwrap_or_default(),
        build: BUILD_TIMESTAMP.to_string(),
        reason,
    }
}

/// Standard page: breadcrumbs + body, with the freshness resolved.
pub async fn shell(env: &Env, active: &str, crumbs: Vec<Crumb>, live: bool, body: &str) -> String {
    shell_sub(env, active, crumbs, live, "", body).await
}

/// Same, plus a secondary bar under the top bar — the detail page's own tabs
/// (`render::tabbar`). `subbar` is already-safe HTML.
pub async fn shell_sub(
    env: &Env,
    active: &str,
    crumbs: Vec<Crumb>,
    live: bool,
    subbar: &str,
    body: &str,
) -> String {
    let fresh = freshness(env, live).await;
    render::layout(active, &crumbs, subbar, body, &fresh)
}

/// Shown at the top of a page that could not read everything it needed.
///
/// The dashboard has no second copy of the repo to fall back to — by design,
/// since a stale copy served silently is worse than a visible gap. So this
/// banner is an error, not a notice: it says what is missing and why.
pub fn fetch_error_banner(reason: &str) -> String {
    format!(
        "<div class=\"banner banner-bad\">{} <strong>Some of this page is missing.</strong> {} \
         Nothing below is a cached or older copy — the dashboard reads <span class=\"mono\">main</span> \
         at request time and shows only what it could read.</div>",
        render::icon("alert"),
        render::esc(reason)
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
