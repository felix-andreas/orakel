//! Repo data access + parsing.
//!
//! Two layers:
//!
//! 1. **Reads.** `text()` fetches a repo file live from GitHub (src/live.rs).
//!    There is no fallback: `main` at request time is the only source of truth,
//!    and a read that fails returns empty text with `live: false` so the page
//!    can say so. Earlier builds compiled a copy of the repo into the Worker
//!    and served it during an outage, which made a broken dashboard look like a
//!    working one showing quietly outdated numbers.
//!    `tree()` likewise returns the live recursive listing, or nothing.
//!
//! 2. **Parsing.** `Table` for the canonical CSVs (column access by NAME, so a
//!    schema that gains a column doesn't break the dashboard) and small
//!    discovery helpers over a path listing (variants, families, run manifests).

use crate::live;
use worker::Env;

// ---------------------------------------------------------------------------
// Reads
// ---------------------------------------------------------------------------

pub struct Doc {
    pub text: String,
    /// true ⇒ came from GitHub at request time.
    pub live: bool,
}

impl Doc {
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

/// A repo file at request time. Empty text with `live: false` when the read
/// failed — never a stale copy.
pub async fn text(env: &Env, path: &str) -> Doc {
    let f = live::repo_text(env, path).await;
    Doc { text: f.text, live: f.live }
}

/// How many repo reads may be in flight at once.
///
/// Unbounded `join_all` cost us reads. Measured 2026-07-28: on the first
/// request after a push — when the SHA changes, so *every* cache entry misses
/// and every read becomes a real subrequest — reads at the tail of the batch
/// came back with **no response at all**, not a status. `/` lost two run
/// manifests and `/runs` lost three, always the oldest, i.e. the ones issued
/// last. Nothing failed in 120 requests made outside that window.
///
/// The pages that hurt are the ones that fan out over a directory (`/runs`
/// reads every manifest, `/` reads ~20 files), and those are exactly the pages
/// that grow as the firm accumulates history — so this gets worse on its own.
/// Bounding in-flight reads keeps most of the parallel win (the original serial
/// version was 0.87s warm / 2.9s cold for ~20 reads) without the burst.
const MAX_IN_FLIGHT: usize = 6;

/// Several repo files at once, in the order asked for, at most
/// `MAX_IN_FLIGHT` concurrently.
///
/// Reads on one page are independent of each other — nothing in a run manifest
/// decides which strategy TOML to fetch — so awaiting them one at a time turns
/// N independent round trips into N sequential ones.
pub async fn read_all(env: &Env, paths: impl IntoIterator<Item = String>) -> Vec<Doc> {
    use futures::stream::StreamExt;
    // `buffered` preserves input order, which callers rely on to zip results
    // back against the paths they asked for.
    futures::stream::iter(paths.into_iter().map(|p| async move { text(env, &p).await }))
        .buffered(MAX_IN_FLIGHT)
        .collect()
        .await
}

/// Every repo path. An unreadable listing yields no paths and `false`, so the
/// page shows its error rather than an empty repo.
pub async fn tree(env: &Env) -> (Vec<String>, bool) {
    match live::repo_tree(env).await {
        Some(paths) => (paths, true),
        None => (Vec::new(), false),
    }
}

// ---------------------------------------------------------------------------
// CSV
// ---------------------------------------------------------------------------

/// Minimal CSV, RFC4180 quoting supported. Our canonical prediction files never
/// quote (predictions/README.md forbids embedded commas), but generated files do
/// — `execution/results/summary.csv` states its fee model in a sentence that
/// contains commas — and a plain `split(',')` silently shifts every column after
/// it. Columns are addressed by NAME so an added column never shifts a page's
/// data; quoting makes sure the names still line up with the values.
pub struct Table {
    pub head: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// One CSV line → fields. `"` opens a quoted field, `""` inside one is a literal
/// quote. Unterminated quotes just run to the end of the line.
fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted => {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            }
            '"' if field.trim().is_empty() => {
                field.clear();
                quoted = true;
            }
            ',' if !quoted => {
                out.push(field.trim().to_string());
                field = String::new();
            }
            _ => field.push(c),
        }
    }
    out.push(field.trim().to_string());
    out
}

