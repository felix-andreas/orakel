//! HTML rendering: the page shell (sidebar + top bar) and the small set of
//! components every page is built from. Plain `format!` string building, no
//! template engine (CODING.md: procedural, data in → HTML out).
//!
//! Component vocabulary — nothing else exists, on purpose:
//!   kpi_row     a row of compact stat cards (label, number, delta, context)
//!   panel       a bordered surface with a header (title, subtitle, chip)
//!   table       a dense data table (per-column class drives alignment)
//!   items/row   compact lists inside panels
//!   badge/chip  status and metadata
//!   minibar     inline proportional bar for dense tables
//!   prose       rendered markdown
//!
//! Labels are SENTENCE CASE everywhere. No uppercasing, in code or CSS.

// ---------------------------------------------------------------------------
// Escaping, markdown
// ---------------------------------------------------------------------------

/// Escape text for safe inclusion in HTML bodies and attribute values.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// The document's leading `# ` heading, if it has one. Panels use it as their
/// title so the same words aren't printed twice (breadcrumb, panel header and
/// then an H1 in the prose was exactly the "repeated page title" problem).
pub fn md_title(src: &str) -> Option<String> {
    let first = src.lines().find(|l| !l.trim().is_empty())?;
    first.strip_prefix("# ").map(|t| t.trim().to_string())
}

/// Markdown → HTML with the leading `# ` heading removed. Use wherever the
/// surrounding panel or breadcrumb already names the document.
pub fn markdown_body(src: &str) -> String {
    let trimmed = src.trim_start_matches(['\r', '\n', ' ', '\t']);
    match md_title(src) {
        Some(_) => markdown(
            trimmed
                .split_once('\n')
                .map(|(_, r)| r)
                .unwrap_or("")
                .trim_start_matches(['\r', '\n']),
        ),
        None => markdown(src),
    }
}

/// Markdown → HTML via pulldown-cmark (tables + strikethrough on).
pub fn markdown(src: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(src, opts);
    let mut out = String::with_capacity(src.len() * 2);
    html::push_html(&mut out, parser);
    out
}

// ---------------------------------------------------------------------------
// Icons — a closed set of 16px stroke glyphs. An icon exists only where it
// speeds up recognition (nav items, KPI cards, the theme toggle).
// ---------------------------------------------------------------------------

pub fn icon(name: &str) -> String {
    let body = match name {
        "gauge" => r#"<path d="M12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"/><path d="m13.4 10.6 3.6-3.6"/><path d="M3.05 13a9 9 0 1 1 17.9 0"/>"#,
        "calendar" => r#"<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M8 3v4M16 3v4M3 10h18"/>"#,
        "play" => r#"<circle cx="12" cy="12" r="9"/><path d="m10 8.5 6 3.5-6 3.5z"/>"#,
        "flask" => r#"<path d="M9 3v6.5L4.3 17A2 2 0 0 0 6 20h12a2 2 0 0 0 1.7-3L15 9.5V3"/><path d="M8 3h8M7.5 14h9"/>"#,
        "bulb" => r#"<path d="M9 18h6M10 21h4"/><path d="M12 3a6 6 0 0 0-3.5 10.9c.5.4.8 1 .9 1.6h5.2c.1-.6.4-1.2.9-1.6A6 6 0 0 0 12 3Z"/>"#,
        "list" => r#"<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>"#,
        "database" => r#"<ellipse cx="12" cy="6" rx="8" ry="3"/><path d="M4 6v6c0 1.7 3.6 3 8 3s8-1.3 8-3V6"/><path d="M4 12v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/>"#,
        "inbox" => r#"<path d="M21 12H16l-2 3h-4l-2-3H3"/><path d="M5.5 5.5 3 12v6a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-6l-2.5-6.5A2 2 0 0 0 16.6 4H7.4a2 2 0 0 0-1.9 1.5Z"/>"#,
        "gavel" => r#"<path d="M14 4 20 10M17 7l-7 7M4 20l6-6M8 10l6 6"/>"#,
        "book" => r#"<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z"/>"#,
        "chart" => r#"<path d="M3 3v18h18"/><path d="m7 14 3-4 3 3 4-6"/>"#,
        "plug" => r#"<path d="M9 2v6M15 2v6"/><path d="M6 8h12v3a6 6 0 0 1-12 0V8Z"/><path d="M12 17v5"/>"#,
        "settings" => r#"<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-2.7 1.1V21a2 2 0 1 1-4 0v-.1A1.6 1.6 0 0 0 7.5 19l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1A1.6 1.6 0 0 0 3 13.6H3a2 2 0 1 1 0-4h.1A1.6 1.6 0 0 0 4.7 7l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.6 1.6 0 0 0 2.7-1.1V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 2.7 1.1l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0 1.1 2.7H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1.3Z"/>"#,
        "target" => r#"<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1.4"/>"#,
        "trend-up" => r#"<path d="m3 17 6-6 4 4 8-8"/><path d="M15 7h6v6"/>"#,
        "layers" => r#"<path d="m12 2 9 5-9 5-9-5 9-5Z"/><path d="m3 12 9 5 9-5"/><path d="m3 17 9 5 9-5"/>"#,
        "bolt" => r#"<path d="M13 2 4 14h7l-1 8 9-12h-7l1-8Z"/>"#,
        "clock" => r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3.5 2"/>"#,
        "check" => r#"<path d="m5 13 4 4 10-10"/>"#,
        "sun" => r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>"#,
        "moon" => r#"<path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z"/>"#,
        "chevron" => r#"<path d="m6 9 6 6 6-6"/>"#,
        "burger" => r#"<path d="M4 6h16M4 12h16M4 18h16"/>"#,
        "external" => r#"<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M21 14v5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5"/>"#,
        "arrow-left" => r#"<path d="M19 12H5"/><path d="m12 19-7-7 7-7"/>"#,
        _ => r#"<circle cx="12" cy="12" r="9"/>"#,
    };
    format!(
        r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">{body}</svg>"#
    )
}

