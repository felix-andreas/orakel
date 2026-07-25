//! Policy files: the named execution strategies in `execution/policies/*.toml`.
//!
//! Parsing is pure (`from_toml(&str)`), so a dashboard can hand the engine a
//! policy typed into a text box. Policy files are authored by the CEO and are
//! never edited in place — a change means a new version file (DESIGN.md §5).

use serde::Deserialize;

/// Engine-side defaults for fields the policy files do not carry. They are
/// documented in `engine/README.md`; changing one changes every result, so they
/// live here in one place and are printed in the result JSON.
pub const DEFAULT_BANKROLL_USD: f64 = 1000.0;
/// Polymarket's minimum ticket. Depth that cannot fund this is `unfundable`.
pub const DEFAULT_MIN_STAKE_USD: f64 = 1.0;
/// How stale a "delay_hours later" observation may be before it stops counting
/// as that observation (hours).
pub const DEFAULT_DELAY_TOLERANCE_HOURS: f64 = 12.0;
/// Depth is measured within this distance of the touch (DESIGN.md §4.2), which
/// also sets the scale of the linear slippage penalty.
pub const DEPTH_BAND: f64 = 0.05;

#[derive(Debug, Clone, Deserialize)]
pub struct Policy {
    pub name: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub character: String,
    #[serde(default)]
    pub combine: Combine,
    pub entry: Entry,
    pub sizing: Sizing,
    pub exit: Exit,
    #[serde(default)]
    pub costs: Costs,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Combine {
    #[serde(default = "default_combine")]
    pub method: String,
}

impl Default for Combine {
    fn default() -> Self {
        Combine { method: default_combine() }
    }
}

fn default_combine() -> String {
    "best-improvement".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    /// Minimum |our probability − the executable price|.
    pub min_edge: f64,
    /// Keep only signals whose *claimed* edge (|p_model − p_market|) is at or
    /// above this quantile of the whole signal set.
    #[serde(default)]
    pub edge_percentile: Option<f64>,
    pub sides: Vec<String>,
    #[serde(default)]
    pub delay_hours: f64,
    #[serde(default)]
    pub delay_tolerance_hours: Option<f64>,
    /// `true` = a synthetic (mid + assumed_spread) fill is still allowed but is
    /// counted and flagged; the depth gate then cannot be evaluated and the row
    /// is counted as `depth_unknown`. See engine/README.md — this is the single
    /// most consequential interpretive choice in the engine.
    #[serde(default)]
    pub require_book: bool,
    /// Maximum acceptable ask − bid.
    pub min_spread_ok: f64,
    pub min_depth_usd: f64,
    #[serde(default)]
    pub respect_venue_epsilon: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sizing {
    /// `flat` | `fractional-kelly`
    pub method: String,
    #[serde(default)]
    pub stake_usd: Option<f64>,
    #[serde(default)]
    pub kelly_fraction: Option<f64>,
    #[serde(default)]
    pub bankroll_usd: Option<f64>,
    #[serde(default)]
    pub max_bankroll_fraction: Option<f64>,
    pub max_per_market_usd: f64,
    #[serde(default)]
    pub min_stake_usd: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Exit {
    /// `hold-to-resolution` | `take-profit`
    pub rule: String,
    #[serde(default)]
    pub close_fraction: Option<f64>,
    #[serde(default = "default_true")]
    pub else_hold_to_resolution: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct Costs {
    #[serde(default = "default_spread")]
    pub assumed_spread: f64,
    #[serde(default = "default_book_fraction")]
    pub max_book_fraction: f64,
    #[serde(default)]
    pub fee_bps: f64,
}

impl Default for Costs {
    fn default() -> Self {
        Costs {
            assumed_spread: default_spread(),
            max_book_fraction: default_book_fraction(),
            fee_bps: 0.0,
        }
    }
}

fn default_spread() -> f64 {
    0.03
}

fn default_book_fraction() -> f64 {
    1.0
}

impl Policy {
    /// Parse a policy TOML. Pure.
    pub fn from_toml(text: &str) -> Result<Policy, String> {
        let p: Policy = toml::from_str(text).map_err(|e| e.to_string())?;
        p.validate()?;
        Ok(p)
    }

    fn validate(&self) -> Result<(), String> {
        match self.sizing.method.as_str() {
            "flat" => {
                if self.sizing.stake_usd.is_none() {
                    return Err("sizing.method = flat needs stake_usd".to_string());
                }
            }
            "fractional-kelly" => {
                if self.sizing.kelly_fraction.is_none() {
                    return Err("sizing.method = fractional-kelly needs kelly_fraction".to_string());
                }
            }
            m => return Err(format!("unknown sizing.method '{m}'")),
        }
        match self.exit.rule.as_str() {
            "hold-to-resolution" => {}
            "take-profit" => {
                if self.exit.close_fraction.is_none() {
                    return Err("exit.rule = take-profit needs close_fraction".to_string());
                }
            }
            r => return Err(format!("unknown exit.rule '{r}'")),
        }
        if !(self.combine.method == "best-improvement"
            || self.combine.method.starts_with("single:"))
        {
            return Err(format!(
                "unsupported combine.method '{}' (supported: best-improvement, single:<family>/<variant>)",
                self.combine.method
            ));
        }
        for s in &self.sides_normalized() {
            if s != "buy" && s != "sell" {
                return Err(format!("unknown entry side '{s}'"));
            }
        }
        Ok(())
    }

    pub fn sides_normalized(&self) -> Vec<String> {
        self.entry.sides.iter().map(|s| s.to_ascii_lowercase()).collect()
    }

    pub fn bankroll(&self) -> f64 {
        self.sizing.bankroll_usd.unwrap_or(DEFAULT_BANKROLL_USD)
    }

    pub fn min_stake(&self) -> f64 {
        self.sizing.min_stake_usd.unwrap_or(DEFAULT_MIN_STAKE_USD)
    }

    pub fn delay_tolerance_hours(&self) -> f64 {
        self.entry.delay_tolerance_hours.unwrap_or(DEFAULT_DELAY_TOLERANCE_HOURS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIRROR: &str = r#"
name = "mirror"
version = 1
[combine]
method = "best-improvement"
[entry]
min_edge = 0.01
sides = ["buy", "sell"]
delay_hours = 0
require_book = false
min_spread_ok = 1.00
min_depth_usd = 0.0
respect_venue_epsilon = false
[sizing]
method = "flat"
stake_usd = 10.0
max_per_market_usd = 100.0
[exit]
rule = "hold-to-resolution"
[costs]
assumed_spread = 0.03
max_book_fraction = 1.0
fee_bps = 0
"#;

    #[test]
    fn parses_a_real_policy_and_applies_defaults() {
        let p = Policy::from_toml(MIRROR).unwrap();
        assert_eq!(p.name, "mirror");
        assert_eq!(p.sizing.stake_usd, Some(10.0));
        assert_eq!(p.bankroll(), DEFAULT_BANKROLL_USD);
        assert_eq!(p.min_stake(), DEFAULT_MIN_STAKE_USD);
        assert_eq!(p.entry.edge_percentile, None);
        assert!(p.exit.else_hold_to_resolution);
    }

    #[test]
    fn rejects_incoherent_policies() {
        let bad = MIRROR.replace("method = \"flat\"", "method = \"martingale\"");
        assert!(Policy::from_toml(&bad).unwrap_err().contains("unknown sizing.method"));
        let bad = MIRROR.replace("stake_usd = 10.0\n", "");
        assert!(Policy::from_toml(&bad).unwrap_err().contains("needs stake_usd"));
        let bad = MIRROR.replace("\"best-improvement\"", "\"precision-weighted\"");
        assert!(Policy::from_toml(&bad).unwrap_err().contains("unsupported combine.method"));
        let bad = MIRROR.replace("[\"buy\", \"sell\"]", "[\"hedge\"]");
        assert!(Policy::from_toml(&bad).unwrap_err().contains("unknown entry side"));
    }
}