impl Table {
    pub fn parse(src: &str) -> Table {
        let mut lines = src
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(split_csv);
        let head = lines.next().unwrap_or_default();
        Table { head, rows: lines.collect() }
    }

    pub fn col(&self, name: &str) -> Option<usize> {
        self.head.iter().position(|h| h == name)
    }

    /// Cell by column name — "" when the column or value is missing.
    pub fn cell<'a>(&self, row: &'a [String], name: &str) -> &'a str {
        self.col(name)
            .and_then(|i| row.get(i))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn numo(&self, row: &[String], name: &str) -> Option<f64> {
        self.cell(row, name).parse::<f64>().ok()
    }

    pub fn num(&self, row: &[String], name: &str) -> f64 {
        self.numo(row, name).unwrap_or(0.0)
    }

}

/// Mean of a column over the given rows (None when there are no rows).
pub fn mean(t: &Table, rows: &[&Vec<String>], col: &str) -> Option<f64> {
    if rows.is_empty() {
        return None;
    }
    let sum: f64 = rows.iter().map(|r| t.num(r, col)).sum();
    Some(sum / rows.len() as f64)
}

// ---------------------------------------------------------------------------
// Repo structure discovery (over a path listing)
// ---------------------------------------------------------------------------

/// `strategies/<family>/<variant>/strategy.toml` → (family, variant), sorted.
/// The `_template` scaffold is not a real variant.
pub fn variants(paths: &[String]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = paths
        .iter()
        .filter_map(|p| {
            let rest = p.strip_prefix("strategies/")?.strip_suffix("/strategy.toml")?;
            let (family, variant) = rest.split_once('/')?;
            if family == "_template" || variant.contains('/') {
                return None;
            }
            Some((family.to_string(), variant.to_string()))
        })
        .collect();
    out.sort();
    out
}

/// Everything `strategy.toml` declares about one variant. Two REQUIRED
/// plain-English fields (strategies/README.md), and the dashboard never shows a
/// variant by name alone when it can show what the variant does:
///   `summary`   one line — the description in tables, lists and index rows.
///   `explainer` 4–8 sentences a smart outsider can read cold — the lede on the
///               variant's own page, where there is room for the whole idea.
pub struct VariantMeta {
    pub family: String,
    pub variant: String,
    pub summary: String,
    pub explainer: String,
    pub status: String,
    pub created: String,
    pub supersedes: String,
    pub labels: Vec<String>,
    pub slot: Option<i64>,
    pub trial_started: String,
    pub review_due: String,
    pub success_guideline: String,
    pub retired_on: String,
    pub retire_reason: String,
    /// `parked`: the thesis held, there is just no board to trade right now.
    pub parked_on: String,
    pub park_reason: String,
    pub reopen_when: String,
}

impl VariantMeta {
    pub fn key(&self) -> String {
        format!("{}/{}", self.family, self.variant)
    }
    pub fn href(&self) -> String {
        format!("/strategies/{}/{}", self.family, self.variant)
    }
}

pub fn parse_variant_meta(family: &str, variant: &str, src: &str) -> VariantMeta {
    let t = toml_of(src);
    VariantMeta {
        family: family.to_string(),
        variant: variant.to_string(),
        summary: tstr(&t, "summary").to_string(),
        explainer: tstr(&t, "explainer").trim().to_string(),
        status: tstr(&t, "status").to_string(),
        created: tstr(&t, "created").to_string(),
        supersedes: tstr(&t, "supersedes").to_string(),
        labels: tlist(&t, "labels"),
        slot: int_at(&t, &["trial", "slot"]),
        trial_started: str_at(&t, &["trial", "started"]).to_string(),
        review_due: str_at(&t, &["trial", "review_due"]).to_string(),
        success_guideline: str_at(&t, &["trial", "success_guideline"]).to_string(),
        parked_on: str_at(&t, &["parked", "date"]).to_string(),
        park_reason: str_at(&t, &["parked", "reason"]).trim().to_string(),
        reopen_when: str_at(&t, &["parked", "reopen_when"]).trim().to_string(),
        retired_on: str_at(&t, &["retirement", "date"]).to_string(),
        retire_reason: str_at(&t, &["retirement", "reason"]).to_string(),
    }
}

