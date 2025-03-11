
// rpc/mempool_distro.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::mempool_info::{MempoolEntryJsonWrap, MempoolEntry};
use rand::rngs::StdRng;
use rand::SeedableRng; 
use rand::prelude::SliceRandom;
use crate::utils::{log_error, LOGGED_TXS};
use crate::rpc::mempool::MEMPOOL_CACHE; 
use crate::utils::MEMPOOL_DISTRIBUTION_CACHE;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;             // For spawning tasks
use std::time::Duration;


const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC
const MAX_CACHE_SIZE: usize = 100_000; // Rolling cache size
const MAX_DUST_CACHE_SIZE: usize = 150_000; // Rolling cache size

static DUST_FREE_TX_CACHE: Lazy<Arc<DashMap<String, MempoolEntry>>> =
    Lazy::new(|| Arc::new(DashMap::with_capacity(100_000)));

static DUST_CACHE: Lazy<Arc<DashSet<String>>> =
    Lazy::new(|| Arc::new(DashSet::with_capacity(150_000)));


pub async fn fetch_mempool_distribution(config: &RpcConfig) -> Result<(), MyError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // Remove expired TXs from DUST_CACHE
    DUST_CACHE.retain(|tx_id| MEMPOOL_CACHE.contains(tx_id));

    // Remove expired TXs from DUST_FREE_TX_CACHE
    DUST_FREE_TX_CACHE.retain(|tx_id, _| MEMPOOL_CACHE.contains(tx_id));

    // Collect new transaction IDs that are not in either cache
    let new_tx_ids: Vec<String> = MEMPOOL_CACHE.iter()
        .filter(|txid| !DUST_FREE_TX_CACHE.contains_key(txid.as_str()) && !DUST_CACHE.contains(txid.as_str()))
        .map(|txid| txid.clone())
        .collect();

    // Step 1: Process transactions one at a time with controlled concurrency
    let semaphore = Arc::new(Semaphore::new(10)); // Limit to 10 concurrent tasks
    let mut tasks = Vec::new();

    for tx_id in new_tx_ids {
        let permit = semaphore.clone().acquire_owned().await?;
        let client = client.clone();
        let config = config.clone();

        tasks.push(task::spawn(async move {
            let _permit = permit; // Hold the permit until the task completes

            let json_rpc_request = json!({
                "jsonrpc": "1.0",
                "id": "1",
                "method": "getmempoolentry",
                "params": [tx_id]
            });

            let result = client.post(&config.address)
                .basic_auth(&config.username, Some(&config.password))
                .header(CONTENT_TYPE, "application/json")
                .json(&json_rpc_request)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        MyError::TimeoutError(format!(
                            "Request to {} timed out for method 'getmempoolentry'",
                            config.address
                        ))
                    } else {
                        MyError::RpcRequestError(tx_id.clone(), e.to_string())
                    }
                })?
                .json::<MempoolEntryJsonWrap>()
                .await
                .map_err(|e| MyError::JsonParsingError(tx_id.clone(), e.to_string()))
                .map(|wrap| (tx_id.clone(), wrap.result));

            match result {
                Ok((tx_id, mempool_entry)) => {
                    if mempool_entry.fees.base < DUST_THRESHOLD {
                        // Insert into DUST_CACHE, evict if necessary
                        if DUST_CACHE.len() == MAX_DUST_CACHE_SIZE {
                            let mut keys: Vec<_> = DUST_CACHE.iter().map(|key| key.clone()).collect();
                            let mut rng = StdRng::seed_from_u64(42);
                            keys.shuffle(&mut rng);
                            if let Some(random_key) = keys.first() {
                                DUST_CACHE.remove(random_key);
                            }
                        }
                        DUST_CACHE.insert(tx_id.clone());
                    } else {
                        // Insert into DUST_FREE_TX_CACHE, evict if necessary
                        if DUST_FREE_TX_CACHE.len() == MAX_CACHE_SIZE {
                            let mut keys: Vec<_> = DUST_FREE_TX_CACHE.iter().map(|entry| entry.key().clone()).collect();
                            let mut rng = StdRng::seed_from_u64(42);
                            keys.shuffle(&mut rng);
                            if let Some(random_key) = keys.first() {
                                DUST_FREE_TX_CACHE.remove(random_key);
                            }
                        }
                        DUST_FREE_TX_CACHE.insert(tx_id.clone(), mempool_entry);
                    }
                    Ok(()) // Return Ok(()) to satisfy the Result type
                }
                Err(e) => {
                    let logged_txs_read = LOGGED_TXS.read().await;
                    if !logged_txs_read.0.contains(&tx_id) {
                        // Log the error
                        if let Err(log_err) = log_error(&format!(
                            "getmempoolentry failed: {}",
                            e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                
                        drop(logged_txs_read);
                        let mut logged_txs_write = LOGGED_TXS.write().await;
                        let (set, queue) = &mut *logged_txs_write;
                        if set.len() >= 500 {
                            if let Some(oldest_tx) = queue.pop_front() {
                                set.remove(&oldest_tx);
                            }
                        }
                         // Wrap `tx_id` in an `Arc` for shared ownership
                        let tx_id_arc = Arc::new(tx_id.clone());
                        set.insert(tx_id_arc.clone());
                        queue.push_back(tx_id_arc);
                    }
                    return Err(MyError::RpcRequestError(tx_id.clone(), e.to_string())); // Return CustomError
                }
            }
        }));
    }

    // Wait for all tasks to complete
    for task in tasks {
        match task.await {
            Ok(result) => {
                if let Err(e) = result {
                    // Convert the error to a string
                    let error_string = format!("{:?}", e);
    
                    // Extract the Tx ID from the error string
                    if let Some(tx_id) = extract_tx_id_from_error_string(&error_string) {
                        // Check if the Tx ID is already logged
                        let logged_txs_read = LOGGED_TXS.read().await;
                        if !logged_txs_read.0.contains(&tx_id) {
                            if let Err(log_err) = log_error(&format!(
                                "Task failed: {}", error_string
                            )) {
                                eprintln!("Task reported Tx failure: {}", log_err);
                            }
    
                            // Mark the Tx ID as logged
                            drop(logged_txs_read);
                            let mut logged_txs_write = LOGGED_TXS.write().await;
                            let (set, queue) = &mut *logged_txs_write;
                            if set.len() >= 500 {
                                if let Some(oldest_tx) = queue.pop_front() {
                                    set.remove(&oldest_tx);
                                }
                            }
                             // Wrap `tx_id` in an `Arc` for shared ownership
                            let tx_id_arc = Arc::new(tx_id.clone());
                            set.insert(tx_id_arc.clone());
                            queue.push_back(tx_id_arc);
                        }
                    } else {
                        // If no Tx ID is found, log the error as-is
                        if let Err(log_err) = log_error(&format!(
                                "Task failed: {}", error_string
                            )) {
                                eprintln!("Task reported Tx failure: {}", log_err);
                            }
                    }
                }
            }
            Err(e) => {
                // Log the join error using your custom log_error function
                if let Err(log_err) = log_error(&format!(
                    "Task joined failed: {}", e
                )) {
                    eprintln!("Task join failure: {}", log_err);
                }
            }
        }
    }

    // Step 2: Calculate Metrics
    let mut dist = MEMPOOL_DISTRIBUTION_CACHE.write().await;
    dist.update_metrics(&DUST_FREE_TX_CACHE);

    Ok(())
}

/// Helper function uses Regex to search for TxID 64-character hex string.
fn extract_tx_id_from_error_string(error_string: &str) -> Option<String> {
    // Look for the Tx ID pattern in the error string
    let tx_id_pattern = r#"RpcRequestError\("([a-f0-9]{64})"#;
    let re = regex::Regex::new(tx_id_pattern).unwrap();

    // Try to capture the Tx ID
    if let Some(captures) = re.captures(error_string) {
        if let Some(tx_id) = captures.get(1) {
            return Some(tx_id.as_str().to_string());
        }
    }

    // If no Tx ID is found, return None
    None
}