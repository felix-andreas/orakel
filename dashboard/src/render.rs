//! HTML rendering: the page shell (sidebar + top bar) and the small set of
//! components every page is built from. Plain `format!` string building, no
//! template engine (CODING.md: procedural, data in → HTML out).
//!
//! Component vocabulary — nothing else exists, on purpose:
//!   section     a heading, optional one-line subtitle, and content. NOT a box.
//!   stat_grid   dense grid of value/label/context stats (the headline strip)
//!   stat_line   one-line stat strip, for pages whose point is the content below
//!   table       a dense data table (per-column class drives alignment)
//!   items/row   compact lists — hairline-separated rows, never boxes
//!   badge/chip  status and metadata
//!   minibar     inline proportional bar for dense tables
//!   prose       rendered markdown
//!
//! PRINCIPLES.md, binding: **cards are the exception, not the container** and
//! **never nest a card in a card**. There is therefore no generic bordered
//! "panel" component any more — hierarchy is carried by headings, spacing and
//! type scale. The only bordered surfaces left in the stylesheet are things
//! that float (the settings popover, chart tooltips) or warn (the snapshot
//! banner).
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

/// The document's leading `# ` heading, if it has one. Sections use it as their
/// title so the same words aren't printed twice (breadcrumb, section header and
/// then an H1 in the prose was exactly the "repeated page title" problem).
pub fn md_title(src: &str) -> Option<String> {
    let first = src.lines().find(|l| !l.trim().is_empty())?;
    first.strip_prefix("# ").map(|t| t.trim().to_string())
}

/// Markdown → HTML with the leading `# ` heading removed. Use wherever the
/// surrounding section or breadcrumb already names the document.
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
// speeds up recognition (nav items, the settings control, theme choices).
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
        "sliders" => r#"<path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h10M18 18h2"/><circle cx="16" cy="6" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="16" cy="18" r="2"/>"#,
        "target" => r#"<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1.4"/>"#,
        "trend-up" => r#"<path d="m3 17 6-6 4 4 8-8"/><path d="M15 7h6v6"/>"#,
        "layers" => r#"<path d="m12 2 9 5-9 5-9-5 9-5Z"/><path d="m3 12 9 5 9-5"/><path d="m3 17 9 5 9-5"/>"#,
        "bolt" => r#"<path d="M13 2 4 14h7l-1 8 9-12h-7l1-8Z"/>"#,
        "clock" => r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3.5 2"/>"#,
        "alert" => r#"<path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"/><path d="M12 9v4M12 17h.01"/>"#,
        "check" => r#"<path d="m5 13 4 4 10-10"/>"#,
        "sun" => r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>"#,
        "moon" => r#"<path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z"/>"#,
        "monitor" => r#"<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>"#,
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

