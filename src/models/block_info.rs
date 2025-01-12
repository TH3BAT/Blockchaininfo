
// models/block_info.rs

use serde::{Serialize, Deserialize};  // For serializing and deserializing structures.

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHash {
    pub result: String,           // The block hash is a plain string.
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockInfoJsonWrap {
    pub result: BlockInfo,        // Contains the block's details.
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockInfo {
     pub hash: String,             // Block hash.
     pub confirmations: u64,       // Number of confirmations.
     pub height: u64,              // Block height.
     pub version: u32,             // Block version.
     #[serde(rename = "versionHex")]
     pub version_hex: String,       // Block version in hex.
     pub merkleroot: String,       // Merkle root of the block.
     pub time: u64,                // Block timestamp.
     pub mediantime: u64,          // Median block time.
     pub nonce: u64,               // Nonce used for mining.
     pub bits: String,             // Difficulty target.
     pub difficulty: f64,          // Current difficulty.
     pub chainwork: String,        // Chain work as a hex string.
     #[serde(rename = "nTx")]
     pub n_tx: u32,                 // Number of transactions in the block.
     pub previousblockhash: Option<String>, // Hash of the previous block.
     pub nextblockhash: Option<String>,     // Hash of the next block.
     pub strippedsize: u64,        // Stripped size of the block.
     pub size: u64,                // Total size of the block.
     pub weight: u64,              // Block weight.
     pub tx: Vec<String>,          // List of transaction IDs.
}



