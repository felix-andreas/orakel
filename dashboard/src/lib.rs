//! orakel dashboard — Cloudflare Worker (workers-rs), v1 skeleton.
//!
//! Server-rendered HTML over repo data EMBEDDED AT BUILD TIME (include_str!).
//! Each deploy snapshots repo state; live GitHub API + R2 reads come later
//! (see README.md). No external assets; core pages are JS-free (plain <a>
//! navigation, CSS-only tabs and burger menu). The /dev page additionally
//! loads /charts.js, our hand-rolled dependency-free chart framework.

mod render;

use render::{
    badge, badge_row, card, chip_row, csv_table, empty_state, esc, file_ref, kv, layout, markdown,
    note, parse_csv, section, subsection, tabs,
};
use worker::{event, Context, Env, Headers, Request, Response, Result, Router};

// ---------------------------------------------------------------------------
// Embedded repo snapshot. Paths are relative to THIS FILE (src/lib.rs).
// Specific known files only — no globs. Files that may not exist yet
// (predictions/scores.csv) are staged into OUT_DIR by build.rs so the crate
// always builds.
// ---------------------------------------------------------------------------

const STATE_TOML: &str = include_str!("../../ops/state.toml");
const DECISIONS_MD: &str = include_str!("../../ops/decisions.md");
const RUNS_README_MD: &str = include_str!("../../ops/runs/README.md");
const PREDICTIONS_CSV: &str = include_str!("../../predictions/predictions.csv");
const RESOLUTIONS_CSV: &str = include_str!("../../predictions/resolutions.csv");
const SCORES_CSV: &str = include_str!(concat!(env!("OUT_DIR"), "/scores.csv"));
const INBOX_CEO_MD: &str = include_str!("../../roles/ceo/inbox/README.md");
const INBOX_FELIX_MD: &str = include_str!("../../roles/felix/inbox/README.md");
const INBOX_MARKET_RESEARCHER_MD: &str =
    include_str!("../../roles/market-researcher/inbox/README.md");
const WIKI_INDEX_MD: &str = include_str!("../../wiki/index.md");
const CSS: &str = include_str!("style.css");
const CHARTS_JS: &str = include_str!("charts.js");
const FAVICON_SVG: &str = include_str!("favicon.svg");

/// Set by build.rs at compile time — never Date::now at runtime.
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

fn static_response(body: &str, content_type: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("content-type", content_type)?;
    headers.set("cache-control", "public, max-age=300")?;
    Ok(Response::ok(body)?.with_headers(headers))
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::from_html(page_operations()))
        .get("/predictions", |_, _| Response::from_html(page_predictions()))
        .get("/inboxes", |_, _| Response::from_html(page_inboxes()))
        .get("/wiki", |_, _| Response::from_html(page_wiki()))
        .get("/dev", |_, _| Response::from_html(page_dev()))
        .get("/dev/data/line.json", |_, _| {
            static_response(&dev_line_json(), "application/json; charset=utf-8")
        })
        .get("/dev/data/bar.json", |_, _| {
            static_response(&dev_bar_json(), "application/json; charset=utf-8")
        })
        .get("/style.css", |_, _| {
            static_response(CSS, "text/css; charset=utf-8")
        })
        .get("/charts.js", |_, _| {
            static_response(CHARTS_JS, "text/javascript; charset=utf-8")
        })
        .get("/favicon.svg", |_, _| {
            static_response(FAVICON_SVG, "image/svg+xml")
        })
        .run(req, env)
        .await
}

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

