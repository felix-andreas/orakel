//! `/dev` — the pre-production playground.
//!
//!   /dev            the chart framework, on deterministic example data
//!   /dev/endpoints  every JSON endpoint the dashboard serves
//!
//! New dashboard features live here before graduating to the real pages.

use crate::render::{badge, esc, section, section_foot, table};
use crate::{shell, trail};
use worker::Env;

pub async fn charts(env: &Env) -> String {
    let body = format!(
        "{line}{bar}{note}",
        line = section_foot(
            "Line",
            "market probability over time — drag horizontally to zoom, double-click to reset",
            &badge("Chart.line", ""),
            r#"<div class="chart" id="chart-line"></div>"#,
            "<span>multi-series and dot-mode series are supported: <span class=\"mono\">{series:[{label,points,mode}]}</span></span><a href=\"/dev/data/line.json\">line.json →</a>"
        ),
        bar = section_foot(
            "Bar",
            "Brier score by variant — hover a bar for the exact value, lower is better",
            &badge("Chart.bar", ""),
            r#"<div class="chart" id="chart-bar"></div>"#,
            "<span>bars carry a tone: <span class=\"mono\">ok</span> / <span class=\"mono\">bad</span> / neutral, and negative values draw below a zero baseline</span><a href=\"/dev/data/bar.json\">bar.json →</a>"
        ),
        note = format!(
            "{}{}",
            crate::render::note(
                "Example series only — deterministic data generated in the Worker (fixed seed, never Date::now), served from <span class=\"mono\">/dev/data/*.json</span>. Colours and fonts are read from the CSS tokens at render time, so both themes and the theme toggle work without a redraw path of their own.",
            ),
            r#"<script src="/charts.js"></script>
<script>
(function () {
  function load(url, fn) {
    fetch(url).then(function (r) { return r.json(); }).then(fn)
      .catch(function () {
        document.querySelectorAll(".chart").forEach(function (c) {
          if (!c.firstChild) c.textContent = "failed to load " + url;
        });
      });
  }
  load("/dev/data/line.json", function (d) {
    Chart.line(document.getElementById("chart-line"), d);
  });
  load("/dev/data/bar.json", function (d) {
    Chart.bar(document.getElementById("chart-bar"), d, {yPrecision:3});
  });
})();
</script>"#
        ),
    );

    shell(
        env,
        "/dev",
        trail(&[("Development", ""), ("Charts", "")]),
        true,
        &body,
    )
    .await
}

