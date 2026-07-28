//! Rebuild the snapshot worker's watchlist and mirror it to R2.
//!
//! The watchlist decides which markets get an hourly book snapshot, and a book
//! that was not recorded at the time cannot be recovered afterwards. Assembling
//! it by hand has lost markets twice:
//!
//! - **2026-07-25** — it was mirrored *after* the run that produced predictions,
//!   so 18 of the first 21 scored signals had no book to check fills against.
//!   Fixed by mirroring at run start.
//!
//! - **2026-07-27** — three gold boards were predicted at ~09:00 against a
//!   watchlist mirrored at 07:05 and got no book all day. They came from
//!   *predictions*, not from an application, so "mirror when applications
//!   change" never fired.
//!
//! Both failures are the same shape: a human deciding, per run, which markets
//! belong. That decision is mechanical, so this tool makes it:
//!
//! > every market of every **active application**, UNION every market carrying
//! > an **unresolved prediction**, MINUS everything already resolved.
//!
//! Run it at the start of a run and again at the close. It is idempotent and
//! takes seconds, so there is no judgement call left to get wrong.
//!
//! ## Gamma's `closed` is a filter, not a flag
//!
//! `&closed=true` returns *only* closed markets and `&closed=false` returns
//! *only* open ones — in both directions it excludes, it does not annotate.
//! Omitting it from a resolution sweep makes the sweep structurally incapable
//! of finding a resolved market; including it in an open-market check makes
//! that check structurally incapable of finding an open one. This tool asks for
//! open markets explicitly and treats "absent from the open set" as closed.

use anyhow::{bail, Context, Result};
use clap::Parser;
use s3::{creds::Credentials, Bucket, Region};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const BUCKET: &str = "orakel";
const KEY: &str = "config/watchlist.json";
const LOCAL: &str = "ops/watchlist.json";
const GAMMA: &str = "https://gamma-api.polymarket.com";

#[derive(Parser)]
#[command(about = "Rebuild the snapshot watchlist from active applications and unresolved predictions")]
struct Args {
    /// Repo root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Build and write ops/watchlist.json, but do not upload to R2.
    #[arg(long)]
    dry_run: bool,
}

/// One market the snapshot worker should poll. Field names are the worker's
/// (`workers/snapshot/src/lib.rs`), which is the schema of record.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct WatchMarket {
    condition_id: String,
    market_slug: String,
    token_ids: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Watchlist {
    updated: String,
    /// How the list was built, so a future reader does not have to guess.
    source: String,
    markets: Vec<WatchMarket>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let root = &args.root;

    let resolved = load_resolved(root)?;
    println!("{} markets already resolved", resolved.len());

    let (apps, inactive) = load_applications(root)?;
    println!("{} active applications ({inactive} inactive, skipped)", apps.len());

    let predicted = load_predicted(root)?;
    println!("{} markets carry a prediction", predicted.len());

    // Applications name an EVENT (a whole board), so each expands to many
    // markets; predictions name a market directly. Both go through Gamma so
    // that token_ids come from the venue rather than from our own records.
    let mut markets: BTreeMap<String, WatchMarket> = BTreeMap::new();
    for slug in &apps {
        let found = event_markets(slug)?;
        if found.is_empty() {
            // Not fatal: a board can be applied for before it lists. Say so —
            // silence here is what an empty watchlist looks like from inside.
            println!("  ! {slug}: no open markets (unlisted, or fully resolved)");
        }
        for m in found {
            markets.insert(m.condition_id.clone(), m);
        }
    }
    for slug in &predicted {
        if resolved.contains(slug) {
            continue;
        }
        if markets.values().any(|m| &m.market_slug == slug) {
            continue; // already covered by an application
        }
        match market_by_slug(slug)? {
            Some(m) => {
                println!("  + {slug}: from a prediction, not covered by any application");
                markets.insert(m.condition_id.clone(), m);
            }
            None => println!("  ! {slug}: carries an unresolved prediction but is not open at Gamma"),
        }
    }

    let markets: Vec<WatchMarket> = markets
        .into_values()
        .filter(|m| !resolved.contains(&m.market_slug))
        .filter(|m| !m.token_ids.is_empty())
        .collect();
    let tokens: usize = markets.iter().map(|m| m.token_ids.len()).sum();
    if markets.is_empty() {
        bail!("refusing to write an empty watchlist — that silently stops all book collection");
    }

    let out = Watchlist {
        // Gamma's clock, so the tool needs no wall-clock of its own and the
        // stamp is comparable with the venue's own timestamps.
        updated: gamma_now()?,
        source: "active applications UNION markets with unresolved predictions, minus resolved"
            .into(),
        markets,
    };
    let json = serde_json::to_string_pretty(&out)? + "\n";

    let path = root.join(LOCAL);
    let previous = std::fs::read_to_string(&path).unwrap_or_default();
    std::fs::write(&path, &json).with_context(|| format!("writing {}", path.display()))?;
    println!("\nwrote {} — {} markets / {tokens} tokens", path.display(), out.markets.len());
    report_diff(&previous, &out.markets);

    if args.dry_run {
        println!("--dry-run: not uploaded");
        return Ok(());
    }

    let bucket = open_bucket()?;
    bucket.put_object(KEY, json.as_bytes())?;
    // Read back rather than trust the PUT: the snapshot worker reads what is in
    // R2, not what we meant to put there.
    let back = bucket.get_object(KEY)?;
    if back.as_slice() != json.as_bytes() {
        bail!("R2 readback differs from what was uploaded — watchlist may be corrupt");
    }
    println!("mirrored to r2://{BUCKET}/{KEY} (readback byte-identical)");
    Ok(())
}

