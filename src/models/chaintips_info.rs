
// models/chain_tips.rs

use serde::{Deserialize, Serialize};

// Wraps the response for deserialization.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChainTipsResponse {
    pub result: Vec<ChainTip>,
}

// Represents a single chain tip.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChainTip {
    pub height: u64,
    pub hash: String,
    pub branchlen: u64,
    pub status: String, // Can be "active", "valid-fork", "valid-headers", or "unknown".
}

