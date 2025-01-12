
// models/mempool_info.rs

use serde::{Deserialize, Serialize}; // For serializing and deserializing structures.

// Wrapper Struct - The Bitcoin RPC response wraps the actual mempoolinfo data inside the result field.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
    pub struct MempoolInfoJsonWrap {
        pub error: Option<String>,    // Optional for any error message.
        pub id: Option<String>,       // Optional Request ID.
        pub result: MempoolInfo,
}

// Represents the mempool information retrieved from the Bitcoin RPC `getmempoolinfo` call.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MempoolInfo {
      pub loaded: bool,             // Indicates if mempool data is fully loaded in memory.
      pub size: u64,                // The current number of transactions in the mempool.
      pub bytes: u64,               // The total size of all transactions in the mempool, in bytes.
      pub usage: u64,               // The total memory usage of the mempool, in bytes.
      pub total_fee: f64,           // The total fees (in BTC) of all transactions in the mempool.
      pub maxmempool: u64,          // The maximum memory capacity for the mempool, in bytes.
      pub mempoolminfee: f64,       // The minimum fee rate required to enter the mempool.
      pub minrelaytxfee: f64,       // The minimum fee rate required to be relayed to other nodes.
      pub incrementalrelayfee: f64, // The incremental fee rate used to calculate replacement cost.
      pub unbroadcastcount: u64,    // The number of transactions currently marked as unbroadcast.
      pub fullrbf: bool,            // Indicates whether the mempool accepts RBF transactions. 
}

impl MempoolInfo {
    // Converts the minrelaytxfee to vSats (satoshis per virtual byte).
    pub fn min_relay_tx_fee_vsats(&self) -> u64 {
        (self.minrelaytxfee * 100_000_000.0 / 1_000.0) as u64
    }
}
