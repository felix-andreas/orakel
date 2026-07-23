//! Plain format!-based HTML rendering helpers. No template engine — data in,
//! HTML string out (CODING.md: procedural, simple).

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

pub struct NavItem {
    pub href: &'static str,
    pub label: &'static str,
}

pub const NAV: [NavItem; 6] = [
    NavItem { href: "/", label: "Operations" },
    NavItem { href: "/predictions", label: "Predictions" },
    NavItem { href: "/snapshots", label: "Snapshots" },
    NavItem { href: "/inboxes", label: "Inboxes" },
    NavItem { href: "/wiki", label: "Wiki" },
    NavItem { href: "/dev", label: "Development" },
];

/// Shared page shell: sidebar nav (burger dropdown on mobile via the
/// checkbox hack — no JS), header, content, footer.
/// `active` is the href of the current page; `body` is already-safe HTML.
pub fn layout(active: &str, title: &str, desc: &str, body: &str, build_ts: &str) -> String {
    let mut nav = String::new();
    for item in NAV.iter() {
        let current = if item.href == active { " aria-current=\"page\"" } else { "" };
        nav.push_str(&format!(
            "<a href=\"{}\"{}>{}</a>",
            item.href, current, item.label
        ));
    }
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
</head>
<body>
<div class="shell">
  <input type="checkbox" id="nav-toggle" class="nav-toggle">
  <aside class="sidebar">
    <div class="sidebar-top">
      <a class="wordmark" href="/">orakel<span> / dashboard</span></a>
      <label class="burger" for="nav-toggle" aria-label="Toggle navigation"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h16"/></svg></label>
    </div>
    <nav class="nav">{nav}</nav>
    <div class="sidebar-foot">build-time snapshot</div>
  </aside>
  <div class="main">
    <main class="content">
      <header class="page-header">
        <h1 class="page-title">{title}</h1>
        <p class="page-desc">{desc}</p>
      </header>
      {body}
    </main>
    <footer class="footer">
      <span>orakel — agentic prediction-market research firm</span>
      <span>built {build_ts} · data embedded at build time</span>
    </footer>
  </div>
</div>
</body>
</html>"#,
        title = esc(title),
        desc = esc(desc),
        nav = nav,
        body = body,
        build_ts = esc(build_ts),
    )
}

/// Flat content section: heading + already-safe inner HTML. The shadcn-like
/// default — prefer this (headings + whitespace) over cards; a card is for
/// the rare case where boxed grouping genuinely earns its border.
pub fn section(title: &str, inner_html: &str) -> String {
    format!(
        "<section class=\"section\"><h2 class=\"section-title\">{}</h2>{}</section>",
        esc(title),
        inner_html
    )
}

/// Small flat sub-grouping inside a section/tab: muted uppercase label +
/// content. Used e.g. for the Operations State groups (no boxes).
pub fn subsection(title: &str, inner_html: &str) -> String {
    format!(
        "<section class=\"subsection\"><h3 class=\"subsection-title\">{}</h3>{}</section>",
        esc(title),
        inner_html
    )
}

/// Muted one-line reference to the repo file a block renders (flat, no box).
pub fn file_ref(path: &str, desc: &str) -> String {
    format!(
        "<p class=\"file-ref\"><span class=\"mono\">{}</span> · {}</p>",
        esc(path),
        esc(desc)
    )
}

/// A card with a title, optional description, and pre-rendered content HTML.
pub fn card(title: &str, desc: &str, content_html: &str) -> String {
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<p class=\"card-desc\">{}</p>", esc(desc))
    };
    format!(
        "<div class=\"card\"><div class=\"card-header\"><h3 class=\"card-title\">{}</h3>{}</div><div class=\"card-content\">{}</div></div>",
        esc(title),
        desc_html,
        content_html
    )
}

