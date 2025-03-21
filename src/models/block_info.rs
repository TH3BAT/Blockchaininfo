
// models/block_info.rs

use serde::Deserialize;  // For serializing and deserializing structures.
use std::collections::{VecDeque, HashMap};
use std::sync::{Mutex, Arc};

/// This struct holds block hash from getblockhash RPC method.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockHash {
    pub error: Option<String>,    // Optional for any error message.
    pub id: Option<String>,       // Optional Request ID.
    pub result: String,           // The block hash is a plain string.
}

/// Wrapper Struct - The Bitcoin RPC response wraps the actual getblock data inside the result field (verbose = 1).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfoJsonWrap {
    pub result: BlockInfo,        // Contains the block's details.
}

/// This struct holds data from getblock RPC method (verbose = 1).
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfo {
    pub hash: String,             // Block hash.
    pub confirmations: u64,       // Number of confirmations.
    pub height: u64,              // Block height.
    pub version: u32,             // Block version.
    #[serde(rename = "versionHex")]
    pub version_hex: String,      // Block version in hex.
    pub merkleroot: String,       // Merkle root of the block.
    pub time: u64,                // Block timestamp.
    pub mediantime: u64,          // Median block time.
    pub nonce: u64,               // Nonce used for mining.
    pub bits: String,             // Difficulty target.
    pub difficulty: f64,          // Current difficulty.
    pub chainwork: String,        // Chain work as a hex string.
    #[serde(rename = "nTx")]
    pub n_tx: u32,                // Number of transactions in the block.
    pub previousblockhash: Option<String>, // Hash of the previous block.
    pub nextblockhash: Option<String>,     // Hash of the next block.
    pub strippedsize: u64,        // Stripped size of the block.
    pub size: u64,                // Total size of the block.
    pub weight: u64,              // Block weight.
    pub tx: Vec<String>,          // List of transaction IDs (for verbose=1).
}

/// Wrapper Struct - The Bitcoin RPC response wraps the actual getblock data inside the result field (verbose = 2).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfoFullJsonWrap {
    pub result: BlockInfoFull,        // Contains the block's details.
}

/// This struct holds data from getblock RPC method (verbose = 2).
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfoFull {
    pub hash: String,             // Block hash.
    pub confirmations: u64,       // Number of confirmations.
    pub height: u64,              // Block height.
    pub version: u32,             // Block version.
    #[serde(rename = "versionHex")]
    pub version_hex: String,      // Block version in hex.
    pub merkleroot: String,       // Merkle root of the block.
    pub time: u64,                // Block timestamp.
    pub mediantime: u64,          // Median block time.
    pub nonce: u64,               // Nonce used for mining.
    pub bits: String,             // Difficulty target.
    pub difficulty: f64,          // Current difficulty.
    pub chainwork: String,        // Chain work as a hex string.
    #[serde(rename = "nTx")]
    pub n_tx: u32,                // Number of transactions in the block.
    pub previousblockhash: Option<String>, // Hash of the previous block.
    pub nextblockhash: Option<String>,     // Hash of the next block.
    pub strippedsize: u64,        // Stripped size of the block.
    pub size: u32,                // Total size of the block.
    pub weight: u32,              // Block weight.
    #[serde(default)]
    pub tx: Vec<Transaction>, // Full transaction details (for verbose=2).
}

/// This struct holds full transaction data from getblock (verbose = 2).
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    pub txid: String,             
    pub hash: String,
    pub version: u32,
    pub size: u32,
    pub vsize: u32,
    pub weight: u32,
    pub locktime: u64,
    pub vin: Vec<TxIn>,
    pub vout:Vec<TxOut>,         
}

impl Transaction {
    /// Extracts wallet addresses from the transaction's outputs originating from block info, verbose = 2.
    /// Filters out outputs with empty addresses.
    pub fn extract_wallet_addresses(&self) -> Vec<String> {
        self.vout
            .iter()
            .filter(|output| !output.script_pub_key.address.is_empty())
            .map(|output| output.script_pub_key.address.clone())
            .collect()
    }
}

/// This struct holds TxOut data from getblock (verbose = 2).
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct TxOut {
    pub value: f64,               
    n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: ScriptPubKey, 
}

/// This struct holds ScriptPubKey data from getblock (verbose = 2).
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ScriptPubKey {
    pub asm: String,              
    pub desc: String,
    pub hex: String,              
    #[serde(default)]
    pub address: String, 
    pub r#type: String,        
}

/// This struct holds TxIn data from getblock (verbose = 2).
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct TxIn {
    #[serde(default)]
    pub coinbase: Option<String>,
    #[serde(default)]
    pub txid: Option<String>,
    pub vout: Option<u32>,   
    #[serde(rename = "scriptSig")]
    pub script_sig: Option<ScriptSig>,
    #[serde(default)]     
    pub txinwitness: Option<Vec<String>>,
    pub sequence: u32,       
}

/// This struct holds ScriptSig data from getblock (verbose = 2).
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct ScriptSig {
    pub asm: String,              
    pub hex: String,       
}

/// This struct holds our known miners data used for Hash Rate Distribution chart and last miner.
#[derive(Deserialize, Clone)]
pub struct MinersData {
    pub miners: Vec<Miner>,
}

/// This struct holds miner name and known wallet address.
#[derive(Deserialize, Clone)]
pub struct Miner {
    pub name: String,
    pub wallet: String,
}

/// This struct stores a rolling 24-hour miner history for hash rate distribution chart and last miner.
pub struct BlockHistory {
    pub blocks: Mutex<VecDeque<Option<Arc<str>>>>, // Thread-safe rolling window
}

impl BlockHistory {
    pub fn new() -> Self {
        BlockHistory {
            blocks: Mutex::new(VecDeque::with_capacity(144)),
        }
    }

     /// Returns the last miner inserted, or `None` if the buffer is empty.
     pub fn last_miner(&self) -> Option<Arc<str>> {
        let blocks = self.blocks.lock().unwrap(); // Lock the Mutex
        blocks.back().cloned()? // Get the last element and clone it
    }

    /// Add latest miner.
    pub fn add_block(&self, miner: Option<String>) {
        let mut blocks = self.blocks.lock().unwrap();
        if blocks.len() == 144 {
            blocks.pop_front(); // Remove the oldest block
        }
        blocks.push_back(miner.map(|m| Arc::from(m.into_boxed_str()))); // Add the new block
    }

    /// Returns miner(s) and total blocks mined past 144 blocks.
    pub fn get_miner_distribution(&self) -> Vec<(Arc<str>, u64)> {
        let blocks = self.blocks.lock().unwrap().clone(); // Clone the VecDeque and release the lock
        let mut distribution = HashMap::new();
        for miner in blocks.iter().flatten() {
            *distribution.entry(miner.clone()).or_insert(0) += 1;
        }
        distribution
            .into_iter()
            // .map(|(k, v)| (k.to_string(), v)) // Convert &str to String only once
            .collect()
    }
}
