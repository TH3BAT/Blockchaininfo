
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
use crate::utils::{BLOCKCHAIN_INFO_CACHE, MEMPOOL_DISTRIBUTION_CACHE};
use dashmap::DashSet;
use once_cell::sync::Lazy;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;             // For spawning tasks


const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC
const MAX_CACHE_SIZE: usize = 100_000; // Rolling cache size
const MAX_DUST_CACHE_SIZE: usize = 150_000; // Rolling cache size

static LAST_BLOCK_NUMBER: Lazy<DashSet<u64>> = Lazy::new(|| DashSet::new());

static DUST_FREE_TX_CACHE: Lazy<Arc<DashMap<String, MempoolEntry>>> =
    Lazy::new(|| Arc::new(DashMap::with_capacity(100_000)));

static DUST_CACHE: Lazy<Arc<DashSet<String>>> =
    Lazy::new(|| Arc::new(DashSet::with_capacity(150_000)));


pub async fn fetch_mempool_distribution(config: &RpcConfig) -> Result<(), MyError> {
    let cache = &DUST_FREE_TX_CACHE;
    let dust_cache = &DUST_CACHE;
    let client = Client::new();
    let blockchain_info = BLOCKCHAIN_INFO_CACHE.read().await;

    // Step 0: Remove Expired TXs if Block Changed
    if !LAST_BLOCK_NUMBER.contains(&blockchain_info.blocks) {
        LAST_BLOCK_NUMBER.clear(); // Clear the set
        LAST_BLOCK_NUMBER.insert(blockchain_info.blocks); // Insert the new block number
    }

    // Collect new transaction IDs that are not in either cache
    let new_tx_ids: Vec<String> = MEMPOOL_CACHE.iter()
        .filter(|txid| !cache.contains_key(txid.as_str()) && !dust_cache.contains(txid.as_str()))
        .map(|txid| txid.clone())
        .collect();

    // Step 1: Process transactions one at a time with controlled concurrency
    let semaphore = Arc::new(Semaphore::new(10)); // Limit to 10 concurrent tasks
    let mut tasks = Vec::new();

    for tx_id in new_tx_ids {
        let permit = semaphore.clone().acquire_owned().await?;
        let client = client.clone();
        let config = config.clone();
        // let cache = cache;
        // let dust_cache = dust_cache.clone();

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
                .map_err(|e| MyError::RpcRequestError(tx_id.clone(), e.to_string()))?
                .json::<MempoolEntryJsonWrap>()
                .await
                .map_err(|e| MyError::JsonParsingError(tx_id.clone(), e.to_string()))
                .map(|wrap| (tx_id.clone(), wrap.result));

            match result {
                Ok((tx_id, mempool_entry)) => {
                    if mempool_entry.fees.base < DUST_THRESHOLD {
                        // Insert into DUST_CACHE, evict if necessary
                        if dust_cache.len() == MAX_DUST_CACHE_SIZE {
                            let mut keys: Vec<_> = dust_cache.iter().map(|key| key.clone()).collect();
                            let mut rng = StdRng::seed_from_u64(42);
                            keys.shuffle(&mut rng);
                            if let Some(random_key) = keys.first() {
                                dust_cache.remove(random_key);
                            }
                        }
                        dust_cache.insert(tx_id.clone());
                    } else {
                        // Insert into DUST_FREE_TX_CACHE, evict if necessary
                        if cache.len() == MAX_CACHE_SIZE {
                            let mut keys: Vec<_> = cache.iter().map(|entry| entry.key().clone()).collect();
                            let mut rng = StdRng::seed_from_u64(42);
                            keys.shuffle(&mut rng);
                            if let Some(random_key) = keys.first() {
                                cache.remove(random_key);
                            }
                        }
                        cache.insert(tx_id.clone(), mempool_entry);
                    }
                    Ok(()) // Return Ok(()) to satisfy the Result type
                }
                Err(e) => {
                    cache.remove(&tx_id);
                    let logged_txs_read = LOGGED_TXS.read().await;
                    if !logged_txs_read.contains(&tx_id) {
                        // Log the error using your custom log_error function
                        log_error(&format!(
                            "RPC request failed for TX {}: {:?}",
                            tx_id, e
                        ));
                
                        drop(logged_txs_read);
                        let mut logged_txs_write = LOGGED_TXS.write().await;
                        logged_txs_write.insert(tx_id.to_string()); // Mark as logged
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
                    // Log the error using your custom log_error function
                    log_error(&format!("Task failed: {:?}", e));
                }
            }
            Err(e) => {
                // Log the join error using your custom log_error function
                log_error(&format!("Task join failed: {:?}", e));
            }
        }
    }

    // Step 2: Calculate Metrics
    let mut dist = MEMPOOL_DISTRIBUTION_CACHE.write().await;
    dist.update_metrics(&cache);

    Ok(())
}