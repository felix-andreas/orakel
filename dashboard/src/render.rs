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

/// A single navigable page (a sidebar leaf / former tab).
pub struct NavLeaf {
    pub href: &'static str,
    pub label: &'static str,
}

/// A sidebar group. A group with one leaf renders as a plain top-level link
/// (label → that leaf); a group with several leaves renders as a foldable
/// `<details>` disclosure whose open/closed state persists to localStorage
/// (see the script in `layout`). `id` is the localStorage key suffix.
pub struct NavGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub leaves: &'static [NavLeaf],
}

pub const NAV: [NavGroup; 7] = [
    NavGroup {
        id: "operations",
        label: "Operations",
        leaves: &[
            NavLeaf { href: "/", label: "State" },
            NavLeaf { href: "/decisions", label: "Decisions" },
            NavLeaf { href: "/runs", label: "Runs" },
            NavLeaf { href: "/ideas", label: "Ideas" },
        ],
    },
    NavGroup {
        id: "strategies",
        label: "Strategies",
        leaves: &[NavLeaf { href: "/strategies", label: "Strategies" }],
    },
    NavGroup {
        id: "predictions",
        label: "Predictions",
        leaves: &[
            NavLeaf { href: "/predictions", label: "Log" },
            NavLeaf { href: "/resolutions", label: "Resolutions" },
            NavLeaf { href: "/scores", label: "Scores" },
        ],
    },
    NavGroup {
        id: "snapshots",
        label: "Snapshots",
        leaves: &[NavLeaf { href: "/snapshots", label: "Snapshots" }],
    },
    NavGroup {
        id: "inboxes",
        label: "Inboxes",
        leaves: &[NavLeaf { href: "/inboxes", label: "Inboxes" }],
    },
    NavGroup {
        id: "wiki",
        label: "Wiki",
        leaves: &[NavLeaf { href: "/wiki", label: "Wiki" }],
    },
    NavGroup {
        id: "development",
        label: "Development",
        leaves: &[
            NavLeaf { href: "/dev", label: "Charts" },
            NavLeaf { href: "/dev/endpoints", label: "Endpoints" },
        ],
    },
];

const CHEVRON_SVG: &str = "<svg class=\"nav-chevron\" width=\"14\" height=\"14\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2.2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m9 6 6 6-6 6\"/></svg>";

/// Render the grouped sidebar navigation. Single-leaf groups become plain
/// links; multi-leaf groups become foldable `<details>` (the group holding the
/// active page is rendered `open`). Persistence is layered on in JS by
/// `layout`; without JS the native disclosure still folds.
fn sidebar_nav(active: &str) -> String {
    let mut out = String::new();
    for g in NAV.iter() {
        if g.leaves.len() == 1 {
            let leaf = &g.leaves[0];
            let current = if leaf.href == active {
                " aria-current=\"page\""
            } else {
                ""
            };
            out.push_str(&format!(
                "<a class=\"nav-link nav-solo\" href=\"{}\"{}>{}</a>",
                leaf.href,
                current,
                esc(g.label)
            ));
            continue;
        }
        let has_active = g.leaves.iter().any(|l| l.href == active);
        let mut sub = String::new();
        for leaf in g.leaves {
            let current = if leaf.href == active {
                " aria-current=\"page\""
            } else {
                ""
            };
            sub.push_str(&format!(
                "<a class=\"nav-link nav-sub\" href=\"{}\"{}>{}</a>",
                leaf.href,
                current,
                esc(leaf.label)
            ));
        }
        out.push_str(&format!(
            "<details class=\"nav-group\" data-nav=\"{}\"{}><summary class=\"nav-group-head\"><span>{}</span>{}</summary><div class=\"nav-sublist\">{}</div></details>",
            esc(g.id),
            if has_active { " open" } else { "" },
            esc(g.label),
            CHEVRON_SVG,
            sub
        ));
    }
    out
}

/// Persists each nav group's open/closed state to localStorage. Progressive
/// enhancement only: native `<details>` folds without it. The group holding
/// the active page always renders open and ignores a stored "collapsed".
const NAV_PERSIST_JS: &str = r#"<script>
(function () {
  try {
    document.querySelectorAll(".nav-group").forEach(function (d) {
      var key = "orakel.nav." + d.getAttribute("data-nav");
      var active = !!d.querySelector('[aria-current="page"]');
      if (!active) {
        var stored = localStorage.getItem(key);
        if (stored === "0") d.open = false;
        else if (stored === "1") d.open = true;
      }
      d.addEventListener("toggle", function () {
        try { localStorage.setItem(key, d.open ? "1" : "0"); } catch (e) {}
      });
    });
  } catch (e) {}
})();
</script>"#;