/// The wordmark: the sea-shell mark and the firm's name, nothing else. The mark
/// is the same path as `src/favicon.svg`, inlined so it can be drawn in
/// `currentColor` and follow the theme — the favicon file itself must hard-code
/// a colour, because a browser tab icon has nothing to inherit from. Keep the
/// two paths in sync if the mark ever changes.
fn wordmark(class: &str) -> String {
    format!(
        r#"<a class="{class}" href="/"><svg class="mark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M11.5 12A2.5 2.5 0 0 0 7 13.5c0 1.38 1 3.5 3.5 3.5c3 0 4.5-2.5 4.5-4.5c0-3-2-5.5-5.5-5.5S3 9.5 3 14c0 3.314 2.5 8 9 8c4.97 0 9-5.03 9-10S17 2 13 2a2 2 0 0 0-2 2v3"/></svg>orakel</a>"#
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
            // "Backtest", not "Execution": the firm places no orders
            // (CONSTITUTION.md §5) and the page is a replay of stored signals
            // against stored prices. The icon still reads as replay.
            NavLink { href: "/backtest", label: "Backtest", icon: "play", owns: &["/backtest/"] },
            // "Paper book", not "Positions" or "Execution": the label has to
            // carry the fact that no money and no order is involved
            // (CONSTITUTION.md §5) every time the page is named, not only once
            // on the page itself.
            NavLink { href: "/execution", label: "Paper book", icon: "layers", owns: &[] },
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
    /// Repo timestamp shown next to the dot. Empty when nothing could be read.
    pub stamp: String,
    pub build: String,
    /// What went wrong, in words. `Some` ⇒ the page is showing an incomplete
    /// picture and the banner says why. There is no fallback data source, so
    /// this is never "here is an older copy" — it is "this is missing".
    pub reason: Option<String>,
}

/// The settings popover: theme (light / dark / system), density, sidebar
/// sections, and where the thing lives. Native Popover API — `popovertarget`
/// on the button, `popover` on the panel — so opening, light-dismiss, Esc and
/// focus handling are the browser's, not ours. Without JS it still opens; the
/// controls then say so instead of pretending to work.
fn settings_popover() -> String {
    format!(
        r#"<div id="settings-pop" popover class="pop" aria-label="Settings">
  <div class="pop-group">
    <div class="pop-label">Theme</div>
    <div class="seg" id="seg-theme" role="group" aria-label="Colour theme">
      <button type="button" data-theme-set="light">{sun}<span>Light</span></button>
      <button type="button" data-theme-set="dark">{moon}<span>Dark</span></button>
      <button type="button" data-theme-set="system">{monitor}<span>System</span></button>
    </div>
  </div>
  <div class="pop-group">
    <div class="pop-label">Density</div>
    <div class="seg" id="seg-density" role="group" aria-label="Row density">
      <button type="button" data-density-set="comfortable"><span>Comfortable</span></button>
      <button type="button" data-density-set="compact"><span>Compact</span></button>
    </div>
  </div>
  <div class="pop-group">
    <div class="pop-label">Sidebar sections</div>
    <div class="seg" role="group" aria-label="Sidebar sections">
      <button type="button" data-nav-all="1"><span>Expand all</span></button>
      <button type="button" data-nav-all="0"><span>Collapse all</span></button>
    </div>
  </div>
  <div class="pop-links">
    <a href="https://github.com/felix-andreas/orakel" target="_blank" rel="noreferrer">{ext}<span>Repository</span></a>
    <a href="/dev/endpoints">{plug}<span>Endpoints</span></a>
  </div>
  <noscript><p class="pop-note">These settings need JavaScript. Without it the page follows your system colour scheme and every sidebar section stays open.</p></noscript>
</div>"#,
        sun = icon("sun"),
        moon = icon("moon"),
        monitor = icon("monitor"),
        ext = icon("external"),
        plug = icon("plug"),
    )
}

/// The full page: sidebar (collapsible groups + the settings popover at its
/// bottom-left), top bar (breadcrumbs + freshness), an optional secondary bar,
/// content, footer. `active` is the current route path; `subbar` and `body` are
/// already-safe HTML ("" ⇒ no secondary bar).
///
/// One header band, always. On desktop the wordmark sits in the sidebar's own
/// 3rem row, level with the top bar across the seam. Below 900px the sidebar
/// stops being a band at all — the burger moves into the top bar, which then
/// carries everything, and the nav drops out of it as a FULL-VIEWPORT overlay
/// (dynamic viewport units, so mobile browser chrome cannot clip it). The bar's
/// left slot is the breadcrumb while the menu is closed and the wordmark while
/// it is open (`.wordmark-bar`), because navigation has replaced the page's
/// context and the app's name is what belongs there.
pub fn layout(
    active: &str,
    crumbs: &[Crumb],
    subbar: &str,
    body: &str,
    fresh: &Freshness,
) -> String {
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

    // There is no second source of data, so "not live" means "not shown", not
    // "shown from a snapshot". The indicator has to read as a fault.
    let (dot, source, stamp_html) = if fresh.live {
        ("dot dot-ok", "live", format!(" · {}", esc(&fresh.stamp)))
    } else {
        ("dot dot-bad", "cannot read repo", String::new())
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
<script>try{{var t=localStorage.getItem("orakel-theme");if(t==="dark"||t==="light")document.documentElement.setAttribute("data-theme",t);var d=localStorage.getItem("orakel-density");if(d==="compact")document.documentElement.setAttribute("data-density","compact");}}catch(e){{}}</script>
</head>
<body>
<div class="shell">
  <input type="checkbox" id="nav-toggle" class="nav-toggle">
  <aside class="sidebar">
    <div class="sidebar-top">
      {wordmark}
    </div>
    <nav class="nav" id="nav">{nav}</nav>
    <div class="sidebar-foot">
      <button type="button" class="settings-btn" popovertarget="settings-pop">{gear}<span>Settings</span></button>
    </div>
  </aside>
  <div class="main">
    <div class="topbar">
      {wordmark_bar}
      <nav class="crumbs" aria-label="Breadcrumb">{crumb_html}</nav>
      <div class="topbar-right">
        <span class="updated" title="Data freshness"><span class="{dot}"></span>{source}{stamp}</span>
        <label class="burger" for="nav-toggle" aria-label="Toggle navigation">{burger}</label>
      </div>
    </div>
{subbar}
    <main class="content">
{fetch_error}{body}
    </main>
    <footer class="footer">
      <span>orakel — agentic prediction-market research firm</span>
      <span class="num">worker built {build}</span>
      <!-- reads: {reads} -->
    </footer>
  </div>
</div>
{pop}
<script>
(function () {{
  var root = document.documentElement;
  function store(k, v) {{ try {{ localStorage.setItem(k, v); }} catch (e) {{}} }}
  function read(k) {{ try {{ return localStorage.getItem(k); }} catch (e) {{ return null; }} }}

  /* Sidebar groups: expanded/collapsed per section, remembered in
     localStorage. The group holding the active route is always expanded.
     Without JS every group stays open — nothing is unreachable. */
  var nav = document.getElementById("nav");
  var groups = nav ? nav.querySelectorAll(".nav-group") : [];
  function setGroup(g, open, persist) {{
    var head = g.querySelector(".nav-group-head");
    head.setAttribute("aria-expanded", open ? "true" : "false");
    if (persist) store("orakel-nav:" + g.getAttribute("data-group"), open ? "1" : "0");
  }}
  for (var i = 0; i < groups.length; i++) (function (g) {{
    var open = true;
    if (g.getAttribute("data-active") !== "1") {{
      var saved = read("orakel-nav:" + g.getAttribute("data-group"));
      if (saved !== null) open = saved === "1";
    }}
    setGroup(g, open, false);
    g.querySelector(".nav-group-head").addEventListener("click", function () {{
      var head = g.querySelector(".nav-group-head");
      setGroup(g, head.getAttribute("aria-expanded") !== "true", true);
    }});
  }})(groups[i]);

  /* Settings: theme (light / dark / SYSTEM — the default) and density, both
     persisted. "system" removes the attribute so the stylesheet's
     prefers-color-scheme block takes over; charts.js re-renders on both. */
  function currentTheme() {{
    var t = read("orakel-theme");
    return t === "dark" || t === "light" ? t : "system";
  }}
  function currentDensity() {{
    return read("orakel-density") === "compact" ? "compact" : "comfortable";
  }}
  function mark(sel, attr, value) {{
    var btns = document.querySelectorAll(sel + " [" + attr + "]");
    for (var i = 0; i < btns.length; i++)
      btns[i].setAttribute("aria-pressed", btns[i].getAttribute(attr) === value ? "true" : "false");
  }}
  function applyTheme(v) {{
    if (v === "dark" || v === "light") root.setAttribute("data-theme", v);
    else root.removeAttribute("data-theme");
    mark('#seg-theme', 'data-theme-set', v);
  }}
  function applyDensity(v) {{
    if (v === "compact") root.setAttribute("data-density", "compact");
    else root.removeAttribute("data-density");
    mark('#seg-density', 'data-density-set', v);
  }}
  applyTheme(currentTheme());
  applyDensity(currentDensity());

  document.addEventListener("click", function (e) {{
    var t = e.target.closest ? e.target.closest("[data-theme-set],[data-density-set],[data-nav-all]") : null;
    if (!t) return;
    if (t.hasAttribute("data-theme-set")) {{
      var v = t.getAttribute("data-theme-set");
      store("orakel-theme", v);
      applyTheme(v);
    }} else if (t.hasAttribute("data-density-set")) {{
      var d = t.getAttribute("data-density-set");
      store("orakel-density", d);
      applyDensity(d);
    }} else {{
      var open = t.getAttribute("data-nav-all") === "1";
      for (var i = 0; i < groups.length; i++) setGroup(groups[i], open, true);
    }}
  }});
}})();
</script>
</body>
</html>"#,
        title = esc(&title),
        burger = icon_sized("burger", 18),
        gear = icon("sliders"),
        nav = nav,
        pop = settings_popover(),
        wordmark = wordmark("wordmark"),
        wordmark_bar = wordmark("wordmark wordmark-bar"),
        subbar = subbar,
        crumb_html = crumb_html,
        dot = dot,
        source = source,
        stamp = stamp_html,
        // Rendered here rather than by each page, so a page cannot forget it.
        fetch_error = fresh
            .reason
            .as_deref()
            .map(crate::fetch_error_banner)
            .unwrap_or_default(),
        build = esc(&fresh.build),
        // Read telemetry as an HTML comment, not visible chrome — it is a
        // diagnostic for whoever is chasing the cold-cache read loss, and
        // PRINCIPLES.md is explicit that nothing visible may exist without a
        // UX purpose. Putting it in the RESPONSE rather than in a log is what
        // makes it usable: a cold request creates a new isolate, so any
        // counter I query afterwards belongs to a different one. The page has
        // to carry its own numbers.
        reads = esc(&crate::live::read_stats_line()),
        body = body,
    )
}

// ---------------------------------------------------------------------------
// The secondary bar — a detail page's own tabs
// ---------------------------------------------------------------------------

/// The tab bar that sits under the top bar (`layout`'s `subbar` slot): one
/// entry per tab, each a REAL link, so a tab is linkable, bookmarkable and
/// correct under the back button. Nothing here hides content client-side.
///
/// Each tab is `(key, label, count)`; `key` "" is the page's default tab and
/// carries no query parameter, so the bare URL stays the canonical address of
/// the thing. `count` "" ⇒ no number. The active tab is marked by WEIGHT and a
/// rule under it — never by dimming the others (PRINCIPLES: contrast serves
/// reading).
pub fn tabbar(base: &str, tabs: &[(&str, &str, String)], active: &str) -> String {
    let mut out = String::new();
    for (key, label, count) in tabs {
        let href = if key.is_empty() {
            base.to_string()
        } else {
            format!("{base}?tab={key}")
        };
        let n = if count.is_empty() {
            String::new()
        } else {
            format!("<span class=\"tab-n\">{}</span>", esc(count))
        };
        out.push_str(&format!(
            "<a href=\"{}\"{}>{}{}</a>",
            esc(&href),
            if *key == active { " aria-current=\"page\"" } else { "" },
            esc(label),
            n
        ));
    }
    format!("<nav class=\"subbar\" aria-label=\"Sections of this page\"><div class=\"subbar-in\">{out}</div></nav>")
}

// ---------------------------------------------------------------------------
// Stats — the headline numbers. No cards: a grid of value + label + context,
// separated by space, at about a third of the height a card row costs.
// ---------------------------------------------------------------------------

pub struct Stat {
    /// Already-safe HTML (usually a number, optionally with a <small> suffix).
    pub value: String,
    pub label: String,
    /// "" | "ok" | "warn" | "bad"
    pub tone: &'static str,
    /// One short line under the label. Plain text.
    pub context: String,
    /// Wraps the value in a link when set.
    pub href: String,
}

pub fn stat(value: &str, label: &str) -> Stat {
    Stat {
        value: value.to_string(),
        label: label.to_string(),
        tone: "",
        context: String::new(),
        href: String::new(),
    }
}

impl Stat {
    pub fn tone(mut self, tone: &'static str) -> Self {
        self.tone = tone;
        self
    }
    pub fn context(mut self, text: &str) -> Self {
        self.context = text.to_string();
        self
    }
    pub fn href(mut self, href: &str) -> Self {
        self.href = href.to_string();
        self
    }
}

fn tone_class(tone: &str) -> &'static str {
    match tone {
        "ok" => " s-ok",
        "warn" => " s-warn",
        "bad" => " s-bad",
        _ => "",
    }
}

