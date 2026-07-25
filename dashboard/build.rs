//! Build script: emits a compile-time build timestamp, and nothing else.
//!
//! Earlier versions also staged an EMBEDDED REPO PACK — every renderable repo
//! file concatenated into the binary — so the Worker could still render pages
//! when GitHub was unreachable. That fallback was removed on 2026-07-25: it
//! made an outage look like a working dashboard showing quietly outdated
//! numbers, and it cost ~1 MiB of the Worker bundle. `main` at request time is
//! now the only source of truth, and a failed read is shown as an error.

use std::env;

fn main() {
    // Build timestamp (UTC, RFC3339). Computed here, at compile time — the
    // Worker never calls Date::now for this.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", rfc3339_utc(secs));

    // Nothing outside src/ affects the build any more, so repo edits no longer
    // force a Worker rebuild.
    println!("cargo:rerun-if-changed=build.rs");
    let _ = env::var("OUT_DIR");
}

/// Seconds since the epoch → `YYYY-MM-DDTHH:MM:SSZ`, civil-from-days.
fn rfc3339_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    // Howard Hinnant's civil_from_days, shifted to a March-based year.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}