fn icon_sized(name: &str, size: u32) -> String {
    icon(name).replacen(
        r#"width="14" height="14""#,
        &format!(r#"width="{size}" height="{size}""#),
        1,
    )
}

// ---------------------------------------------------------------------------
// Navigation tree — the information architecture, in one place.
// ---------------------------------------------------------------------------

pub struct NavLink {
    pub href: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    /// Route prefixes that also light this item up (detail views).
    pub owns: &'static [&'static str],
}

pub struct NavGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub links: &'static [NavLink],
}

pub const NAV: &[NavGroup] = &[
    NavGroup {
        id: "overview",
        label: "Overview",
        links: &[
            NavLink { href: "/", label: "Dashboard", icon: "gauge", owns: &[] },
            NavLink { href: "/runs", label: "Daily runs", icon: "calendar", owns: &["/runs/"] },
            NavLink { href: "/execution", label: "Execution", icon: "play", owns: &[] },
        ],
    },
    NavGroup {
        id: "research",
        label: "Research",
        links: &[
            NavLink {
                href: "/strategies",
                label: "Strategies",
                icon: "flask",
                owns: &["/strategies/"],
            },
            NavLink { href: "/ideas", label: "Ideas", icon: "bulb", owns: &[] },
            NavLink {
                href: "/predictions",
                label: "Predictions",
                icon: "list",
                owns: &["/markets/"],
            },
        ],
    },
    NavGroup {
        id: "data",
        label: "Data",
        links: &[NavLink {
            href: "/snapshots",
            label: "Snapshots",
            icon: "database",
            owns: &["/snapshots/"],
        }],
    },
    NavGroup {
        id: "firm",
        label: "Firm",
        links: &[
            NavLink { href: "/state", label: "State", icon: "settings", owns: &[] },
            NavLink { href: "/decisions", label: "Decisions", icon: "gavel", owns: &[] },
            NavLink { href: "/inboxes", label: "Inboxes", icon: "inbox", owns: &[] },
            NavLink { href: "/wiki", label: "Wiki", icon: "book", owns: &["/wiki/"] },
        ],
    },
    NavGroup {
        id: "development",
        label: "Development",
        links: &[
            NavLink { href: "/dev", label: "Charts", icon: "chart", owns: &[] },
            NavLink { href: "/dev/endpoints", label: "Endpoints", icon: "plug", owns: &[] },
        ],
    },
];