/// Every variant in the repo, with its manifest read. Returns `live = false`
/// if any manifest could not be read.
pub async fn variant_metas(env: &Env, paths: &[String]) -> (Vec<VariantMeta>, bool) {
    let keys = variants(paths);
    // One manifest per variant, all in flight at once. Awaited one at a time
    // this was the single largest cost on most pages — every page needs the
    // metas, and the reads have nothing to do with each other.
    let docs = read_all(
        env,
        keys.iter()
            .map(|(f, v)| format!("strategies/{f}/{v}/strategy.toml")),
    )
    .await;

    let live = docs.iter().all(|d| d.live);
    let out = keys
        .iter()
        .zip(&docs)
        .map(|((family, variant), doc)| parse_variant_meta(family, variant, &doc.text))
        .collect();
    (out, live)
}

/// `strategies/<family>/FAMILY.md` → family names, sorted.
pub fn families(paths: &[String]) -> Vec<String> {
    let mut out: Vec<String> = paths
        .iter()
        .filter_map(|p| {
            let rest = p.strip_prefix("strategies/")?.strip_suffix("/FAMILY.md")?;
            if rest.contains('/') || rest == "_template" {
                return None;
            }
            Some(rest.to_string())
        })
        .collect();
    out.sort();
    out
}

/// Files directly under `dir` matching `ext`, sorted ascending.
pub fn files_in(paths: &[String], dir: &str, ext: &str) -> Vec<String> {
    let prefix = format!("{}/", dir.trim_end_matches('/'));
    let mut out: Vec<String> = paths
        .iter()
        .filter(|p| {
            p.starts_with(&prefix) && p.ends_with(ext) && !p[prefix.len()..].contains('/')
        })
        .cloned()
        .collect();
    out.sort();
    out
}

/// `ops/runs/*.toml` manifests, NEWEST FIRST. `<date>.toml` is run 1 of the
/// day, `<date>-N.toml` run N.
pub fn run_paths(paths: &[String]) -> Vec<String> {
    let mut manifests: Vec<String> = paths
        .iter()
        .filter(|p| p.starts_with("ops/runs/") && p.ends_with(".toml"))
        .cloned()
        .collect();
    manifests.sort_by_key(|p| {
        let stem = p.trim_start_matches("ops/runs/").trim_end_matches(".toml");
        let (date, n) = if stem.len() > 10 && stem.as_bytes()[10] == b'-' {
            (&stem[..10], stem[11..].parse::<u32>().unwrap_or(1))
        } else {
            (stem, 1)
        };
        (date.to_string(), n)
    });
    manifests.reverse();
    manifests
}

/// Every wiki page path except the index, sorted.
pub fn wiki_pages(paths: &[String]) -> Vec<String> {
    let mut out: Vec<String> = paths
        .iter()
        .filter(|p| p.starts_with("wiki/") && p.ends_with(".md") && *p != "wiki/index.md")
        .cloned()
        .collect();
    out.sort();
    out
}

/// The asset a market slug belongs to: the token right after `will-`
/// (`will-wti-reach-130-in-july-2026` → `wti`). Used to group predictions into
/// boards on the dashboard; falls back to the whole slug.
pub fn asset_of(slug: &str) -> String {
    slug.strip_prefix("will-")
        .and_then(|s| s.split('-').next())
        .filter(|s| !s.is_empty())
        .unwrap_or(slug)
        .to_string()
}

/// Short human label for a market slug in a dense chart axis:
/// `will-spy-dip-to-715-by-july-20-2026` → `spy 715`.
pub fn short_market(slug: &str) -> String {
    let body = slug.strip_prefix("will-").unwrap_or(slug);
    let parts: Vec<&str> = body.split('-').collect();
    let asset = parts.first().copied().unwrap_or("");
    let level = parts
        .iter()
        .find(|p| p.chars().next().is_some_and(|c| c.is_ascii_digit()) && p.len() <= 6);
    match level {
        Some(l) => format!("{asset} {l}"),
        None => asset.to_string(),
    }
}