/// Shared page shell: grouped sidebar nav (foldable groups persisted to
/// localStorage; burger dropdown on mobile via the checkbox hack), header,
/// content, footer. `active` is the href of the current page; `body` is
/// already-safe HTML.
pub fn layout(active: &str, title: &str, desc: &str, body: &str, build_ts: &str) -> String {
    let nav = sidebar_nav(active);
    let desc_html = if desc.is_empty() {
        String::new()
    } else {
        format!("<p class=\"page-desc\">{}</p>", esc(desc))
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
    <div class="sidebar-foot">agentic prediction-market research firm</div>
  </aside>
  <div class="main">
    <main class="content">
      <header class="page-header">
        <h1 class="page-title">{title}</h1>
        {desc_html}
      </header>
      {body}
    </main>
    <footer class="footer">
      <span>orakel — agentic prediction-market research firm</span>
      <span>built {build_ts} · server-rendered from repo state</span>
    </footer>
  </div>
</div>
{NAV_PERSIST_JS}
</body>
</html>"#,
        title = esc(title),
        desc_html = desc_html,
        nav = nav,
        body = body,
        build_ts = esc(build_ts),
        NAV_PERSIST_JS = NAV_PERSIST_JS,
    )
}

/// Breadcrumb trail for detail pages. `crumbs` are (href, label) pairs; the
/// last is rendered as the current (unlinked) location.
pub fn breadcrumb(crumbs: &[(&str, &str)]) -> String {
    let mut out = String::from("<nav class=\"crumbs\" aria-label=\"Breadcrumb\">");
    for (i, (href, label)) in crumbs.iter().enumerate() {
        if i > 0 {
            out.push_str("<span class=\"crumb-sep\" aria-hidden=\"true\">/</span>");
        }
        if i + 1 == crumbs.len() || href.is_empty() {
            out.push_str(&format!("<span class=\"crumb-current\">{}</span>", esc(label)));
        } else {
            out.push_str(&format!("<a href=\"{}\">{}</a>", esc(href), esc(label)));
        }
    }
    out.push_str("</nav>");
    out
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

/// Polymarket permalink for a market/event slug. Our `market_slug` values are
/// the gamma-API event slugs (see the variants' discover code), so the public
/// URL is `/event/<slug>`.
pub fn market_url(slug: &str) -> String {
    format!("https://polymarket.com/event/{}", slug)
}

/// External link opening in a new tab, with the outbound arrow affordance.
pub fn ext_link(href: &str, text: &str) -> String {
    format!(
        "<a class=\"ext\" href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{}<span class=\"ext-mark\" aria-hidden=\"true\">↗</span></a>",
        esc(href),
        esc(text)
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

/// Render parsed CSV rows (first row = header) as a table. Cells in a column
/// headed `market_slug` or `market` become Polymarket links. Long opaque ids
/// (hex condition ids, decimal token ids — >28 chars, no '-') are abbreviated
/// with the full value in a hover title, so real prediction rows stay
/// readable; slugs and timestamps (which contain '-') are never touched.
pub fn csv_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let header = &rows[0];
    // Column indices whose values are market slugs → linkify to Polymarket.
    let is_market_col: Vec<bool> = header
        .iter()
        .map(|h| matches!(h.as_str(), "market_slug" | "market"))
        .collect();
    let mut out = String::from("<div class=\"table-wrap\"><table class=\"data\"><thead><tr>");
    for h in header {
        out.push_str(&format!("<th>{}</th>", esc(h)));
    }
    out.push_str("</tr></thead><tbody>");
    for row in &rows[1..] {
        out.push_str("<tr>");
        for (i, cell) in row.iter().enumerate() {
            let is_market = is_market_col.get(i).copied().unwrap_or(false)
                && !cell.is_empty()
                && cell != "—"
                && cell != "?";
            if is_market {
                out.push_str(&format!(
                    "<td class=\"cell-market\"><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{}<span class=\"ext-mark\" aria-hidden=\"true\">↗</span></a></td>",
                    esc(&market_url(cell)),
                    esc(cell)
                ));
                continue;
            }
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