fn link_is_active(link: &NavLink, path: &str) -> bool {
    link.href == path || link.owns.iter().any(|p| path.starts_with(p))
}

/// Destination for a breadcrumb given only its label: a nav item's own route,
/// or a section's first page. "" when the label names neither.
fn nav_href(label: &str) -> String {
    for g in NAV {
        if g.label == label {
            return g.links.first().map(|l| l.href.to_string()).unwrap_or_default();
        }
        for l in g.links {
            if l.label == label {
                return l.href.to_string();
            }
        }
    }
    String::new()
}

// ---------------------------------------------------------------------------
// Page shell
// ---------------------------------------------------------------------------

/// One breadcrumb hop. `href` empty ⇒ the current page (rendered as text).
pub struct Crumb {
    pub label: String,
    pub href: String,
}

pub fn crumb(label: &str, href: &str) -> Crumb {
    Crumb { label: label.to_string(), href: href.to_string() }
}

/// What the top-right freshness indicator says.
pub struct Freshness {
    /// true ⇒ every read on this page came from GitHub at request time.
    pub live: bool,
    /// Timestamp shown next to the dot (state date when live, build stamp otherwise).
    pub stamp: String,
    pub build: String,
}

/// The full page: sidebar (collapsible groups, localStorage-persisted), top bar
/// (breadcrumbs + theme toggle + freshness), content, footer.
/// `active` is the current route path; `body` is already-safe HTML.
///
/// One header band, always. On desktop the wordmark sits in the sidebar's own
/// 3rem row, level with the top bar across the seam. Below 900px the sidebar
/// stops being a band at all — the burger moves into the top bar, which then
/// carries everything, and the nav drops out of it as an overlay. The bar's
/// left slot is the breadcrumb while the menu is closed and the wordmark while
/// it is open (`.wordmark-bar`), because navigation has replaced the page's
/// context and the app's name is what belongs there.
pub fn layout(active: &str, crumbs: &[Crumb], body: &str, fresh: &Freshness) -> String {
    // --- sidebar ---
    let mut nav = String::new();
    for group in NAV {
        let has_active = group.links.iter().any(|l| link_is_active(l, active));
        let mut items = String::new();
        for link in group.links {
            let current = if link_is_active(link, active) {
                " aria-current=\"page\""
            } else {
                ""
            };
            items.push_str(&format!(
                "<a href=\"{}\"{}>{}<span>{}</span></a>",
                link.href,
                current,
                icon(link.icon),
                esc(link.label)
            ));
        }
        nav.push_str(&format!(
            r#"<div class="nav-group" data-group="{id}" data-active="{act}">
<button type="button" class="nav-group-head" aria-expanded="true" aria-controls="nav-{id}"><span class="chev">{chev}</span>{label}</button>
<div class="nav-items" id="nav-{id}">{items}</div>
</div>"#,
            id = group.id,
            act = if has_active { "1" } else { "0" },
            chev = icon("chevron"),
            label = esc(group.label),
            items = items,
        ));
    }

    // --- breadcrumbs: every level except the leaf is navigable --------------
    // A crumb with no explicit href is resolved against NAV by label, so a
    // section ("Research") lands on that section's first page. PRINCIPLES:
    // everything that looks navigable is navigable.
    let mut crumb_html = String::new();
    for (i, c) in crumbs.iter().enumerate() {
        if i > 0 {
            crumb_html.push_str("<span class=\"sep\">/</span>");
        }
        let last = i + 1 == crumbs.len();
        let href = if c.href.is_empty() { nav_href(&c.label) } else { c.href.clone() };
        if last || href.is_empty() {
            crumb_html.push_str(&format!("<span class=\"here\">{}</span>", esc(&c.label)));
        } else {
            crumb_html.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                esc(&href),
                esc(&c.label)
            ));
        }
    }

    let title = crumbs
        .last()
        .map(|c| c.label.clone())
        .unwrap_or_else(|| "Dashboard".to_string());

    let (dot, source) = if fresh.live {
        ("dot dot-ok", "live")
    } else {
        ("dot dot-warn", "snapshot")
    };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="noindex">
