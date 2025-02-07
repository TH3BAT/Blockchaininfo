
// rpc/mempool_distro.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::mempool_info::{MempoolEntryJsonWrap, MempoolEntry};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
// use once_cell::sync::Lazy;
use std::sync::Arc;
use std::collections::HashMap;

const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC
const MAX_CACHE_SIZE: usize = 10_000; // Rolling cache size

static DUST_FREE_TX_CACHE: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, MempoolEntry>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn fetch_mempool_distribution(
    config: &RpcConfig,
    all_tx_ids: Vec<String>,
) -> Result<((usize, usize, usize), (usize, usize, usize), (usize, usize), f64, f64, f64), MyError> {
    let client = Client::new();
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

    let mut cache = DUST_FREE_TX_CACHE.lock().await;
    let new_tx_ids: Vec<String> = all_tx_ids
        .into_iter()
        .filter(|txid| !cache.contains_key(txid))
        .collect();

    // ✅ **Step 1: Update Cache (Only Add Dust-Free TXs)**
    for tx_id in new_tx_ids.iter() {
        let json_rpc_request = json!({
            "jsonrpc": "1.0",
            "id": "1",
            "method": "getmempoolentry",
            "params": [tx_id]
        });

        let response = client
            .post(&config.address)
            .basic_auth(&config.username, Some(&config.password))
            .header(CONTENT_TYPE, "application/json")
            .json(&json_rpc_request)
            .send()
            .await?
            .json::<MempoolEntryJsonWrap>()
            .await?;

        let mempool_entry = response.result;

        if mempool_entry.fees.base < DUST_THRESHOLD {
            continue;
        }

        cache.insert(tx_id.clone(), mempool_entry);

        if cache.len() > MAX_CACHE_SIZE {
            let oldest_key = cache.keys().next().cloned();
            if let Some(oldest_key) = oldest_key {
                cache.remove(&oldest_key);
            }
        }
    }

    // ✅ **Step 2: Process Cached Transactions & Calculate Distribution**
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

    let average_fee = if count > 0 { total_fee / count as f64 } else { 0.0 };
    let median_fee = if !fees.is_empty() {
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

    let average_fee_rate = if total_vsize > 0 {
        (total_fee * 100_000_000.0) / total_vsize as f64
    } else {
        0.0
    };

    Ok((
        (small, medium, large),
        (young, moderate, old),
        (rbf_count, non_rbf_count),
        average_fee,
        median_fee,
        average_fee_rate,
    ))
}