fn page_operations() -> String {
    // NB: parse as toml::Table — a document. (toml::Value::from_str parses a
    // single TOML *value* and rejects a document.)
    let state_html = match STATE_TOML.parse::<toml::Table>() {
        Ok(state) => state_sections(&state),
        Err(e) => empty_state(
            "Failed to parse ops/state.toml",
            &format!("<div class=\"mono\">{}</div>", esc(&e.to_string())),
        ),
    };

    let decisions_html = file_ref(
        "ops/decisions.md",
        "Append-only. Every structural change: what, why, who. Newest first.",
    ) + &format!("<div class=\"prose\">{}</div>", markdown(DECISIONS_MD));

    let runs_html = empty_state(
        "No run manifests yet",
        "<div>ops/runs/ contains no <span class=\"mono\">&lt;YYYY-MM-DD&gt;.toml</span> manifests in this build snapshot. The CEO writes one per orchestrated run.</div>",
    ) + &file_ref("ops/runs/README.md", "Manifest format reference.")
        + &format!("<div class=\"prose\">{}</div>", markdown(RUNS_README_MD));

    let body = tabs(
        "ops",
        &[
            ("State", state_html),
            ("Decisions", decisions_html),
            ("Runs", runs_html),
        ],
    );

    layout(
        "/",
        "Operations",
        "Current operating state (ops/state.toml), decision log, and run manifests.",
        &body,
        BUILD_TIMESTAMP,
    )
}

fn page_predictions() -> String {
    // predictions.csv
    let rows = parse_csv(PREDICTIONS_CSV);
    let predictions_html = if rows.len() <= 1 {
        let header = rows.first().cloned().unwrap_or_default();
        empty_state(
            "No predictions yet",
            &format!(
                "<div>predictions/predictions.csv holds only its header in this build snapshot. Rows appear once research slots start producing signals.</div>{}",
                chip_row(&header)
            ),
        )
    } else {
        let n = rows.len() - 1;
        format!(
            "<p class=\"page-desc\" style=\"margin:0\">{} prediction{}</p>{}",
            n,
            if n == 1 { "" } else { "s" },
            csv_table(&rows)
        )
    };

    // resolutions.csv
    let res_rows = parse_csv(RESOLUTIONS_CSV);
    let resolutions_html = if res_rows.len() <= 1 {
        let header = res_rows.first().cloned().unwrap_or_default();
        empty_state(
            "No resolutions yet",
            &format!(
                "<div>predictions/resolutions.csv is appended by the CEO when a market resolves.</div>{}",
                chip_row(&header)
            ),
        )
    } else {
        csv_table(&res_rows)
    };

    // scores.csv — generated by scoring/, may not exist yet (build.rs stages
    // an empty placeholder so this compiles either way).
    let score_rows = parse_csv(SCORES_CSV);
    let scores_html = if score_rows.len() <= 1 {
        empty_state(
            "No scores yet",
            "<div>predictions/scores.csv has not been generated — <span class=\"mono\">scoring/</span> runs once the first predictions resolve. It will be picked up automatically at the next build.</div>",
        )
    } else {
        csv_table(&score_rows)
    };

    let body = tabs(
        "predictions",
        &[
            ("Predictions", predictions_html),
            ("Resolutions", resolutions_html),
            ("Scores", scores_html),
        ],
    );

    layout(
        "/predictions",
        "Predictions",
        "The canonical append-only prediction log, resolutions, and scoring aggregates.",
        &body,
        BUILD_TIMESTAMP,
    )
}

fn page_inboxes() -> String {
    let inboxes: [(&str, &str, &str); 3] = [
        ("CEO", "roles/ceo/inbox/", INBOX_CEO_MD),
        ("Felix", "roles/felix/inbox/", INBOX_FELIX_MD),
        (
            "Market researcher",
            "roles/market-researcher/inbox/",
            INBOX_MARKET_RESEARCHER_MD,
        ),
    ];

    let mut cards = String::new();
    for (role, path, readme) in inboxes.iter() {
        cards.push_str(&card(
            role,
            path,
            &format!(
                "<div class=\"prose\">{}</div>{}",
                markdown(readme),
                note("No messages in this build snapshot — only the inbox README is embedded. Real message files will render here once the dashboard reads the repo live."),
            ),
        ));
    }

    let body = format!(
        "<div class=\"grid-cards\" style=\"grid-template-columns:repeat(auto-fill,minmax(340px,1fr))\">{}</div>",
        cards
    );

    layout(
        "/inboxes",
        "Inboxes",
        "Per-role message queues (roles/<role>/inbox/). Researcher and executor inboxes are created per slot / per live variant.",
        &body,
        BUILD_TIMESTAMP,
    )
}

