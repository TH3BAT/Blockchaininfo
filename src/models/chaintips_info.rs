
// models/chain_tips.rs

use serde::Deserialize;

/// Wrapper Struct - The Bitcoin RPC response wraps the actual getchaintips data inside the result field.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ChainTipsResponse {
    pub error: Option<String>,    // Optional for any error message.
    pub id: Option<String>,       // Optional Request ID.
    pub result: Vec<ChainTip>,
}

/// This struct holds data from getchaintips RPC method.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ChainTip {
    pub height: u64,
    pub hash: String,
    pub branchlen: u64,
    pub status: String, // Can be "active", "valid-fork", "valid-headers", or "unknown".
}