/// Dense headline grid: as many columns as fit, one line of context each.
pub fn stat_grid(items: &[Stat]) -> String {
    let mut out = String::new();
    for s in items {
        let value = if s.href.is_empty() {
            format!("<b class=\"stat-v{}\">{}</b>", tone_class(s.tone), s.value)
        } else {
            format!(
                "<a class=\"stat-v{}\" href=\"{}\">{}</a>",
                tone_class(s.tone),
                esc(&s.href),
                s.value
            )
        };
        out.push_str(&format!(
            "<div class=\"stat\">{value}<span class=\"stat-k\">{}</span>{}</div>",
            esc(&s.label),
            if s.context.is_empty() {
                String::new()
            } else {
                format!("<span class=\"stat-c\">{}</span>", esc(&s.context))
            }
        ));
    }
    format!("<div class=\"statgrid\">{out}</div>")
}

/// One-line statistics strip: the same numbers in about a quarter of the
/// height. Each entry is (value_html, label, tone) with tone "" | "ok" | "warn"
/// | "bad".
pub fn stat_line(items: &[(String, String, &str)]) -> String {
    let mut out = String::new();
    for (value, label, tone) in items {
        out.push_str(&format!(
            "<span class=\"stat\"><b class=\"stat-v{}\">{value}</b><span class=\"stat-k\">{}</span></span>",
            tone_class(tone),
            esc(label)
        ));
    }
    format!("<div class=\"statline\">{out}</div>")
}

