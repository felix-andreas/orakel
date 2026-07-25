//! The canonical signal-set schema and its (pure) CSV reader.
//!
//! One row per signal. `bid`/`ask`/`depth_*` may be empty — the engine then
//! synthesizes a book from `p_market` and the policy's `assumed_spread` and
//! marks the resulting trade `synthetic_fill` (DESIGN.md §4.1).

use crate::parse_ts;

/// Required columns, in canonical order. `asset` is an optional trailing
/// attribution column (see `signals/README.md`); when absent the engine falls
/// back to `market_slug`.
pub const SIGNAL_HEADER: [&str; 18] = [
    "signal_set",
    "t",
    "market_slug",
    "condition_id",
    "outcome",
    "token_id",
    "family",
    "variant",
    "model",
    "p_model",
    "p_market",
    "bid",
    "ask",
    "depth_bid_usd",
    "depth_ask_usd",
    "resolved_outcome",
    "resolved_date",
    "synthetic_book",
];

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    pub signal_set: String,
    /// Signal time, unix seconds UTC.
    pub t: i64,
    pub market_slug: String,
    pub condition_id: String,
    /// The outcome token this probability is for ("Yes" / "No").
    pub outcome: String,
    pub token_id: String,
    pub family: String,
    pub variant: String,
    /// Exact model id that produced `p_model`.
    pub model: String,
    pub p_model: f64,
    /// Market midpoint at `t` (recorded by convention even in thin books —
    /// see wiki/reference/thin-market-price-read.md).
    pub p_market: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    /// USD notional resting within 5c of the touch, bid side.
    pub depth_bid_usd: Option<f64>,
    /// USD notional resting within 5c of the touch, ask side.
    pub depth_ask_usd: Option<f64>,
    /// The market's winning outcome label.
    pub resolved_outcome: String,
    /// Resolution time, unix seconds UTC (a bare date parses as 12:00 UTC).
    pub resolved_at: i64,
    /// True when the row carries no real book and the engine must synthesize one.
    pub synthetic_book: bool,
    /// Optional attribution key (asset / board). Falls back to `market_slug`.
    pub asset: String,
}

impl Signal {
    /// Did the token this signal is about end up winning?
    pub fn token_won(&self) -> bool {
        self.outcome.eq_ignore_ascii_case(&self.resolved_outcome)
    }

    /// Attribution key for the per-asset breakdown.
    pub fn asset_key(&self) -> &str {
        if self.asset.is_empty() {
            &self.market_slug
        } else {
            &self.asset
        }
    }

    /// `family/variant`, the firm-wide attribution key for a strategy variant.
    pub fn variant_key(&self) -> String {
        format!("{}/{}", self.family, self.variant)
    }

    /// Claimed edge: |our probability − the market midpoint|. Policy-independent,
    /// which is what makes `edge_percentile` a property of the signal set.
    pub fn claimed_edge(&self) -> f64 {
        (self.p_model - self.p_market).abs()
    }
}