fn page_wiki() -> String {
    let body = file_ref(
        "wiki/index.md",
        "Durable, cross-strategy knowledge. Maintained by the market researcher.",
    ) + &format!(
        "<div class=\"prose\">{}</div>{}",
        markdown(WIKI_INDEX_MD),
        note("v1 skeleton renders the index only — links to individual wiki pages are not routed yet; read them in the repo. Full wiki browsing arrives with live GitHub reads."),
    );

    layout(
        "/wiki",
        "Wiki",
        "Index of the firm's knowledge base.",
        &body,
        BUILD_TIMESTAMP,
    )
}

// ---------------------------------------------------------------------------
// /dev — pre-production playground (charts framework)
// ---------------------------------------------------------------------------

fn page_dev() -> String {
    let charts_html = file_ref(
        "dashboard/src/charts.js",
        "Hand-rolled, dependency-free SVG charts, themed by the shadcn tokens (dark mode for free). This framework will render predictions/scores/backtests once real series exist.",
    ) + &section(
        "Line — market probability",
        "<p class=\"muted-line\">Drag horizontally to zoom into a range; double-click to reset; hover for values.</p><div class=\"chart\" id=\"chart-line\"></div>",
    ) + &section(
        "Bar — Brier score by variant",
        "<p class=\"muted-line\">Hover a bar for the exact value. Lower is better.</p><div class=\"chart\" id=\"chart-bar\"></div>",
    ) + &note("Example series only — deterministic data generated in the Worker (fixed seed, never Date::now), served from /dev/data/*.json.")
        + r#"<script src="/charts.js"></script>
<script>
(function () {
  function load(url, fn) {
    fetch(url).then(function (r) { return r.json(); }).then(fn)
      .catch(function (e) {
        document.querySelectorAll(".chart").forEach(function (c) {
          if (!c.firstChild) c.textContent = "failed to load " + url;
        });
      });
  }
  load("/dev/data/line.json", function (d) {
    Chart.line(document.getElementById("chart-line"), d);
  });
  load("/dev/data/bar.json", function (d) {
    Chart.bar(document.getElementById("chart-bar"), d);
  });
})();
</script>"#;

    let endpoints_html = file_ref(
        "dashboard/src/lib.rs",
        "JSON endpoints backing the charts — deterministic example data until real series are wired up.",
    ) + &kv(
        "GET /dev/data/line.json",
        "<span class=\"mono\">{label, points:[{t, v}]}</span> — t = unix ms · <a href=\"/dev/data/line.json\">open</a>",
    ) + &kv(
        "GET /dev/data/bar.json",
        "<span class=\"mono\">{label, bars:[{label, v}]}</span> · <a href=\"/dev/data/bar.json\">open</a>",
    ) + &kv(
        "GET /charts.js",
        "<span class=\"mono\">Chart.line(el, data, opts)</span> / <span class=\"mono\">Chart.bar(el, data, opts)</span> · <a href=\"/charts.js\">open</a>",
    );

    let body = tabs(
        "dev",
        &[
            ("Charts", charts_html),
            ("Endpoints", endpoints_html),
        ],
    );

    layout(
        "/dev",
        "Development",
        "Pre-production playground — new dashboard features live here before graduating to the real pages.",
        &body,
        BUILD_TIMESTAMP,
    )
}

// Deterministic example data (CONSTITUTION: reproducible; never Date::now or
// OS randomness — a fixed-seed LCG so every deploy serves identical JSON).

/// Numerical-Recipes LCG → uniform f64 in [0, 1).
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*seed >> 33) as f64 / (1u64 << 31) as f64
}