// ---------------------------------------------------------------------------
// TOML access helpers
// ---------------------------------------------------------------------------

pub fn toml_of(src: &str) -> toml::Table {
    src.parse::<toml::Table>().unwrap_or_default()
}

pub fn tstr<'a>(t: &'a toml::Table, key: &str) -> &'a str {
    t.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

pub fn tbool(t: &toml::Table, key: &str) -> Option<bool> {
    t.get(key).and_then(|v| v.as_bool())
}

pub fn tlist(t: &toml::Table, key: &str) -> Vec<String> {
    t.get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn value_at<'a>(root: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let mut cur = root.get(*first)?;
    for key in rest {
        cur = cur.get(key)?;
    }
    Some(cur)
}

pub fn str_at<'a>(t: &'a toml::Table, path: &[&str]) -> &'a str {
    value_at(t, path).and_then(|v| v.as_str()).unwrap_or("")
}

pub fn int_at(t: &toml::Table, path: &[&str]) -> Option<i64> {
    value_at(t, path)?.as_integer()
}

pub fn bool_at(t: &toml::Table, path: &[&str]) -> Option<bool> {
    value_at(t, path)?.as_bool()
}

pub fn arr_at<'a>(t: &'a toml::Table, path: &[&str]) -> Vec<&'a toml::Value> {
    value_at(t, path)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

pub fn table_at<'a>(t: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Table> {
    value_at(t, path)?.as_table()
}

pub fn str_list(t: &toml::Table, path: &[&str]) -> Vec<String> {
    value_at(t, path)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Path safety (user-supplied slugs / families / variants / wiki paths)
// ---------------------------------------------------------------------------

/// A single path segment we are willing to interpolate into a repo path.
pub fn safe_segment(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s != "_template"
        && s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && !s.contains("..")
}

/// A relative path (may contain `/`) under a known directory.
pub fn safe_path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 200
        && !s.contains("..")
        && !s.starts_with('/')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
}

// ---------------------------------------------------------------------------
// Dates (no deps; Howard Hinnant's algorithms)
// ---------------------------------------------------------------------------

/// Weekday name of a `YYYY-MM-DD` string ("" when unparseable).
pub fn weekday(date: &str) -> &'static str {
    let mut it = date.split('-');
    let (Some(y), Some(m), Some(d)) = (it.next(), it.next(), it.next()) else {
        return "";
    };
    let (Ok(y), Ok(m), Ok(d)) = (y.parse::<i64>(), m.parse::<i64>(), d.parse::<i64>()) else {
        return "";
    };
    let days = days_from_civil(y, m, d);
    // 1970-01-01 was a Thursday (index 4 with Sunday = 0).
    let idx = (days + 4).rem_euclid(7) as usize;
    ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"][idx]
}

pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `2026-07-25T01:52:10Z` → unix epoch milliseconds (None when unparseable).
pub fn ts_ms(ts: &str) -> Option<i64> {
    let (date, time) = ts.split_once('T')?;
    let mut dp = date.split('-');
    let y: i64 = dp.next()?.parse().ok()?;
    let m: i64 = dp.next()?.parse().ok()?;
    let d: i64 = dp.next()?.parse().ok()?;
    let t = time.trim_end_matches('Z');
    let mut tp = t.split(':');
    let hh: i64 = tp.next()?.parse().ok()?;
    let mm: i64 = tp.next().unwrap_or("0").parse().unwrap_or(0);
    let ss: i64 = tp
        .next()
        .unwrap_or("0")
        .split('.')
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    Some((days_from_civil(y, m, d) * 86_400 + hh * 3_600 + mm * 60 + ss) * 1_000)
}

/// Whole days between two `YYYY-MM-DD` strings (None when unparseable).
pub fn days_between(from: &str, to: &str) -> Option<i64> {
    let parse = |s: &str| -> Option<i64> {
        let mut it = s.split('-');
        let y = it.next()?.parse().ok()?;
        let m = it.next()?.parse().ok()?;
        let d = it.next()?.get(..2)?.parse().ok()?;
        Some(days_from_civil(y, m, d))
    };
    Some(parse(to)? - parse(from)?)
}
