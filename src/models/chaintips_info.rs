
// models/chain_tips.rs

use serde::Deserialize;

// Wraps the response for deserialization.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ChainTipsResponse {
    pub error: Option<String>,    // Optional for any error message.
    pub id: Option<String>,       // Optional Request ID.
    pub result: Vec<ChainTip>,
}

// Represents a single chain tip.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ChainTip {
    pub height: u64,
    pub hash: String,
    pub branchlen: u64,
    pub status: String, // Can be "active", "valid-fork", "valid-headers", or "unknown".
}