/// Slugs in `predictions/resolutions.csv`. A market here is finished; keeping
/// it costs an hourly CLOB call forever and records a dead book.
fn load_resolved(root: &Path) -> Result<BTreeSet<String>> {
    let path = root.join("predictions/resolutions.csv");
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(column(&text, "market_slug"))
}

/// Slugs carrying any prediction. Filtered against `resolved` by the caller.
fn load_predicted(root: &Path) -> Result<BTreeSet<String>> {
    let path = root.join("predictions/predictions.csv");
    let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(column(&text, "market_slug"))
}

/// One column of a CSV, by header name. Values here are slugs and condition
/// ids — no embedded commas — so a full CSV parser would be ceremony.
fn column(text: &str, name: &str) -> BTreeSet<String> {
    let mut lines = text.lines();
    let Some(header) = lines.next() else {
        return BTreeSet::new();
    };
    let Some(i) = header.split(',').position(|h| h.trim() == name) else {
        return BTreeSet::new();
    };
    lines
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| l.split(',').nth(i).map(|v| v.trim().to_string()))
        .filter(|v| !v.is_empty())
        .collect()
}

/// Event slugs of every `active = true` application, plus a count of the ones
/// skipped. Returns event slugs; `event_markets` expands them.
fn load_applications(root: &Path) -> Result<(BTreeSet<String>, usize)> {
    let mut active = BTreeSet::new();
    let mut inactive = 0usize;
    let strategies = root.join("strategies");
    for family in read_dirs(&strategies)? {
        for variant in read_dirs(&family)? {
            let apps = variant.join("applications");
            if !apps.is_dir() {
                continue;
            }
            for entry in std::fs::read_dir(&apps)? {
                let p = entry?.path();
                if p.extension().is_none_or(|e| e != "toml") {
                    continue;
                }
                let text = std::fs::read_to_string(&p)?;
                let v: toml::Value =
                    toml::from_str(&text).with_context(|| format!("parsing {}", p.display()))?;
                // Absent `active` means active: applications were written
                // before the field existed and adding it must not silently
                // drop them from collection.
                if v.get("active").and_then(|a| a.as_bool()) == Some(false) {
                    inactive += 1;
                    continue;
                }
                match v.get("market_slug").and_then(|s| s.as_str()) {
                    Some(s) if !s.is_empty() => {
                        active.insert(s.to_string());
                    }
                    _ => println!("  ! {}: no market_slug", p.display()),
                }
            }
        }
    }
    Ok((active, inactive))
}

fn read_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let p = entry?.path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

/// Every OPEN market of one event. `closed=false` is a filter (see module
/// docs): closed legs simply do not come back, which is what we want.
fn event_markets(slug: &str) -> Result<Vec<WatchMarket>> {
    let url = format!("{GAMMA}/events?slug={slug}&closed=false&limit=100");
    let events: serde_json::Value = get_json(&url)?;
    let mut out = Vec::new();
    for ev in events.as_array().cloned().unwrap_or_default() {
        for m in ev["markets"].as_array().cloned().unwrap_or_default() {
            if let Some(w) = to_market(&m) {
                out.push(w);
            }
        }
    }
    Ok(out)
}

