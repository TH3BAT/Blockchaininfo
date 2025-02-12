
// rpc/mempool_distro.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::mempool_info::{MempoolEntryJsonWrap, MempoolEntry};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use crate::utils::log_error;
use crate::rpc::mempool::MEMPOOL_CACHE; 


const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC
const MAX_CACHE_SIZE: usize = 10_000; // Rolling cache size

static DUST_FREE_TX_CACHE: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, MempoolEntry>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

static DUST_CACHE: once_cell::sync::Lazy<Arc<Mutex<HashSet<String>>>> = 
once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));


static LAST_BLOCK_NUMBER: once_cell::sync::Lazy<Mutex<u64>> = 
once_cell::sync::Lazy::new(|| Mutex::new(0));

pub async fn fetch_mempool_distribution(
    config: &RpcConfig,
    active_block_number: u64, 
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
    let mut dust_cache = DUST_CACHE.lock().await;

    let all_tx_ids = {
        let read_guard = MEMPOOL_CACHE.read().unwrap(); // Lock read access
        read_guard.clone() // Clone the HashSet before async move
    }; // Drops the read lock immediately
    
    let new_tx_ids: Vec<String> = all_tx_ids.iter()
        .filter(|txid| !cache.contains_key(txid.as_str()) && !dust_cache.contains(txid.as_str()))
        .cloned()
        .collect();    

    // Lock block number tracking
    let mut last_block = LAST_BLOCK_NUMBER.lock().await;
    
    // Step 0: Remove Expired TXs if Block Changed
    if *last_block != active_block_number {
        *last_block = active_block_number; // Update last seen block number
    
        // Remove TXs that no longer exist in mempool
        cache.retain(|txid, _| all_tx_ids.contains(txid));
        dust_cache.clear(); // Clean out old dust transactions
    }
    
    // Step 1: Update Cache (Only Add Dust-Free TXs)
    for tx_id in new_tx_ids.iter() {
        let json_rpc_request = json!({
            "jsonrpc": "1.0",
            "id": "1",
            "method": "getmempoolentry",
            "params": [tx_id]
        });

        let response = match client
            .post(&config.address)
            .basic_auth(&config.username, Some(&config.password))
            .header(CONTENT_TYPE, "application/json")
            .json(&json_rpc_request)
            .send()
            .await
        {
            Ok(resp) => match resp.json::<MempoolEntryJsonWrap>().await {
                Ok(parsed_response) => parsed_response.result, // Success, proceed
                Err(e) => {
                    log_error(&format!(
                        "JSON parse error for TX {}: {:?}",
                        tx_id, e
                    ));
                    return Err(MyError::JsonParsingError(tx_id.clone(), e.to_string()));  // Return CustomError
                }
            },
            Err(e) => {
                log_error(&format!(
                    "RPC request failed for TX {}: {:?}",
                    tx_id, e
                ));
                return Err(MyError::RpcRequestError(tx_id.clone(), e.to_string())); // Return CustomError
            }
        };

        // Now `mempool_entry` contains a valid response or we’ve logged the failure.
        let mempool_entry = response;

        if mempool_entry.fees.base < DUST_THRESHOLD {
            dust_cache.insert(tx_id.clone()); // Store it for future lookups
            continue; // Skip processing
        }
        

        cache.insert(tx_id.clone(), mempool_entry);

        if cache.len() > MAX_CACHE_SIZE {
            let oldest_key = cache.keys().next().cloned();
            if let Some(oldest_key) = oldest_key {
                cache.remove(&oldest_key);
            }
        }
    }

    // Step 2: Process Cached Transactions & Calculate Distribution
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