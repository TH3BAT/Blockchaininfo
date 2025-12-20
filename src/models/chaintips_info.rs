//! Data models for `getchaintips`.
//!
//! Bitcoin Core's `getchaintips` RPC returns metadata describing the state
//! of all known chain branches (active chain, valid forks, headers-only
//! branches, and unknown tips).
//!
//! This information is useful for:
//! - detecting local divergence,
//! - monitoring valid forks or stale branches,
//! - visualizing multi-branch chain states,
//! - surfacing stale/side chains to the operator.
//!
//! These structs intentionally mirror Core’s response without modification.
//! Interpretation and filtering occur in higher-level modules.

use serde::Deserialize;

/// Wrapped RPC response for `getchaintips`.
///
/// Core returns a vector of chain tips — one representing the active chain
/// and zero or more representing forks or incomplete branches.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ChainTipsJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: Vec<ChainTip>,
}

/// A single chain tip.
///
/// `status` describes Core's interpretation:  
/// - `"active"`: the active best chain  
/// - `"valid-fork"`: valid but not the active chain  
/// - `"valid-headers"`: headers-only, not fully validated  
/// - `"unknown"`: Core cannot classify the branch  
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ChainTip {
    /// Height of the tip block.
    pub height: u64,

    /// Hash of the tip block.
    pub hash: String,

    /// Length of the branch since it diverged.
    /// branchlen represents how many blocks this branch is behind the active chain since the divergence point.
    pub branchlen: u64,

    /// Core-classified status of the branch.
    pub status: String,
}