fn market_by_slug(slug: &str) -> Result<Option<WatchMarket>> {
    let url = format!("{GAMMA}/markets?slug={slug}&closed=false&limit=10");
    let markets: serde_json::Value = get_json(&url)?;
    Ok(markets
        .as_array()
        .and_then(|a| a.first())
        .and_then(to_market))
}

/// Gamma returns `clobTokenIds` as a JSON *string* holding an array, and a
/// market that has not been prepared for trading has none at all. A market with
/// no token is unpollable, so it is dropped rather than written as a stub.
fn to_market(m: &serde_json::Value) -> Option<WatchMarket> {
    let condition_id = m["conditionId"].as_str()?.to_string();
    let market_slug = m["slug"].as_str()?.to_string();
    if condition_id.is_empty() {
        return None;
    }
    let token_ids: Vec<String> = m["clobTokenIds"]
        .as_str()
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|t| !t.is_empty())
        .collect();
    Some(WatchMarket { condition_id, market_slug, token_ids })
}

/// Say what changed, both directions. A watchlist that silently shrinks is the
/// failure this tool exists to prevent, so a shrink must be visible in the log.
fn report_diff(previous: &str, now: &[WatchMarket]) {
    let before: BTreeSet<String> = serde_json::from_str::<Watchlist>(previous)
        .map(|w| w.markets.into_iter().map(|m| m.market_slug).collect())
        .unwrap_or_default();
    if before.is_empty() {
        return;
    }
    let after: BTreeSet<String> = now.iter().map(|m| m.market_slug.clone()).collect();
    for s in after.difference(&before) {
        println!("  added   {s}");
    }
    for s in before.difference(&after) {
        println!("  dropped {s}");
    }
    if before == after {
        println!("  (unchanged)");
    }
}

fn get_json(url: &str) -> Result<serde_json::Value> {
    let mut last = None;
    for attempt in 0..3 {
        match attohttpc::get(url).send() {
            Ok(r) if r.is_success() => return Ok(r.json()?),
            Ok(r) => last = Some(anyhow::anyhow!("HTTP {}", r.status())),
            Err(e) => last = Some(e.into()),
        }
        std::thread::sleep(std::time::Duration::from_secs(1 << attempt));
    }
    Err(anyhow::anyhow!("{url}: {}", last.unwrap()))
}

/// Gamma's `Date` response header. Avoids pulling in a clock dependency for one
/// informational field, and stamps the file with the venue's clock rather than
/// this container's.
fn gamma_now() -> Result<String> {
    let r = attohttpc::get(format!("{GAMMA}/markets?limit=1")).send()?;
    Ok(r.headers()
        .get("date")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string())
}

fn open_bucket() -> Result<Box<Bucket>> {
    let account = std::env::var("R2_ACCOUNT_ID").context("R2_ACCOUNT_ID not set")?;
    let creds = Credentials::new(
        Some(&std::env::var("R2_ACCESS_KEY_ID").context("R2_ACCESS_KEY_ID not set")?),
        Some(&std::env::var("R2_SECRET_ACCESS_KEY").context("R2_SECRET_ACCESS_KEY not set")?),
        None,
        None,
        None,
    )?;
    let region = Region::Custom {
        region: "auto".to_string(),
        endpoint: format!("https://{account}.r2.cloudflarestorage.com"),
    };
    Ok(Bucket::new(BUCKET, region, creds)?.with_path_style())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_reads_by_name_not_position() {
        let csv = "a,market_slug,c\n1,foo,3\n4,bar,6\n";
        let got = column(csv, "market_slug");
        assert_eq!(got, ["bar".to_string(), "foo".to_string()].into_iter().collect());
    }

    #[test]
    fn column_missing_is_empty_not_panic() {
        assert!(column("a,b\n1,2\n", "market_slug").is_empty());
        assert!(column("", "market_slug").is_empty());
    }

    #[test]
    fn clob_token_ids_are_a_json_string() {
        let m = serde_json::json!({
            "conditionId": "0xabc",
            "slug": "will-x",
            "clobTokenIds": "[\"111\",\"222\"]",
        });
        let w = to_market(&m).unwrap();
        assert_eq!(w.token_ids, vec!["111".to_string(), "222".to_string()]);
    }

    #[test]
    fn market_without_tokens_is_kept_here_and_dropped_by_the_caller() {
        let m = serde_json::json!({ "conditionId": "0xabc", "slug": "will-x" });
        assert!(to_market(&m).unwrap().token_ids.is_empty());
    }
}