// ---------------------------------------------------------------------------
// Sections — a heading, a hairline, content. The default container; NOT a box.
// ---------------------------------------------------------------------------

/// Section: title (+ optional one-line subtitle beside it, and a chip on the
/// right), then the body. `body_html` is already-safe HTML.
pub fn section(title: &str, subtitle: &str, chip_html: &str, body_html: &str) -> String {
    section_inner(title, subtitle, chip_html, body_html, "")
}

/// Section with a footer strip (source path, links, counts).
pub fn section_foot(
    title: &str,
    subtitle: &str,
    chip_html: &str,
    body_html: &str,
    foot_html: &str,
) -> String {
    section_inner(
        title,
        subtitle,
        chip_html,
        body_html,
        &format!("<div class=\"sec-foot\">{foot_html}</div>"),
    )
}

fn section_inner(
    title: &str,
    subtitle: &str,
    chip_html: &str,
    body_html: &str,
    foot: &str,
) -> String {
    let sub = if subtitle.is_empty() {
        String::new()
    } else {
        format!("<span class=\"sec-sub\">{}</span>", esc(subtitle))
    };
    let head = if title.is_empty() && subtitle.is_empty() && chip_html.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"sec-head\"><h2 class=\"sec-title\">{}</h2>{sub}{chip}</div>",
            esc(title),
            chip = chip_html,
        )
    };
    format!("<section class=\"sec\">{head}<div class=\"sec-body\">{body_html}</div>{foot}</section>")
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
        // Parked is neither: the thesis held, there is just nothing to trade.
        // Toning it "bad" would read as a kill, "ok" as working. Neutral is honest.
        "parked" | "dormant" => "",
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

