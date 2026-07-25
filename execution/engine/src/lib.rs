//! orakel execution simulator — the pure core.
//!
//! `simulate(signals, policy) -> SimResult` is a pure function over plain
//! structs: no filesystem, no network, no clock. Everything that touches the
//! outside world lives in `src/main.rs` (the CLI) and `src/bin/*` (the signal
//! set adapters), so this library compiles to wasm32 unchanged.
//!
//! The accounting rule that decides everything is DESIGN.md §3:
//!
//! - BUY YES at ask `a`: capital committed = `a × shares`; payoff `1` if the
//!   token wins, `0` otherwise.
//! - SELL YES at bid `b` (economically: buy NO at `1 − b`): cash received =
//!   `b × shares`, collateral locked = `(1 − b) × shares` until exit. Profit on
//!   a winning wing sale is the whole premium `b`, and the capital tied up is
//!   `1 − b` — which is why cents/trade flatters exactly the trades we should
//!   refuse, and why annualized return on locked capital is the number that
//!   decides.

pub mod metrics;
pub mod policy;
pub mod signal;
pub mod sim;

pub use metrics::{EquityPoint, Group, Metrics, SimResult};
pub use policy::{Combine, Costs, Entry, Exit, Policy, Sizing};
pub use signal::{parse_signals_csv, Signal, SIGNAL_HEADER};
pub use sim::{simulate, Counts, ExitKind, Side, Trade};

/// Engine version stamped into every result file, so a result can always be
/// traced to the code that produced it.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Round to 6 decimals so committed JSON/CSV stays free of round-trip noise
/// like `0.049999999999999996`. Same convention as `scoring/`.
pub fn r6(v: f64) -> f64 {
    if !v.is_finite() {
        return v;
    }
    (v * 1e6).round() / 1e6
}

/// Format a float with at most 6 decimal places, trailing zeros trimmed.
/// Copied from `scoring/src/main.rs` so the CSVs look the same across the firm.
pub fn fmt_f(v: f64) -> String {
    if !v.is_finite() {
        return String::new();
    }
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

/// Parse a timestamp that is either RFC3339 (`2026-07-24T01:41:51Z`) or a bare
/// date (`2026-07-24`). A bare date is taken as **12:00 UTC** that day — the
/// firm-wide convention already used by `scoring/` for `resolved_date`.
pub fn parse_ts(s: &str) -> Result<i64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty timestamp".to_string());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp());
    }
    match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        Ok(d) => Ok(d.and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp()),
        Err(e) => Err(format!("bad timestamp '{s}': {e}")),
    }
}

/// Format a unix timestamp as RFC3339 UTC with second precision.
pub fn fmt_ts(t: i64) -> String {
    match chrono::DateTime::from_timestamp(t, 0) {
        Some(dt) => dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        None => String::new(),
    }
}

/// Format a unix timestamp as a bare UTC date.
pub fn fmt_date(t: i64) -> String {
    match chrono::DateTime::from_timestamp(t, 0) {
        Some(dt) => dt.format("%Y-%m-%d").to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_round_trip() {
        assert_eq!(parse_ts("2026-07-24T01:41:51Z").unwrap(), 1784857311);
        assert_eq!(fmt_ts(1784857311), "2026-07-24T01:41:51Z");
        // bare date -> 12:00 UTC (scoring's convention)
        assert_eq!(
            parse_ts("2026-07-24").unwrap(),
            parse_ts("2026-07-24T12:00:00Z").unwrap()
        );
        assert!(parse_ts("nonsense").is_err());
        assert!(parse_ts("").is_err());
    }

    #[test]
    fn float_formatting_matches_scoring() {
        assert_eq!(fmt_f(0.049999999999999996), "0.05");
        assert_eq!(fmt_f(1.0), "1");
        assert_eq!(fmt_f(-0.0000001), "0");
        assert_eq!(fmt_f(f64::NAN), "");
    }
}
