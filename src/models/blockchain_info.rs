//! Data models for `getblockchaininfo` and helpers for interpreting
//! chain-state metrics.
//!
//! This module mirrors Bitcoin Core’s chain-state RPC response. The struct
//! layout remains intentionally close to the upstream JSON schema, ensuring:
//! - compatibility with Core version changes,
//! - transparent interpretation of consensus-critical fields,
//! - easy debugging against raw RPC output.
//!
//! Additional helper methods provide:
//! - chainwork → bits formatting,
//! - scientific notation formatting for difficulty,
//! - UNIX timestamp parsing and human-readable conversions,
//! - time-since-last-block calculations,
//! - blocks-remaining in the current difficulty epoch.

use serde::Deserialize;
use chrono::{TimeZone, Utc};
use crate::models::errors::MyError;
use tui::style::Color;
use crate::consensus::satoshi_math::*;

/// Wrapper for the response to `getblockchaininfo`.
///
/// Bitcoin Core wraps every RPC result inside `{ result, id, error }`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockchainInfoJsonWrap {
    pub error: Option<String>,
    pub id: String,
    pub result: BlockchainInfo,
}

/// Parsed result from `getblockchaininfo`.
///
/// Chain-state fields include block height, difficulty, chainwork,
/// pruned state, verification progress, and timestamps.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockchainInfo {
    pub bestblockhash: String,
    pub blocks: u64,
    pub chain: String,
    pub chainwork: String,
    pub difficulty: f64,
    #[serde(skip)]
    #[allow(dead_code)]
    pub headers: u64,
    pub initialblockdownload: bool,
    pub mediantime: u64,
    pub pruned: bool,
    pub size_on_disk: u64,
    pub time: u64,
    pub verificationprogress: f64,
    #[serde(skip)]
    #[allow(dead_code)]
    pub warnings: String,
}

impl BlockchainInfo {
    // ---------------------------------------------------------------------
    // Formatting & Interpretation Helpers
    // ---------------------------------------------------------------------

    /// Convert chainwork (hex string) into a bit count.
    ///
    /// Chainwork encodes cumulative PoW; interpreting it as `log2` allows
    /// comparison across nodes or networks.
    pub fn formatted_chainwork_bits(&self) -> Result<String, MyError> {
        u128::from_str_radix(&self.chainwork, 16)
            .map_or_else(
                |_| Err(MyError::InvalidChainworkHexString(self.chainwork.clone())),
                |decimal| {
                    let bits = (decimal as f64).log2();
                    Ok(format!("{:.2} bits", bits))
                },
            )
    }

    /// Scientific formatting for difficulty (e.g., `112.1×10¹²`).
    ///
    /// Bitcoin Core does not return difficulty in scientific notation,
    /// but the raw integer quickly becomes unreadable. This helper scales
    /// the value and annotates it with superscript powers of 10.
    /// This method intentionally deviates from standard scientific notation to maintain compactness on the TUI.
    pub fn format_scientific(value: f64) -> Result<String, MyError> {
        if value == 0.0 {
            return Ok("0.0".to_string());
        }

        let mut exponent = 0;
        let mut scaled_value = value;

        while scaled_value.abs() >= 1000.0 {
            scaled_value /= 10.0;
            exponent += 1;
        }
        while scaled_value.abs() < 100.0 {
            scaled_value *= 10.0;
            exponent -= 1;
        }

        let group_exponent = exponent / 3 * 3;
        scaled_value *= 10_f64.powi(exponent % 3);

        let superscript_map = ['⁰','¹','²','³','⁴','⁵','⁶','⁷','⁸','⁹','⁻'];

        let exp_str = group_exponent.to_string();
        let superscript: String = exp_str
            .chars()
            .filter_map(|c| c.to_digit(10).map(|d| superscript_map[d as usize]))
            .collect();

        if superscript.is_empty() {
            return Err(MyError::from_custom_error(
                "Exponent out of range for superscript formatting".to_string()
            ));
        }

        Ok(format!("{:.1}×10{}", scaled_value, superscript))
    }

    /// Format the node's difficulty using the scientific helper.
    pub fn formatted_difficulty(&self) -> Result<String, MyError> {
        Self::format_scientific(self.difficulty)
    }

    /// Convert median block time (UNIX) into ISO timestamp.
    pub fn parse_mediantime(&self) -> Result<String, MyError> {
        Utc.timestamp_opt(self.mediantime as i64, 0)
            .single()
            .map_or_else(
                || Err(MyError::InvalidMedianTime(self.mediantime)),
                |t| Ok(t.to_string()),
            )
    }

    /// Convert best block time (UNIX) into ISO timestamp.
    pub fn parse_time(&self) -> Result<String, MyError> {
        Utc.timestamp_opt(self.time as i64, 0)
            .single()
            .map_or_else(
                || Err(MyError::InvalidBlockTime(self.time)),
                |t| Ok(t.to_string()),
            )
    }

    /// Calculate the age of the best block.
    pub fn calculate_time_diff(&self) -> Result<String, MyError> {
        let now = Utc::now();
        Utc.timestamp_opt(self.time as i64, 0)
            .single()
            .map_or_else(
                || Err(MyError::InvalidBlockTime(self.time)),
                |block_time| {
                    let duration = now.signed_duration_since(block_time);
                    Ok(format!(
                        "{} hours, {} minutes ago",
                        duration.num_hours(),
                        duration.num_minutes() % SECONDS_PER_MINUTE as i64
                    ))
                },
            )
    }

    /// Blocks remaining in the 2016-block difficulty epoch.
    ///
    /// The offset adjustment occurs earlier in `rpc/blocks.rs`; this method
    /// assumes the height passed in is already aligned to Core semantics.
    /// Bitcoin Core counts block zero as the first block in epoch computations, but this 
    /// offset is handled upstream.
    pub fn blocks_until_adjustment(&self) -> Result<u64, MyError> {
        if self.blocks == 0 {
            return Err(MyError::InvalidBlockHeight(self.blocks));
        }

        Ok(DIFFICULTY_ADJUSTMENT_INTERVAL - (self.blocks % DIFFICULTY_ADJUSTMENT_INTERVAL))
    }

    /// Blocks remaining *with* a color-coded urgency indicator for the UI.
    pub fn display_blocks_until_difficulty_adjustment(&self)
        -> Result<(String, Color), MyError>
    {
        let blocks_left = self.blocks_until_adjustment()?;
        let color = match blocks_left {
            1001..=2016 => Color::Gray,
            251..=1000 => Color::Yellow,
            101..=250 => Color::Red,
            0..=100 => Color::LightRed,
            _ => Color::Gray,
        };

        Ok((blocks_left.to_string(), color))
    }
}