/// 120 days of a plausible market probability: mean-reverting random walk
/// with occasional news jumps. Shape matches future real data: ms timestamps.
fn dev_line_json() -> String {
    const START_MS: u64 = 1_772_323_200_000; // 2026-03-01T00:00:00Z, fixed
    const DAY_MS: u64 = 86_400_000;
    let mut seed: u64 = 42;
    let mut v: f64 = 0.34;
    let mut points = String::new();
    for i in 0..120u64 {
        let r1 = lcg(&mut seed);
        let r2 = lcg(&mut seed);
        let jump = if r2 > 0.97 { (r1 - 0.5) * 0.22 } else { 0.0 };
        v += (0.5 - v) * 0.01 + (r1 - 0.5) * 0.035 + jump;
        v = v.clamp(0.02, 0.98);
        if i > 0 {
            points.push(',');
        }
        points.push_str(&format!(
            "{{\"t\":{},\"v\":{:.3}}}",
            START_MS + i * DAY_MS,
            v
        ));
    }
    format!(
        "{{\"label\":\"example — daily close, p(YES)\",\"points\":[{}]}}",
        points
    )
}

/// Plausible per-variant Brier scores (lower is better).
fn dev_bar_json() -> String {
    let variants = [
        "weather/nyc-temp-v1",
        "weather/nyc-temp-v2",
        "crypto/btc-close-v1",
        "politics/approval-v1",
        "sports/nba-totals-v1",
        "macro/cpi-nowcast-v1",
    ];
    let mut seed: u64 = 7;
    let mut bars = String::new();
    for (i, name) in variants.iter().enumerate() {
        if i > 0 {
            bars.push(',');
        }
        bars.push_str(&format!(
            "{{\"label\":\"{}\",\"v\":{:.3}}}",
            name,
            0.09 + lcg(&mut seed) * 0.17
        ));
    }
    format!(
        "{{\"label\":\"example — Brier score by variant\",\"bars\":[{}]}}",
        bars
    )
}

// ---------------------------------------------------------------------------
// ops/state.toml → flat State tab (stat strip + flat subsections, no cards)
// ---------------------------------------------------------------------------

fn stat(label: &str, value_html: &str) -> String {
    format!(
        "<div class=\"stat\"><div class=\"stat-label\">{}</div><div class=\"stat-value\">{}</div></div>",
        esc(label),
        value_html
    )
}