/// Parse a signal-set CSV from a string. Pure: no I/O, no clock.
///
/// Rows that cannot be parsed are returned as warnings rather than aborting the
/// run — a malformed row must never silently disappear.
pub fn parse_signals_csv(content: &str) -> Result<(Vec<Signal>, Vec<String>), String> {
    let mut warnings = Vec::new();
    let mut out = Vec::new();
    if content.trim().is_empty() {
        return Ok((out, warnings));
    }
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .comment(Some(b'#'))
        .from_reader(content.as_bytes());
    let headers = rdr.headers().map_err(|e| format!("bad header: {e}"))?.clone();
    let idx = |name: &str| headers.iter().position(|h| h.trim() == name);
    let mut missing = Vec::new();
    for name in SIGNAL_HEADER {
        if idx(name).is_none() {
            missing.push(name);
        }
    }
    if !missing.is_empty() {
        return Err(format!("missing column(s): {}", missing.join(", ")));
    }
    let col: Vec<usize> = SIGNAL_HEADER.iter().map(|n| idx(n).unwrap()).collect();
    let i_asset = idx("asset");

    for rec in rdr.records() {
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                warnings.push(format!("unreadable row: {e}"));
                continue;
            }
        };
        let line = rec.position().map(|p| p.line()).unwrap_or(0);
        let get = |i: usize| rec.get(i).unwrap_or("").trim().to_string();
        let f = |i: usize| -> Option<f64> {
            let s = rec.get(i).unwrap_or("").trim();
            if s.is_empty() {
                None
            } else {
                s.parse::<f64>().ok().filter(|v| v.is_finite())
            }
        };

        let t = match parse_ts(&get(col[1])) {
            Ok(v) => v,
            Err(e) => {
                warnings.push(format!("line {line}: {e} — row skipped"));
                continue;
            }
        };
        let resolved_at = match parse_ts(&get(col[16])) {
            Ok(v) => v,
            Err(e) => {
                warnings.push(format!("line {line}: resolved_date: {e} — row skipped"));
                continue;
            }
        };
        let (p_model, p_market) = match (f(col[9]), f(col[10])) {
            (Some(a), Some(b)) if (0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b) => (a, b),
            _ => {
                warnings.push(format!(
                    "line {line}: p_model/p_market not probabilities — row skipped"
                ));
                continue;
            }
        };
        let resolved_outcome = get(col[15]);
        if resolved_outcome.is_empty() {
            warnings.push(format!("line {line}: no resolved_outcome — row skipped"));
            continue;
        }
        let synthetic_raw = get(col[17]).to_ascii_lowercase();
        let synthetic_book = matches!(synthetic_raw.as_str(), "1" | "true" | "yes" | "y");

        out.push(Signal {
            signal_set: get(col[0]),
            t,
            market_slug: get(col[2]),
            condition_id: get(col[3]),
            outcome: get(col[4]),
            token_id: get(col[5]),
            family: get(col[6]),
            variant: get(col[7]),
            model: get(col[8]),
            p_model,
            p_market,
            bid: f(col[11]).filter(|v| (0.0..=1.0).contains(v)),
            ask: f(col[12]).filter(|v| (0.0..=1.0).contains(v)),
            depth_bid_usd: f(col[13]).filter(|v| *v >= 0.0),
            depth_ask_usd: f(col[14]).filter(|v| *v >= 0.0),
            resolved_outcome,
            resolved_at,
            synthetic_book,
            asset: i_asset.map(&get).unwrap_or_default(),
        });
    }
    Ok((out, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "signal_set,t,market_slug,condition_id,outcome,token_id,family,variant,model,p_model,p_market,bid,ask,depth_bid_usd,depth_ask_usd,resolved_outcome,resolved_date,synthetic_book,asset\n\
        s,2026-07-24T01:41:51Z,mkt-a,0xa,Yes,tok1,barrier-touch,ladder-rv,opus,0.02,0.10,0.09,0.11,500,400,No,2026-07-26,0,wti\n\
        s,2026-07-25T12:00:00Z,mkt-b,0xb,Yes,tok2,barrier-touch,ladder-rv,opus,0.40,0.30,,,,,Yes,2026-07-27T09:00:00Z,1,spy\n";

    #[test]
    fn parses_rows_and_optional_columns() {
        let (sigs, warn) = parse_signals_csv(CSV).unwrap();
        assert!(warn.is_empty(), "{warn:?}");
        assert_eq!(sigs.len(), 2);
        assert_eq!(sigs[0].bid, Some(0.09));
        assert_eq!(sigs[0].depth_ask_usd, Some(400.0));
        assert!(!sigs[0].synthetic_book);
        assert!(!sigs[0].token_won()); // Yes token, market resolved No
        assert_eq!(sigs[0].asset_key(), "wti");
        assert_eq!(sigs[0].variant_key(), "barrier-touch/ladder-rv");
        assert!((sigs[0].claimed_edge() - 0.08).abs() < 1e-12);
        // bare resolved_date -> 12:00 UTC
        assert_eq!(sigs[0].resolved_at, parse_ts("2026-07-26T12:00:00Z").unwrap());

        assert_eq!(sigs[1].bid, None);
        assert!(sigs[1].synthetic_book);
        assert!(sigs[1].token_won());
        assert_eq!(sigs[1].resolved_at, parse_ts("2026-07-27T09:00:00Z").unwrap());
    }

    #[test]
    fn bad_rows_warn_and_are_skipped_not_dropped_silently() {
        let csv = format!("{CSV}s,not-a-time,mkt-c,0xc,Yes,tok3,f,v,m,0.5,0.5,,,,,No,2026-07-26,1,x\n");
        let (sigs, warn) = parse_signals_csv(&csv).unwrap();
        assert_eq!(sigs.len(), 2);
        assert_eq!(warn.len(), 1);
        assert!(warn[0].contains("row skipped"), "{warn:?}");
    }

    #[test]
    fn missing_column_is_a_hard_error() {
        let err = parse_signals_csv("signal_set,t\n").unwrap_err();
        assert!(err.contains("missing column"), "{err}");
    }
}
