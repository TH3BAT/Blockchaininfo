//! Data models and derived metrics for Bitcoin mempool state.
//!
//! This module mirrors the RPC structures returned by:
//! - `getmempoolinfo`
//! - `getrawmempool`
//! - `getmempoolentry`
//!
//! These models serve two roles:
//! 1. **Raw RPC mirrors** — deserialize responses exactly as Core returns them.
//! 2. **Derived analytics** — produce distributions and fee statistics that power
//!    BlockchainInfo’s mempool visualizations.
//!
//! The mempool distribution logic intentionally avoids modifying the raw
//! structures. It instead computes meaningful summaries such as:
//! - vsize segmentation (small / medium / large)
//! - age segmentation (young / moderate / old)
//! - RBF vs. non-RBF counts
//! - average/median fees
//! - fee-per-vbyte estimates
//!
//! Core philosophy: keep raw RPC models pure, push "interpretation" upward.

use serde::Deserialize;
use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};

//
// ────────────────────────────────────────────────────────────────────────────────
//   Derived Mempool Distribution
// ────────────────────────────────────────────────────────────────────────────────
//

/// Represents the computed mempool distribution used in the dashboard.
///
/// This struct is *not* part of Core's RPC response — it is calculated from
/// all loaded `MempoolEntry` items after dust filtering.
///
/// The segmentation rules are intentionally simple and stable:
/// - vsize buckets: 0–249, 250–1000, 1000+  
/// - age buckets: <5 min, 5–60 min, >60 min
///
/// This keeps the dashboard interpretable across all node types.
#[derive(Default)]
pub struct MempoolDistribution {
    pub small: usize,
    pub medium: usize,
    pub large: usize,

    pub young: usize,
    pub moderate: usize,
    pub old: usize,

    pub rbf_count: usize,
    pub non_rbf_count: usize,

    pub average_fee: f64,
    pub median_fee: f64,

    /// Fee rate estimate: (total fees / total vsize)
    /// Expressed as sats/vB.
    pub average_fee_rate: f64,
}

impl MempoolDistribution {
    /// Updates the distribution metrics using all entries in the mempool cache.
    ///
    /// Assumes the caller has already filtered out dust if needed.
    /// This function is intentionally CPU-light; it should run every refresh cycle.
    pub fn update_metrics(&mut self, cache: &DashMap<[u8; 32], MempoolEntry>) {
        let mut small = 0;
        let mut medium = 0;
        let mut large = 0;

        let mut young = 0;
        let mut moderate = 0;
        let mut old = 0;

        let mut rbf_count = 0;
        let mut non_rbf_count = 0;

        let mut total_fee = 0.0;
        let mut total_vsize = 0;
        let mut count = 0;

        let mut fees: Vec<f64> = Vec::new();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for entry in cache.iter() {
            let e = entry.value();

            // vsize segmentation
            match e.vsize {
                0..=249 => small += 1,
                250..=1000 => medium += 1,
                _ => large += 1,
            }

            // age segmentation
            let age = now.saturating_sub(e.time);
            match age {
                0..=300 => young += 1,
                301..=3600 => moderate += 1,
                _ => old += 1,
            }

            // RBF replaceability
            if e.bip125_replaceable {
                rbf_count += 1;
            } else {
                non_rbf_count += 1;
            }

            // Fee aggregation
            let fee = e.fees.base + e.fees.ancestor + e.fees.modified + e.fees.descendant;
            total_fee += fee;
            total_vsize += e.vsize;
            fees.push(fee);

            count += 1;
        }

        // Assign
        self.small = small;
        self.medium = medium;
        self.large = large;

        self.young = young;
        self.moderate = moderate;
        self.old = old;

        self.rbf_count = rbf_count;
        self.non_rbf_count = non_rbf_count;

        self.average_fee = if count > 0 { total_fee / count as f64 } else { 0.0 };

        // Median fee
        self.median_fee = if !fees.is_empty() {
            fees.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mid = fees.len() / 2;
            if fees.len() % 2 == 0 {
                (fees[mid - 1] + fees[mid]) / 2.0
            } else {
                fees[mid]
            }
        } else {
            0.0
        };

        // sats/vB (Core fee fields are denominated in BTC, not sats.)
        self.average_fee_rate = if total_vsize > 0 {
            (total_fee * 100_000_000.0) / total_vsize as f64
        } else {
            0.0
        };
    }
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   RPC: getmempoolinfo
// ────────────────────────────────────────────────────────────────────────────────
//

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct MempoolInfoJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: MempoolInfo,
}

/// Mirror of Core's `getmempoolinfo` response.
///
/// These values describe global mempool state (memory usage, min fees, RBF mode).
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct MempoolInfo {
    pub loaded: bool,
    pub size: u64,
    pub bytes: u64,
    pub usage: u64,
    pub total_fee: f64,
    pub maxmempool: u64,
    pub mempoolminfee: f64,
    pub minrelaytxfee: f64,
    pub incrementalrelayfee: f64,
    pub unbroadcastcount: u64,
    pub fullrbf: bool,
}

impl MempoolInfo {
    /// Convert Core’s fee rate (BTC/kB) into sats/vB.
    ///
    /// Core expresses fee rates in:
    ///     BTC/kB
    ///
    /// We convert:
    ///     sats/vB = (BTC * 1e8) / 1000
    pub fn min_relay_tx_fee_vsats(&self) -> u64 {
        (self.minrelaytxfee * 100_000_000.0 / 1000.0) as u64
    }
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   RPC: getrawmempool
// ────────────────────────────────────────────────────────────────────────────────
//

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct RawMempoolTxsJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: Vec<String>, // TXIDs only
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   RPC: getmempoolentry
// ────────────────────────────────────────────────────────────────────────────────
//

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct MempoolEntryJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: MempoolEntry,
}

/// Full mempool entry data.
/// Mirrors Bitcoin Core exactly (`getmempoolentry`).
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct MempoolEntry {
    pub vsize: u64,
    pub weight: u64,
    pub time: u64,
    pub height: u64,
    pub descendantcount: u64,
    pub descendantsize: u64,
    pub ancestorcount: u64,
    pub ancestorsize: u64,
    pub wtxid: String,
    pub fees: Fees,
    #[serde(skip)]
    #[allow(dead_code)]
    pub depends: Option<Vec<String>>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub spentby: Option<Vec<String>>,

    #[serde(rename = "bip125-replaceable")]
    pub bip125_replaceable: bool,

    pub unbroadcast: Option<bool>,
}

/// Fee structure mirrored directly from Core.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct Fees {
    pub base: f64,
    pub modified: f64,
    pub ancestor: f64,
    pub descendant: f64,
}
