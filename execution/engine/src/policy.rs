//! Policy files: the named execution strategies in `execution/policies/*.toml`.
//!
//! Parsing is pure (`from_toml(&str)`), so a dashboard can hand the engine a
//! policy typed into a text box. Policy files are authored by the CEO and are
//! never edited in place — a change means a new version file (DESIGN.md §5).

use std::collections::BTreeMap;

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

/// How the venue's trading fee is charged (DESIGN.md §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeeModel {
    /// No venue fee at all. What every `-v1` policy assumed — and it was wrong:
    /// Polymarket has charged taker fees since 2026-01-05. Kept as the default
    /// so the v1 files keep their exact original meaning and their results stay
    /// attributable, and so a fee-free counterfactual remains expressible.
    #[default]
    None,
    /// Polymarket's published taker fee, charged **per taker fill**:
    /// `fee_usdc = shares × rate × p × (1 − p)` with `rate` set per market
    /// category. See `taker_fee` in `sim.rs` for the implementation and
    /// `wiki/recipes/polymarket-api.md` for the venue reference.
    PolymarketTaker,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Costs {
    #[serde(default = "default_spread")]
    pub assumed_spread: f64,
    #[serde(default = "default_book_fraction")]
    pub max_book_fraction: f64,
    /// **Legacy.** A flat fee in basis points of the fill notional. No venue we
    /// trade prices fees this way; it is retained only so the `-v1` policy files
    /// (which all carry `fee_bps = 0`) keep their literal meaning rather than
    /// having a field silently ignored. Charged *in addition* to `fee_model`.
    #[serde(default)]
    pub fee_bps: f64,
    #[serde(default)]
    pub fee_model: FeeModel,
    /// Category → taker rate, e.g. `crypto = 0.07`. Read off the venue's own
    /// `feeSchedule.rate`, not invented.
    #[serde(default)]
    pub fee_rate: BTreeMap<String, f64>,
    /// Signal `asset` key → category name used in `fee_rate`.
    #[serde(default)]
    pub asset_category: BTreeMap<String, String>,
    /// Rate applied to an asset with no `asset_category` entry. Mandatory under
    /// `polymarket-taker`: an unmapped asset must never quietly trade fee-free.
    /// Every trade priced at this fallback is counted (`fee_rate_unmapped`).
    #[serde(default)]
    pub fee_rate_default: Option<f64>,
}

impl Default for Costs {
    fn default() -> Self {
        Costs {
            assumed_spread: default_spread(),
            max_book_fraction: default_book_fraction(),
            fee_bps: 0.0,
            fee_model: FeeModel::None,
            fee_rate: BTreeMap::new(),
            asset_category: BTreeMap::new(),
            fee_rate_default: None,
        }
    }
}

impl Costs {
    /// The taker rate for one signal's asset, and whether it came from an
    /// explicit mapping (`false` = the `fee_rate_default` fallback was used).
    pub fn fee_rate_for(&self, asset: &str) -> (f64, bool) {
        if self.fee_model == FeeModel::None {
            return (0.0, true);
        }
        let key = asset.trim().to_ascii_lowercase();
        if let Some(cat) = self.asset_category.get(&key) {
            if let Some(rate) = self.fee_rate.get(cat.as_str()) {
                return (*rate, true);
            }
        }
        (self.fee_rate_default.unwrap_or(0.0), false)
    }

    /// Human-readable model name for the result JSON and the summary.
    pub fn fee_model_label(&self) -> String {
        match self.fee_model {
            FeeModel::None if self.fee_bps > 0.0 => format!("none + {:.4} bps notional", self.fee_bps),
            FeeModel::None => "none (fee-free — the venue's real taker fee is NOT modelled)".to_string(),
            FeeModel::PolymarketTaker => {
                let mut rates: Vec<String> =
                    self.fee_rate.iter().map(|(c, r)| format!("{c} {r}")).collect();
                rates.sort();
                format!("polymarket-taker: shares x rate x p x (1-p) per fill [{}]", rates.join(", "))
            }
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
        self.validate_fees()
    }

    /// A fee model that can silently charge nothing is worse than no fee model
    /// at all — that is the bug this whole version exists to fix. So the
    /// `polymarket-taker` model refuses to load unless every rate it could
    /// possibly need is declared.
    fn validate_fees(&self) -> Result<(), String> {
        let c = &self.costs;
        if c.fee_bps < 0.0 {
            return Err("costs.fee_bps must not be negative".to_string());
        }
        for (cat, rate) in &c.fee_rate {
            if !(*rate >= 0.0 && *rate <= 1.0) {
                return Err(format!("costs.fee_rate.{cat} = {rate} is not a rate in [0,1]"));
            }
        }
        if c.fee_model == FeeModel::None {
            return Ok(());
        }
        if c.fee_rate.is_empty() {
            return Err(
                "costs.fee_model = polymarket-taker needs a [costs.fee_rate] table".to_string()
            );
        }
        match c.fee_rate_default {
            None => {
                return Err(
                    "costs.fee_model = polymarket-taker needs costs.fee_rate_default, so an unmapped asset cannot trade fee-free by accident".to_string()
                )
            }
            Some(r) if !(0.0..=1.0).contains(&r) => {
                return Err(format!("costs.fee_rate_default = {r} is not a rate in [0,1]"))
            }
            Some(_) => {}
        }
        for (asset, cat) in &c.asset_category {
            if !c.fee_rate.contains_key(cat) {
                return Err(format!(
                    "costs.asset_category.{asset} = '{cat}' has no rate in [costs.fee_rate]"
                ));
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