<title>{title} — orakel</title>
<link rel="icon" type="image/svg+xml" href="/favicon.svg">
<link rel="stylesheet" href="/style.css">
<script>try{{var t=localStorage.getItem("orakel-theme");if(t)document.documentElement.setAttribute("data-theme",t);}}catch(e){{}}</script>
</head>
<body>
<div class="shell">
  <input type="checkbox" id="nav-toggle" class="nav-toggle">
  <aside class="sidebar">
    <div class="sidebar-top">
      <a class="wordmark" href="/">orakel<small>dashboard</small></a>
    </div>
    <nav class="nav" id="nav">{nav}</nav>
  </aside>
  <div class="main">
    <div class="topbar">
      <a class="wordmark wordmark-bar" href="/">orakel<small>dashboard</small></a>
      <nav class="crumbs" aria-label="Breadcrumb">{crumb_html}</nav>
      <div class="topbar-right">
        <span class="updated" title="Data freshness"><span class="{dot}"></span>{source} · {stamp}</span>
        <button type="button" class="icon-btn" id="theme-toggle" aria-label="Switch colour theme"><span class="sun">{sun}</span><span class="moon">{moon}</span></button>
        <label class="burger" for="nav-toggle" aria-label="Toggle navigation">{burger}</label>
      </div>
    </div>
    <main class="content">
{body}
    </main>
    <footer class="footer">
      <span>orakel — agentic prediction-market research firm</span>
      <span class="num">worker built {build}</span>
    </footer>
  </div>
</div>
<script>
(function () {{
  /* Sidebar groups: expanded/collapsed per section, remembered in
     localStorage. The group holding the active route is always expanded.
     Without JS every group stays open — nothing is unreachable. */
  var nav = document.getElementById("nav");
  if (nav) {{
    var groups = nav.querySelectorAll(".nav-group");
    for (var i = 0; i < groups.length; i++) (function (g) {{
      var id = g.getAttribute("data-group");
      var head = g.querySelector(".nav-group-head");
      var open = true;
      if (g.getAttribute("data-active") !== "1") {{
        try {{
          var saved = localStorage.getItem("orakel-nav:" + id);
          if (saved !== null) open = saved === "1";
        }} catch (e) {{}}
      }}
      head.setAttribute("aria-expanded", open ? "true" : "false");
      head.addEventListener("click", function () {{
        var next = head.getAttribute("aria-expanded") !== "true";
        head.setAttribute("aria-expanded", next ? "true" : "false");
        try {{ localStorage.setItem("orakel-nav:" + id, next ? "1" : "0"); }} catch (e) {{}}
      }});
    }})(groups[i]);
  }}
  /* Theme toggle: system default, overridden by an explicit choice that
     survives reloads. charts.js re-renders on the data-theme change. */
  var btn = document.getElementById("theme-toggle");
  if (btn) btn.addEventListener("click", function () {{
    var cur = document.documentElement.getAttribute("data-theme");
    if (!cur) cur = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    var next = cur === "dark" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", next);
    try {{ localStorage.setItem("orakel-theme", next); }} catch (e) {{}}
  }});
}})();
</script>
</body>
</html>"#,
        title = esc(&title),
        burger = icon_sized("burger", 18),
        nav = nav,
        crumb_html = crumb_html,
        dot = dot,
        source = source,
        stamp = esc(&fresh.stamp),
        sun = icon("sun"),
        moon = icon("moon"),
        build = esc(&fresh.build),
        body = body,
    )
}

// ---------------------------------------------------------------------------
// KPI cards
// ---------------------------------------------------------------------------