/// CSS-only tab component (shadcn Tabs look): hidden radio inputs drive which
/// label pill is active and which panel shows, via nth-of-type pair rules in
/// style.css. No JS, no page reload; radios stay keyboard-accessible (arrow
/// keys switch tabs). `id` must be unique per page. The first pane is the
/// default tab. Pane HTML must be already-safe. style.css pairs up to 6 tabs —
/// extend its selector lists before passing more.
pub fn tabs(id: &str, panes: &[(&str, String)]) -> String {
    debug_assert!(panes.len() <= 6, "style.css tab pair rules cover 6 tabs");
    let mut inputs = String::new();
    let mut labels = String::new();
    let mut panels = String::new();
    for (i, (label, pane_html)) in panes.iter().enumerate() {
        let input_id = format!("tab-{}-{}", id, i);
        inputs.push_str(&format!(
            "<input class=\"tab-input\" type=\"radio\" name=\"tab-{}\" id=\"{}\"{}>",
            esc(id),
            esc(&input_id),
            if i == 0 { " checked" } else { "" },
        ));
        labels.push_str(&format!(
            "<label for=\"{}\">{}</label>",
            esc(&input_id),
            esc(label)
        ));
        panels.push_str(&format!(
            "<section class=\"tab-panel\">{}</section>",
            pane_html
        ));
    }
    format!(
        "<div class=\"tabs\">{}<div class=\"tab-list\">{}</div>{}</div>",
        inputs, labels, panels
    )
}

/// Key/value row for inside cards. `value_html` is already-safe HTML.
pub fn kv(key: &str, value_html: &str) -> String {
    format!(
        "<div class=\"kv\"><span class=\"k\">{}</span><span class=\"v\">{}</span></div>",
        esc(key),
        value_html
    )
}

/// tone: "" (neutral) | "ok" | "warn" | "bad"
pub fn badge(text: &str, tone: &str) -> String {
    let class = match tone {
        "ok" => "badge badge-ok",
        "warn" => "badge badge-warn",
        "bad" => "badge badge-bad",
        _ => "badge",
    };
    format!("<span class=\"{}\">{}</span>", class, esc(text))
}

pub fn badge_row(badges: &[String]) -> String {
    format!("<div class=\"badge-row\">{}</div>", badges.join(""))
}

pub fn empty_state(title: &str, detail_html: &str) -> String {
    format!(
        "<div class=\"empty\"><strong>{}</strong>{}</div>",
        esc(title),
        detail_html
    )
}

pub fn note(html_inner: &str) -> String {
    format!("<p class=\"note\">{}</p>", html_inner)
}

/// Minimal CSV split: comma-separated, no quoting support. Fine for orakel's
/// canonical CSVs, whose writers never emit quoted or embedded commas
/// (predictions/README.md schema). Revisit if a field ever needs quoting.
pub fn parse_csv(src: &str) -> Vec<Vec<String>> {
    src.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|f| f.trim().to_string()).collect())
        .collect()
}

/// Render parsed CSV rows (first row = header) as a table. Long opaque ids
/// (hex condition ids, decimal token ids — >28 chars, no '-') are abbreviated
/// with the full value in a hover title, so real prediction rows stay
/// readable; slugs and timestamps (which contain '-') are never touched.
pub fn csv_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut out = String::from("<div class=\"table-wrap\"><table class=\"data\"><thead><tr>");
    for h in &rows[0] {
        out.push_str(&format!("<th>{}</th>", esc(h)));
    }
    out.push_str("</tr></thead><tbody>");
    for row in &rows[1..] {
        out.push_str("<tr>");
        for cell in row {
            let is_opaque_id =
                cell.chars().count() > 28 && cell.chars().all(|c| c.is_ascii_alphanumeric());
            if is_opaque_id {
                let head: String = cell.chars().take(8).collect();
                let tail: String = {
                    let n = cell.chars().count();
                    cell.chars().skip(n - 4).collect()
                };
                out.push_str(&format!(
                    "<td title=\"{}\"><span class=\"mono\">{}…{}</span></td>",
                    esc(cell),
                    esc(&head),
                    esc(&tail)
                ));
            } else {
                out.push_str(&format!("<td>{}</td>", esc(cell)));
            }
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table></div>");
    out
}

/// Header columns as monospace chips (for empty-state schema display).
pub fn chip_row(items: &[String]) -> String {
    let chips: String = items
        .iter()
        .map(|c| format!("<span class=\"chip\">{}</span>", esc(c)))
        .collect();
    format!("<div class=\"chip-row\">{}</div>", chips)
}