/// A short list of caveats. Each entry is already-safe HTML.
pub fn notes(items: &[String]) -> String {
    let inner: String = items
        .iter()
        .map(|n| format!("<li>{n}</li>"))
        .collect();
    format!("<ul class=\"notes\">{inner}</ul>")
}

/// key → value row inside a list. `value_html` is already-safe HTML.
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
/// A hairline-separated ROW, not a box — five things read as five lines.
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
    table_classed(head, body_rows, "data")
}

/// Same, but every header cell sorts the table (see /table.js). Use where the
/// reader's question decides the order — the execution matrix, above all.
pub fn table_sortable(head: &[(&str, &str)], body_rows: &[Vec<String>]) -> String {
    format!(
        "{}<script src=\"/table.js\"></script>",
        table_classed(head, body_rows, "data sortable")
    )
}

fn table_classed(head: &[(&str, &str)], body_rows: &[Vec<String>], class: &str) -> String {
    let mut out = format!("<div class=\"table-wrap\"><table class=\"{class}\"><thead><tr>");
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

/// `<details>` block: summary line + already-safe body HTML. A row with a
/// disclosure triangle, not a card — the body is what deserves the space.
pub fn doc(summary_html: &str, meta_html: &str, body_html: &str, open: bool) -> String {
    format!(
        "<details class=\"doc\"{}><summary><span class=\"doc-title\">{}</span><span class=\"doc-meta\">{}</span></summary><div class=\"doc-body\">{}</div></details>",
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

    // Fold YAML block scalars (`key: >-`, `>`, `|`, `|-`) into their value.
    //
    // Without this the parser returned the literal marker: `/ideas` rendered a
    // row whose "what decided it" column read **`>-`**, which is the worst kind
    // of cell — it occupies the space where the answer should be. Every idea
    // file written since 07-25 uses a block scalar for `summary`, so this was
    // most of them.
    //
    // Continuation lines are those indented past the key. Folded style (`>`)
    // joins with spaces, literal style (`|`) keeps newlines; both are needed
    // because both appear in `ideas/` and `roles/*/inbox/`.
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut lines = head.lines().peekable();
    while let Some(l) = lines.next() {
        let Some((k, v)) = l.split_once(':') else { continue };
        let key = k.trim().to_string();
        let v = v.trim();
        let v = v.split(" #").next().unwrap_or(v).trim();

        let literal = v.starts_with('|');
        if v.starts_with('>') || literal {
            let mut parts: Vec<String> = Vec::new();
            while let Some(next) = lines.peek() {
                let indent = next.len() - next.trim_start().len();
                if next.trim().is_empty() || indent > 0 {
                    parts.push(lines.next().unwrap().trim().to_string());
                } else {
                    break;
                }
            }
            let joined = if literal { parts.join("\n") } else { parts.join(" ") };
            fields.push((key, joined.trim().to_string()));
        } else {
            fields.push((key, v.to_string()));
        }
    }
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

/// A rate held as a fraction, shown as a percentage (0.1723 → "17.2%").
pub fn fmt_pct(v: f64, decimals: usize) -> String {
    format!("{:.*}%", decimals, v * 100.0)
}

/// "1 variant" / "3 variants" — every count chip goes through here.
///
/// The `-y → -ies` rule only applies after a CONSONANT: "strategy → strategies"
/// but "day → days", not "daies".
pub fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        return format!("1 {noun}");
    }
    match noun.strip_suffix('y') {
        Some(stem)
            if !stem
                .chars()
                .next_back()
                .is_some_and(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')) =>
        {
            format!("{n} {stem}ies")
        }
        _ => format!("{n} {noun}s"),
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