pub struct Kpi {
    pub label: String,
    /// Already-safe HTML (usually a number, optionally with a <small> suffix).
    pub value: String,
    /// (text, tone) — tone: "up" | "down" | "warn" | "" (neutral).
    pub delta: Option<(String, &'static str)>,
    pub context: String,
    pub icon: &'static str,
    /// 1..5 — which accent tint the icon square uses.
    pub tint: u8,
}

pub fn kpi(label: &str, value: &str, icon_name: &'static str, tint: u8) -> Kpi {
    Kpi {
        label: label.to_string(),
        value: value.to_string(),
        delta: None,
        context: String::new(),
        icon: icon_name,
        tint,
    }
}

impl Kpi {
    pub fn delta(mut self, text: &str, tone: &'static str) -> Self {
        self.delta = Some((text.to_string(), tone));
        self
    }
    pub fn context(mut self, text: &str) -> Self {
        self.context = text.to_string();
        self
    }
}

/// One-line statistics strip: the same numbers a KPI row carries, in about a
/// quarter of the height. Used on every page whose point is the content below
/// it — KPI cards are reserved for pages where the numbers ARE the headline.
/// Each entry is (value_html, label, tone) with tone "" | "ok" | "warn" | "bad".
pub fn stat_line(items: &[(String, String, &str)]) -> String {
    let mut out = String::new();
    for (value, label, tone) in items {
        let class = match *tone {
            "ok" => " s-ok",
            "warn" => " s-warn",
            "bad" => " s-bad",
            _ => "",
        };
        out.push_str(&format!(
            "<span class=\"stat\"><b class=\"stat-v{class}\">{value}</b><span class=\"stat-k\">{}</span></span>",
            esc(label)
        ));
    }
    format!("<div class=\"statline\">{out}</div>")
}

pub fn kpi_row(cards: &[Kpi]) -> String {
    let class = match cards.len() {
        3 => "grid-kpi grid-kpi-3",
        4 => "grid-kpi grid-kpi-4",
        _ => "grid-kpi",
    };
    let mut out = String::new();
    for c in cards {
        let delta = match &c.delta {
            Some((text, tone)) => {
                let (cls, mark) = match *tone {
                    "up" => ("delta delta-up", "↑ "),
                    "down" => ("delta delta-down", "↓ "),
                    "warn" => ("delta delta-warn", ""),
                    _ => ("delta", ""),
                };
                format!("<span class=\"{}\">{}{}</span>", cls, mark, esc(text))
            }
            None => String::new(),
        };
        out.push_str(&format!(
            r#"<article class="kpi"><div class="kpi-head"><span class="kpi-label">{label}</span><span class="kpi-icon kpi-icon-{tint}">{icon}</span></div><div class="kpi-value">{value}</div><div class="kpi-foot">{delta}<span class="kpi-context">{context}</span></div></article>"#,
            label = esc(&c.label),
            tint = c.tint,
            icon = icon(c.icon),
            value = c.value,
            delta = delta,
            context = esc(&c.context),
        ));
    }
    format!("<div class=\"{class}\">{out}</div>")
}

// ---------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------

/// Bordered surface: title + subtitle line on the left, status chip right.
/// `body_html` is already-safe HTML.
pub fn panel(title: &str, subtitle: &str, chip_html: &str, body_html: &str) -> String {
    panel_inner(title, subtitle, chip_html, body_html, "panel-body", "")
}

/// Panel whose body is a flush table (no padding) — for dense data.
pub fn panel_flush(title: &str, subtitle: &str, chip_html: &str, body_html: &str) -> String {
    panel_inner(title, subtitle, chip_html, body_html, "panel-body panel-body-flush", "")
}

/// Panel with a footer strip (links, counts).
pub fn panel_foot(
    title: &str,
    subtitle: &str,
    chip_html: &str,
    body_html: &str,
    foot_html: &str,
    flush: bool,
) -> String {
    let body_class = if flush { "panel-body panel-body-flush" } else { "panel-body" };
    panel_inner(
        title,
        subtitle,
        chip_html,
        body_html,
        body_class,
        &format!("<div class=\"panel-foot\">{foot_html}</div>"),
    )
}

fn panel_inner(
    title: &str,
    subtitle: &str,
    chip_html: &str,
    body_html: &str,
    body_class: &str,
    foot: &str,
) -> String {
    let sub = if subtitle.is_empty() {
        String::new()
    } else {
        format!("<p class=\"panel-sub\">{}</p>", esc(subtitle))
    };
    format!(
        r#"<section class="panel"><div class="panel-head"><div><h2 class="panel-title">{title}</h2>{sub}</div>{chip}</div><div class="{body_class}">{body}</div>{foot}</section>"#,
        title = esc(title),
        sub = sub,
        chip = chip_html,
        body_class = body_class,
        body = body_html,
        foot = foot,
    )
}

// ---------------------------------------------------------------------------
// Badges, chips, small bits
// ---------------------------------------------------------------------------

/// tone: "" (neutral) | "ok" | "warn" | "bad" | "info"
pub fn badge(text: &str, tone: &str) -> String {
    let class = match tone {
        "ok" => "badge badge-ok",
        "warn" => "badge badge-warn",
        "bad" => "badge badge-bad",
        "info" => "badge badge-info",
        _ => "badge",
    };
    format!("<span class=\"{}\">{}</span>", class, esc(text))
}

pub fn chip(text: &str) -> String {
    format!("<span class=\"chip\">{}</span>", esc(text))
}

pub fn chip_row(items: &[String]) -> String {
    let chips: String = items.iter().map(|c| chip(c)).collect();
    format!("<div class=\"chip-row\">{chips}</div>")
}

/// Status word → badge tone, shared by strategy status, message frontmatter
/// status and run step status.
pub fn status_tone(status: &str) -> &'static str {
    match status {
        "ok" | "done" | "answered" | "closed" | "resolved" | "adopted" | "live" | "true" => "ok",
        "open" | "pending" | "trialing" | "trial" | "skipped" | "in-flight" => "warn",
        "failed" | "rejected" | "dead" | "retired" | "killed" | "false" => "bad",
        s if s.starts_with("kill") => "bad",
        _ => "",
    }
}