fn state_sections(state: &toml::Table) -> String {
    // Headline stats — the three numbers Felix actually checks.
    let phase = str_at(state, &["firm", "phase"]).unwrap_or("?");
    let tone = if phase == "operating" { "ok" } else { "warn" };
    let updated = str_at(state, &["firm", "updated"]).unwrap_or("?");
    let total = int_at(state, &["research", "slots_total"]).unwrap_or(0);
    let active = int_at(state, &["research", "slots_active"]).unwrap_or(0);
    let stats = format!(
        "<div class=\"stats\">{}{}{}</div>",
        stat("phase", &badge(phase, tone)),
        stat(
            "research slots",
            &format!("{}<small> / {} active</small>", active, total),
        ),
        stat(
            "state updated",
            &format!("<span class=\"mono\">{}</span>", esc(updated)),
        ),
    );

    let mut groups = String::new();

    // Research slots
    {
        let content = match arr_at(state, &["research", "slot"]) {
            Some(slots) if !slots.is_empty() => {
                let mut rows = String::new();
                for slot in slots {
                    let id = slot.get("id").and_then(|v| v.as_integer()).unwrap_or(0);
                    let variant = slot.get("variant").and_then(|v| v.as_str()).unwrap_or("?");
                    let started =
                        slot.get("trial_started").and_then(|v| v.as_str()).unwrap_or("?");
                    let due = slot
                        .get("trial_review_due")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    rows.push_str(&kv(
                        &format!("slot {}", id),
                        &format!(
                            "<span class=\"mono\">{}</span> · {} → review {}",
                            esc(variant),
                            esc(started),
                            esc(due)
                        ),
                    ));
                }
                rows
            }
            _ => "<p class=\"muted-line\">No trials running — slots are filled by the CEO from the ideas/ backlog.</p>".to_string(),
        };
        groups.push_str(&subsection("Research slots", &content));
    }

    // Strategies (family/variant, by status)
    {
        let mut content = String::new();
        for (key, tone) in [("live", "ok"), ("trial", "warn"), ("retired", "")] {
            let names = str_list(state, &["strategies", key]);
            let value = if names.is_empty() {
                "<span class=\"muted-line\">—</span>".to_string()
            } else {
                badge_row(&names.iter().map(|n| badge(n, tone)).collect::<Vec<_>>())
            };
            content.push_str(&kv(key, &value));
        }
        groups.push_str(&subsection("Strategies", &content));
    }

    // Cadence
    {
        let ceo = str_at(state, &["cadence", "ceo_trigger"]).unwrap_or("?");
        let additional = str_list(state, &["cadence", "additional_triggers"]);
        let additional_html = if additional.is_empty() {
            "<span class=\"muted-line\">none</span>".to_string()
        } else {
            badge_row(&additional.iter().map(|t| badge(t, "")).collect::<Vec<_>>())
        };
        groups.push_str(&subsection(
            "Cadence",
            &(kv("ceo trigger", &esc(ceo)) + &kv("additional", &additional_html)),
        ));
    }

    // Model routing (per-task defaults, CONSTITUTION.md §4)
    {
        let mut content = String::new();
        if let Some(models) = table_at(state, &["models"]) {
            for (role, model) in models {
                content.push_str(&kv(
                    role,
                    &format!(
                        "<span class=\"mono\">{}</span>",
                        esc(model.as_str().unwrap_or("?"))
                    ),
                ));
            }
        }
        groups.push_str(&subsection("Model routing", &content));
    }

    // Active roles
    {
        let roles = str_list(state, &["roles", "active"]);
        let content = if roles.is_empty() {
            "<span class=\"muted-line\">—</span>".to_string()
        } else {
            badge_row(&roles.iter().map(|r| badge(r, "")).collect::<Vec<_>>())
        } + "<p class=\"muted-line\" style=\"margin-top:.6rem\">Researchers/executors are instantiated per slot / per live variant.</p>";
        groups.push_str(&subsection("Active roles", &content));
    }

    // Secrets (presence only — never values)
    {
        let mut content = String::new();
        if let Some(secrets) = table_at(state, &["secrets"]) {
            for (name, v) in secrets {
                let val = v.as_str().unwrap_or("?");
                let b = if val == "missing" {
                    badge("missing", "bad")
                } else {
                    badge(val, "ok")
                };
                content.push_str(&kv(name, &b));
            }
        }
        groups.push_str(&subsection("Secrets", &content));
    }

    // Cloudflare
    {
        let mut content = String::new();
        if let Some(cf) = table_at(state, &["cloudflare"]) {
            for (name, v) in cf {
                let val = v.as_str().unwrap_or("?");
                let html = if name == "mcp" && val == "connected" {
                    badge(val, "ok")
                } else {
                    format!("<span class=\"mono\">{}</span>", esc(val))
                };
                content.push_str(&kv(name, &html));
            }
        }
        groups.push_str(&subsection("Cloudflare", &content));
    }

    stats
        + &format!("<div class=\"grid-flat\">{}</div>", groups)
        + &note(&format!(
            "Snapshot of <span class=\"mono\">ops/state.toml</span> as of this deploy (built {}). A structural change not reflected there didn't happen.",
            esc(BUILD_TIMESTAMP)
        ))
}

// ---------------------------------------------------------------------------
// toml::Value access helpers
// ---------------------------------------------------------------------------

fn value_at<'a>(root: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let mut cur = root.get(*first)?;
    for key in rest {
        cur = cur.get(key)?;
    }
    Some(cur)
}

fn str_at<'a>(v: &'a toml::Table, path: &[&str]) -> Option<&'a str> {
    value_at(v, path)?.as_str()
}

fn int_at(v: &toml::Table, path: &[&str]) -> Option<i64> {
    value_at(v, path)?.as_integer()
}

fn arr_at<'a>(v: &'a toml::Table, path: &[&str]) -> Option<&'a Vec<toml::Value>> {
    value_at(v, path)?.as_array()
}

fn table_at<'a>(v: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Table> {
    value_at(v, path)?.as_table()
}

fn str_list(v: &toml::Table, path: &[&str]) -> Vec<String> {
    arr_at(v, path)
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}
