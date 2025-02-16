
// models/mempool_info.rs

use serde::Deserialize; // For serializing and deserializing structures.

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
    pub average_fee_rate: f64,
}

// Wrapper Struct - The Bitcoin RPC response wraps the actual mempoolinfo data inside the result field.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
    pub struct MempoolInfoJsonWrap {
        pub error: Option<String>,    // Optional for any error message.
        pub id: Option<String>,       // Optional Request ID.
        pub result: MempoolInfo,
}

// Represents the mempool information retrieved from the Bitcoin RPC `getmempoolinfo` call.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
    pub struct RawMempoolTxsJsonWrap {
        pub error: Option<String>,    // Optional for any error message.
        pub id: Option<String>,       // Optional Request ID.
        pub result: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct MempoolEntryJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: MempoolEntry,
}

#[derive(Debug, Deserialize)]
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
    pub depends: Option<Vec<String>>,
    pub spentby: Option<Vec<String>>,
    #[serde(rename = "bip125-replaceable")]
    pub bip125_replaceable: bool,
    pub unbroadcast: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct Fees {
    pub base: f64,
    pub modified: f64,
    pub ancestor: f64,
    pub descendant: f64,
}

