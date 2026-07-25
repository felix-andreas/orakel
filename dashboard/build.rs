//! Build script: emits a compile-time build timestamp and stages the embedded
//! REPO PACK — one file containing every repo text file the dashboard can
//! render, so `include_str!` needs exactly one path and the Worker has a
//! complete offline fallback for every page (not just the handful of files v2
//! hardcoded).
//!
//! Pack format (parsed by src/data.rs):
//!   \x1e<relative path>\n<file body>\n\x1e<next path>\n...
//! \x1e (record separator) never occurs in our markdown/TOML/CSV; any stray
//! one is replaced with a space here so the format stays unambiguous.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Repo root relative to the crate dir.
const ROOT: &str = "..";

/// Directories walked for the pack. Each is also a `rerun-if-changed` watch.
const ROOTS: [&str; 7] = [
    "ops",
    "predictions",
    "ideas",
    "wiki",
    "strategies",
    "roles",
    "execution",
];

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR set by cargo"));

    // Build timestamp (UTC, RFC3339). Computed here, at compile time — the
    // Worker never calls Date::now for this.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", rfc3339_utc(secs));

    let mut files: Vec<String> = Vec::new();
    for root in ROOTS {
        println!("cargo:rerun-if-changed={ROOT}/{root}");
        collect(&PathBuf::from(ROOT).join(root), root, &mut files);
    }
    files.sort();

    let mut pack = String::with_capacity(512 * 1024);
    for rel in &files {
        let Ok(body) = fs::read_to_string(PathBuf::from(ROOT).join(rel)) else {
            continue;
        };
        pack.push('\u{1e}');
        pack.push_str(rel);
        pack.push('\n');
        pack.push_str(&body.replace('\u{1e}', " "));
        if !body.ends_with('\n') {
            pack.push('\n');
        }
    }
    fs::write(out_dir.join("pack.txt"), &pack).expect("write embedded repo pack");
    println!(
        "cargo:warning=embedded repo pack: {} files, {} KiB",
        files.len(),
        pack.len() / 1024
    );
}

/// Recursively collect wanted files under `dir`, pushing repo-relative paths.
fn collect(dir: &Path, rel_dir: &str, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let rel = format!("{rel_dir}/{name}");
        let path = entry.path();
        if path.is_dir() {
            // `data/` holds R2 manifests and frozen datasets, `target/` build
            // output. Neither is ever rendered.
            if name == "data" || name == "target" || name.starts_with('.') {
                continue;
            }
            collect(&path, &rel, out);
        } else if wanted(&rel) {
            out.push(rel);
        }
    }
}

/// Allowlist: markdown / TOML / CSV under the rendered trees, plus the
/// execution engine's RESULTS (including the per-run JSON the equity curves are
/// drawn from — the only JSON in the pack). Role folders contribute inbox
/// messages only (playbooks and private memory stay out). The engine's own
/// source tree and the raw signal sets are deliberately excluded:
/// `execution/signals/ladder-rv-hist.csv` alone is 1.8 MiB and nothing renders
/// it.
fn wanted(rel: &str) -> bool {
    if rel.ends_with("Cargo.toml") {
        return false;
    }
    let text = rel.ends_with(".md") || rel.ends_with(".toml") || rel.ends_with(".csv");
    match rel.split('/').next().unwrap_or("") {
        "ops" | "predictions" | "ideas" | "wiki" | "strategies" => text,
        "roles" => text && rel.contains("/inbox/"),
        "execution" => {
            if rel.starts_with("execution/results/") {
                text || rel.ends_with(".json")
            } else {
                text
                    && (rel.starts_with("execution/policies/")
                        || rel == "execution/DESIGN.md"
                        || rel == "execution/README.md"
                        || rel == "execution/signals/README.md")
            }
        }
        _ => false,
    }
}

/// Days-from-epoch to civil date (Howard Hinnant's algorithm), no deps.
fn rfc3339_utc(secs: u64) -> String {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}