pub fn status_badge(status: &str) -> String {
    badge(status, status_tone(status))
}

/// Inline proportional bar. `frac` is clamped to 0..1; variant 1|2|3 picks tint.
pub fn minibar(frac: f64, variant: u8) -> String {
    let pct = (frac.clamp(0.0, 1.0) * 100.0).round();
    let class = match variant {
        2 => "minibar minibar-2",
        3 => "minibar minibar-3",
        _ => "minibar",
    };
    format!("<span class=\"{class}\"><i style=\"width:{pct}%\"></i></span>")
}

pub fn empty_state(title: &str, detail_html: &str) -> String {
    format!(
        "<div class=\"empty\"><strong>{}</strong>{}</div>",
        esc(title),
        detail_html
    )
}

pub fn note(html_inner: &str) -> String {
    format!("<p class=\"note\">{html_inner}</p>")
}

/// key → value row inside a panel. `value_html` is already-safe HTML.
pub fn row(key: &str, value_html: &str) -> String {
    format!(
        "<div class=\"row\"><span class=\"k\">{}</span><span class=\"v\">{}</span></div>",
        esc(key),
        value_html
    )
}

pub fn rows(inner: &str) -> String {
    format!("<div class=\"rows\">{inner}</div>")
}

/// Compact list entry: title + sub line on the left, trailing HTML right.
/// `href` empty ⇒ not a link.
pub fn item(href: &str, title_html: &str, sub_html: &str, trailing_html: &str) -> String {
    let inner = format!(
        "<div class=\"item-main\"><div class=\"item-title\">{title_html}</div><div class=\"item-sub\">{sub_html}</div></div>{trailing_html}"
    );
    if href.is_empty() {
        format!("<div class=\"item\">{inner}</div>")
    } else {
        format!("<a class=\"item\" href=\"{}\">{inner}</a>", esc(href))
    }
}

pub fn items(inner: &str) -> String {
    format!("<div class=\"items\">{inner}</div>")
}

// ---------------------------------------------------------------------------
// Tables
// ---------------------------------------------------------------------------