pub async fn endpoints(env: &Env) -> String {
    let rows = vec![
        row(
            "GET /dev/data/line.json",
            "{label, points:[{t, v}]} — t = unix ms",
            "deterministic example series",
            "/dev/data/line.json",
        ),
        row(
            "GET /dev/data/bar.json",
            "{label, bars:[{label, v}]}",
            "deterministic example bars",
            "/dev/data/bar.json",
        ),
        row(
            "GET /snapshots/data/&lt;date&gt;/&lt;slug&gt;.json",
            "{label, points:[{t, v}]}",
            "one day of hourly midpoints from R2, cached 5 minutes",
            "",
        ),
        row(
            "GET /data/market-series/&lt;slug&gt;.json",
            "{label, points:[{t, v}]}",
            "the last three days of midpoints for one market, used by /markets/&lt;slug&gt;",
            "",
        ),
        row(
            "GET /execution/data/&lt;set&gt;/v&lt;n&gt;.json",
            "{series:[{label, color, points:[{t, v}]}]}",
            "one equity curve per execution policy, for one signal set and policy version",
            "/execution/data/ladder-rv-hist/v2.json",
        ),
        row(
            "GET /table.js",
            "click a header to sort; third click restores the server's order",
            "sorting for any table.data.sortable, no dependencies",
            "/table.js",
        ),
        row(
            "GET /charts.js",
            "Chart.line(el, data, opts) · Chart.bar(el, data, opts)",
            "the chart framework itself, no dependencies",
            "/charts.js",
        ),
        row(
            "GET /style.css",
            "one tokenised stylesheet",
            "design tokens, components, light and dark",
            "/style.css",
        ),
    ];

    let body = format!(
        "{}{}",
        section_foot(
            "JSON and asset endpoints",
            "everything the Worker serves that is not a page",
            &badge(&crate::render::count(rows.len(), "endpoint"), ""),
            &table(
                &[("Endpoint", ""), ("Shape", ""), ("What it is", "wrap"), ("", "")],
                &rows,
            ),
            "<span class=\"mono\">dashboard/src/lib.rs</span><span>pages are server-rendered HTML; JSON exists only for charts</span>"
        ),
        section(
            "Routes",
            "the page routes behind the sidebar",
            "",
            &table(
                &[("Route", ""), ("Page", "")],
                &[
                    vec!["/".into(), "Dashboard — headline numbers, model vs market, strategies, scoring, coverage, runs".into()],
                    vec!["/runs".into(), "Daily runs — every manifest as a narrative".into()],
                    vec!["/execution".into(), "Execution — the policy × signal-set matrix, equity curves, caveats".into()],
                    vec!["/execution?fees=v1".into(), "The same matrix priced with NO venue fee (superseded)".into()],
                    vec!["/execution?doc=summary".into(), "The engine's full write-up (?doc=design for the accounting rules)".into()],
                    vec!["/strategies".into(), "Families and their variants".into()],
                    vec!["/strategies/&lt;family&gt;".into(), "Family thesis, variants, scoring".into()],
                    vec!["/strategies/&lt;family&gt;/&lt;variant&gt;".into(), "Strategy, facts, applications, predictions, worklog".into()],
                    vec!["/ideas".into(), "The market researcher's backlog".into()],
                    vec!["/predictions".into(), "The prediction log, scores, resolutions".into()],
                    vec!["/markets/&lt;slug&gt;".into(), "One market over time".into()],
                    vec!["/snapshots".into(), "R2 order-book snapshots".into()],
                    vec!["/state".into(), "ops/state.toml".into()],
                    vec!["/decisions".into(), "ops/decisions.md".into()],
                    vec!["/inboxes".into(), "Per-role message queues".into()],
                    vec!["/wiki".into(), "Knowledge base index and pages".into()],
                ],
            ),
        ),
    );

    shell(
        env,
        "/dev/endpoints",
        trail(&[("Development", ""), ("Endpoints", "")]),
        true,
        &body,
    )
    .await
}

fn row(endpoint: &str, shape: &str, what: &str, href: &str) -> Vec<String> {
    vec![
        format!("<span class=\"mono\">{endpoint}</span>"),
        format!("<span class=\"mono muted\">{shape}</span>"),
        what.to_string(),
        if href.is_empty() {
            String::new()
        } else {
            format!("<a class=\"link\" href=\"{}\">open</a>", esc(href))
        },
    ]
}

// ---------------------------------------------------------------------------
// Deterministic example data (CONSTITUTION: reproducible; never Date::now or
// OS randomness — a fixed-seed LCG so every deploy serves identical JSON).
// ---------------------------------------------------------------------------

/// Numerical-Recipes LCG → uniform f64 in [0, 1).
fn lcg(seed: &mut u64) -> f64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*seed >> 33) as f64 / (1u64 << 31) as f64
}

/// 120 days of a plausible market probability: mean-reverting random walk with
/// occasional news jumps. Shape matches real data: ms timestamps.
pub fn line_json() -> String {
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
        points.push_str(&format!("{{\"t\":{},\"v\":{:.3}}}", START_MS + i * DAY_MS, v));
    }
    format!("{{\"label\":\"example — daily close, p(YES)\",\"points\":[{points}]}}")
}

/// Plausible per-variant Brier scores (lower is better).
pub fn bar_json() -> String {
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
    format!("{{\"label\":\"example — Brier score by variant\",\"bars\":[{bars}]}}")
}
