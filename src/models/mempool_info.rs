
// models/mempool_info.rs

use serde::Deserialize; // For serializing and deserializing structures.
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

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

impl MempoolDistribution {
    pub fn update_metrics(&mut self, cache: &HashMap<String, MempoolEntry>) {
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

        for mempool_entry in cache.values() {
            match mempool_entry.vsize {
                0..=249 => small += 1,
                250..=1000 => medium += 1,
                _ => large += 1,
            }

            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let age = current_time.saturating_sub(mempool_entry.time);

            match age {
                0..=300 => young += 1,
                301..=3600 => moderate += 1,
                _ => old += 1,
            }

            if mempool_entry.bip125_replaceable {
                rbf_count += 1;
            } else {
                non_rbf_count += 1;
            }

            let total_entry_fee = mempool_entry.fees.base
                + mempool_entry.fees.ancestor
                + mempool_entry.fees.modified
                + mempool_entry.fees.descendant;

            total_fee += total_entry_fee;
            total_vsize += mempool_entry.vsize;
            fees.push(total_entry_fee);
            count += 1;
        }

        self.small = small;
        self.medium = medium;
        self.large = large;
        self.young = young;
        self.moderate = moderate;
        self.old = old;
        self.rbf_count = rbf_count;
        self.non_rbf_count = non_rbf_count;
        self.average_fee = if count > 0 { total_fee / count as f64 } else { 0.0 };
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
        self.average_fee_rate = if total_vsize > 0 {
            (total_fee * 100_000_000.0) / total_vsize as f64
        } else {
            0.0
        };
    }
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