/// Dense data table. `head` is (label, class) per column — the class is applied
/// to the `<th>` and every `<td>` of that column ("num" right-aligns and turns
/// on tabular figures, "wrap" lets long prose wrap). Cells are already-safe
/// HTML so rows can carry links, badges and mini-bars.
pub fn table(head: &[(&str, &str)], body_rows: &[Vec<String>]) -> String {
    let mut out = String::from("<div class=\"table-wrap\"><table class=\"data\"><thead><tr>");
    for (label, class) in head {
        out.push_str(&format!(
            "<th class=\"{}\">{}</th>",
            class,
            esc(label)
        ));
    }
    out.push_str("</tr></thead><tbody>");
    for r in body_rows {
        out.push_str("<tr>");
        for (i, cell) in r.iter().enumerate() {
            let class = head.get(i).map(|(_, c)| *c).unwrap_or("");
            out.push_str(&format!("<td class=\"{class}\">{cell}</td>"));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table></div>");
    out
}

/// Same as `table`, in a vertically scrolling box with a sticky header.
pub fn table_scroll(head: &[(&str, &str)], body_rows: &[Vec<String>]) -> String {
    table(head, body_rows).replacen(
        "<div class=\"table-wrap\">",
        "<div class=\"table-wrap table-scroll\">",
        1,
    )
}

/// One CSV cell → display HTML: opaque ids (>28 chars, alphanumeric) are
/// abbreviated with the full value in the title attribute.
pub fn cell_text(cell: &str) -> String {
    let opaque = cell.chars().count() > 28 && cell.chars().all(|c| c.is_ascii_alphanumeric());
    if opaque {
        let head: String = cell.chars().take(8).collect();
        let n = cell.chars().count();
        let tail: String = cell.chars().skip(n - 4).collect();
        format!(
            "<span class=\"mono\" title=\"{}\">{}…{}</span>",
            esc(cell),
            esc(&head),
            esc(&tail)
        )
    } else {
        esc(cell)
    }
}

// ---------------------------------------------------------------------------
// Collapsible documents (inbox messages, ideas, worklogs, results)
// ---------------------------------------------------------------------------

/// `<details>` block: summary line + already-safe body HTML.
pub fn doc(summary_html: &str, meta_html: &str, body_html: &str, open: bool) -> String {
    format!(
        "<details class=\"doc\"{}><summary>{}<span class=\"doc-meta\">{}</span></summary><div class=\"doc-body\">{}</div></details>",
        if open { " open" } else { "" },
        summary_html,
        meta_html,
        body_html
    )
}

/// Split optional `---` frontmatter into (fields, body). Values keep only the
/// part before an inline ` #` comment (`status: trialing # -> ...`).
pub fn split_frontmatter(src: &str) -> (Vec<(String, String)>, &str) {
    let Some(rest) = src.strip_prefix("---") else {
        return (Vec::new(), src);
    };
    let Some(end) = rest.find("\n---") else {
        return (Vec::new(), src);
    };
    let head = &rest[..end];
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']);
    let fields = head
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            let v = v.trim();
            let v = v.split(" #").next().unwrap_or(v).trim();
            Some((k.trim().to_string(), v.to_string()))
        })
        .collect();
    (fields, body)
}

// ---------------------------------------------------------------------------
// Number formatting — every figure on the dashboard goes through here so the
// same quantity never appears with two different precisions.
// ---------------------------------------------------------------------------

/// Probability-like value: 4 decimals, no trailing padding beyond that.
pub fn fmt_prob(v: f64) -> String {
    format!("{v:.4}")
}

/// Signed small metric (improvement, edge): 4 decimals with an explicit sign.
pub fn fmt_signed(v: f64) -> String {
    format!("{}{:.4}", if v >= 0.0 { "+" } else { "−" }, v.abs())
}

/// "1 variant" / "3 variants" — every count chip goes through here.
pub fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else if let Some(stem) = noun.strip_suffix('y') {
        format!("{n} {stem}ies")
    } else {
        format!("{n} {noun}s")
    }
}

/// Thousands separators for counts (108, 2 050 000 → "2,050,000").
pub fn fmt_int(v: i64) -> String {
    let neg = v < 0;
    let digits: Vec<char> = v.abs().to_string().chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    if neg {
        format!("−{out}")
    } else {
        out
    }
}

/// Compact token counts: 600000 → "600k", 2050000 → "2.05M".
pub fn fmt_tokens(v: i64) -> String {
    let f = v as f64;
    if f >= 1_000_000.0 {
        format!("{:.2}M", f / 1_000_000.0)
    } else if f >= 1_000.0 {
        format!("{:.0}k", f / 1_000.0)
    } else {
        v.to_string()
    }
}

/// `2026-07-25T01:52:10Z` → `2026-07-25 01:52`; plain dates pass through.
pub fn fmt_ts(ts: &str) -> String {
    if ts.len() >= 16 && ts.contains('T') {
        ts[..16].replace('T', " ")
    } else {
        ts.to_string()
    }
}
