//! Adapter: ladder-rv's frozen resolved-leg checkpoints -> `signals/ladder-rv-hist.csv`.
//!
//! Source is an R2-frozen backtest archive of the `barrier-touch/ladder-rv`
//! variant. Recipe (re-runnable from a clean checkout):
//!
//! ```sh
//! tools/r2data/target/release/r2data pull \
//!   strategies/barrier-touch/ladder-rv/data/backtest-metals-2026-07-25.tar.gz.r2.json \
//!   --out /tmp/lrv.tar.gz
//! mkdir -p /tmp/lrv && tar xzf /tmp/lrv.tar.gz -C /tmp/lrv
//! build-ladder-rv-hist --data /tmp/lrv \
//!   --manifest strategies/barrier-touch/ladder-rv/data/backtest-metals-2026-07-25.tar.gz.r2.json \
//!   --out execution/signals/ladder-rv-hist.csv
//! ```
//!
//! Columns consumed (`ladderrv analyze` output — see that variant's src/main.rs):
//!
//! - `out/gate2_checkpoints.csv`: one row per leg per daily 12:00Z checkpoint —
//!   `q_rv` (the variant's primary model probability, driftless GBM on trailing
//!   14d realized vol), `mid` (CLOB midpoint at t), `winner`.
//! - `legs.csv`: `condition_id`, `token_yes`, `we` (window end), `winner`.
//! - `out/gate0.csv`: `first_touch` — when a touched barrier actually resolved.
//!   Gate 0 reproduced every one of these legs from candle data, so it is the
//!   resolution record, not a guess.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

const FAMILY: &str = "barrier-touch";
const VARIANT: &str = "ladder-rv";
/// `p_model` is produced by the variant's first-passage model, not by an LLM.
const MODEL: &str = "ladderrv-rv14d";

fn main() -> Result<()> {
    let mut data: Option<PathBuf> = None;
    let mut manifest: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--data" => data = args.next().map(PathBuf::from),
            "--manifest" => manifest = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--help" | "-h" => {
                println!("usage: build-ladder-rv-hist --data <extracted-archive-dir> --manifest <*.r2.json> --out <csv>");
                return Ok(());
            }
            other => bail!("unknown argument '{other}'"),
        }
    }
    let data = data.context("--data is required")?;
    let out = out.context("--out is required")?;

    let legs = read_csv(&data.join("legs.csv"))?;
    let gate0 = read_csv(&data.join("out/gate0.csv"))?;
    let cps = read_csv(&data.join("out/gate2_checkpoints.csv"))?;

    let leg_by_slug: HashMap<&str, &HashMap<String, String>> =
        legs.rows.iter().map(|r| (r["market_slug"].as_str(), r)).collect();
    let touch_by_slug: HashMap<&str, &HashMap<String, String>> =
        gate0.rows.iter().map(|r| (r["market_slug"].as_str(), r)).collect();

    let mut rows: Vec<String> = Vec::with_capacity(cps.rows.len());
    let mut skipped = 0usize;
    let mut touched = 0usize;
    for cp in &cps.rows {
        let slug = &cp["market_slug"];
        let (Some(leg), Some(g0)) = (leg_by_slug.get(slug.as_str()), touch_by_slug.get(slug.as_str()))
        else {
            eprintln!("warning: {slug}: no legs.csv/gate0.csv row — checkpoint skipped");
            skipped += 1;
            continue;
        };
        let won = leg["winner"] == "1";
        if won {
            touched += 1;
        }
        // Hold-to-resolution ends when the position actually ends: at the first
        // touch for a leg that touched, at window end for one that did not.
        let resolved_at: i64 = if won {
            g0["first_touch"].parse().context("first_touch")?
        } else {
            leg["we"].parse().context("we")?
        };
        let t: i64 = cp["t"].parse().context("checkpoint t")?;
        if resolved_at <= t {
            // A checkpoint at or after its own resolution cannot be traded.
            skipped += 1;
            continue;
        }
        rows.push(
            [
                "ladder-rv-hist".to_string(),
                engine::fmt_ts(t),
                slug.clone(),
                leg["condition_id"].clone(),
                "Yes".to_string(),
                leg["token_yes"].clone(),
                FAMILY.to_string(),
                VARIANT.to_string(),
                MODEL.to_string(),
                cp["q_rv"].clone(),
                cp["mid"].clone(),
                String::new(), // bid  — no historical book exists for these legs
                String::new(), // ask
                String::new(), // depth_bid_usd
                String::new(), // depth_ask_usd
                if won { "Yes" } else { "No" }.to_string(),
                engine::fmt_ts(resolved_at),
                "1".to_string(), // synthetic_book
                cp["asset"].clone(),
            ]
            .join(","),
        );
    }

    let header = format!("{},asset", engine::SIGNAL_HEADER.join(","));
    let body = format!("{header}\n{}\n", rows.join("\n"));
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p).ok();
    }
    std::fs::write(&out, &body).with_context(|| format!("writing {}", out.display()))?;

    let sidecar = out.with_extension("source.json");
    let manifest_json: serde_json::Value = match &manifest {
        Some(m) => serde_json::from_str(&std::fs::read_to_string(m)?)
            .with_context(|| format!("parsing {}", m.display()))?,
        None => serde_json::Value::Null,
    };
    let meta = serde_json::json!({
        "signal_set": "ladder-rv-hist",
        "built_by": "engine/src/bin/build-ladder-rv-hist.rs",
        "engine_version": engine::ENGINE_VERSION,
        "r2_manifest_path": manifest.as_ref().map(|m| m.display().to_string()),
        "r2_manifest": manifest_json,
        "inputs": [
            {"file": "legs.csv", "sha256": legs.sha256, "rows": legs.rows.len()},
            {"file": "out/gate0.csv", "sha256": gate0.sha256, "rows": gate0.rows.len()},
            {"file": "out/gate2_checkpoints.csv", "sha256": cps.sha256, "rows": cps.rows.len()},
        ],
        "output_rows": rows.len(),
        "skipped_rows": skipped,
        "checkpoints_on_touched_legs": touched,
        "conventions": {
            "p_model": "gate2_checkpoints.q_rv — the variant's RV-primary first-passage probability",
            "p_market": "gate2_checkpoints.mid — CLOB midpoint at the checkpoint",
            "book": "none: the archive stores CLOB price history, not books. Every row is synthetic_book=1.",
            "resolved_date": "gate0.first_touch for touched legs, legs.we for untouched",
        },
    });
    std::fs::write(&sidecar, serde_json::to_string_pretty(&meta)? + "\n")?;

    println!(
        "{} rows -> {} ({} skipped)\n{}",
        rows.len(),
        out.display(),
        skipped,
        sidecar.display()
    );
    Ok(())
}

struct Table {
    rows: Vec<HashMap<String, String>>,
    sha256: String,
}

/// Read a CSV into name->value maps. These files are small enough (a few MB)
/// that clarity beats streaming.
fn read_csv(path: &Path) -> Result<Table> {
    let bytes = std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let text = String::from_utf8(bytes).context("not utf-8")?;
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(text.as_bytes());
    let headers: Vec<String> =
        rdr.headers()?.iter().map(|h| h.trim().to_string()).collect();
    let mut rows = Vec::new();
    for rec in rdr.records() {
        let rec = rec?;
        let mut m = HashMap::with_capacity(headers.len());
        for (i, h) in headers.iter().enumerate() {
            m.insert(h.clone(), rec.get(i).unwrap_or("").trim().to_string());
        }
        rows.push(m);
    }
    Ok(Table { rows, sha256 })
}
