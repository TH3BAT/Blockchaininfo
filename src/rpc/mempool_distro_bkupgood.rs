
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
// use crate::utils::{log_error, LOGGED_TXS};
use crate::rpc::mempool::MEMPOOL_CACHE; 
use crate::utils::{BLOCKCHAIN_INFO_CACHE, MEMPOOL_DISTRIBUTION_CACHE};
// use crate::rpc::initial_mempool_distro::{DUST_CACHE, DUST_FREE_TX_CACHE};
use dashmap::DashSet;
use once_cell::sync::Lazy;
use dashmap::DashMap;
use std::sync::Arc;
use futures::future::join_all;
use tokio::task;

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

    if DUST_CACHE.is_empty() {
        // Step 1: Split transaction IDs into chunks for batch processing
        let chunk_size = 1_000; // Adjust based on your needs
        let chunks: Vec<Vec<String>> = new_tx_ids.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();

        // Step 2: Fetch and process chunks in parallel
        let fetch_tasks: Vec<_> = chunks.into_iter()
            .map(|chunk| {
                let client = client.clone();
                let config = config.clone();
                task::spawn(async move { fetch_mempool_entries(&client, &config, chunk).await })
            })
            .collect();

        let results = join_all(fetch_tasks).await;

        for result in results {
            let entries = result??; // Handle task and result errors
            process_mempool_entries(entries, cache, dust_cache);
        }
    } else {
        // Process transactions in smaller batches
        let batch_size = 100; // Adjust based on your needs
        for chunk in new_tx_ids.chunks(batch_size) {
            let fetch_tasks: Vec<_> = chunk.iter()
                .map(|tx_id| {
                    let client = client.clone();
                    let config = config.clone();
                    let tx_id = tx_id.clone();
                    task::spawn(async move {
                        let json_rpc_request = json!({
                            "jsonrpc": "1.0",
                            "id": "1",
                            "method": "getmempoolentry",
                            "params": [tx_id]
                        });

                        client.post(&config.address)
                            .basic_auth(&config.username, Some(&config.password))
                            .header(CONTENT_TYPE, "application/json")
                            .json(&json_rpc_request)
                            .send()
                            .await
                            .map_err(|e| MyError::RpcRequestError(tx_id.clone(), e.to_string()))?
                            .json::<MempoolEntryJsonWrap>()
                            .await
                            .map_err(|e| MyError::JsonParsingError(tx_id.clone(), e.to_string()))
                            .map(|wrap| (tx_id, wrap.result))
                    })
                })
                .collect();

            let results = join_all(fetch_tasks).await;

            for result in results {
                let (tx_id, mempool_entry) = result??;

                // Step 3: Manage DUST_CACHE and DUST_FREE_TX_CACHE
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
            }
        }
    }

    // Step 4: Calculate Metrics
    let mut dist = MEMPOOL_DISTRIBUTION_CACHE.write().await;
    dist.update_metrics(&cache);

    Ok(())
}


async fn fetch_mempool_entries(
    client: &Client,
    config: &RpcConfig,
    tx_ids: Vec<String>,
) -> Result<Vec<MempoolEntry>, MyError> {
    let batch_requests: Vec<serde_json::Value> = tx_ids
        .iter()
        .map(|tx_id| {
            json!({
                "jsonrpc": "1.0",
                "id": tx_id,
                "method": "getmempoolentry",
                "params": [tx_id]
            })
        })
        .collect();

    let batch_response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&batch_requests)
        .send()
        .await?
        .json::<Vec<MempoolEntryJsonWrap>>()
        .await?;

    Ok(batch_response.into_iter().map(|entry| entry.result).collect())
}


fn process_mempool_entries(entries: Vec<MempoolEntry>, cache: &DashMap<String, MempoolEntry>, dust_cache: &DashSet<String>) {
    for entry in entries {
        if entry.fees.base < DUST_THRESHOLD {
            dust_cache.insert(entry.wtxid.clone());
        } else {
            cache.insert(entry.wtxid.clone(), entry);
        }
    }
}