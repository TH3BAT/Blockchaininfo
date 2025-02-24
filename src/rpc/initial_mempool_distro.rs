// rpc/initial_mempool_distro.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::mempool_info::{MempoolEntryJsonWrap, MempoolEntry, MempoolDistribution};
use crate::rpc::mempool::MEMPOOL_CACHE; 
use crate::utils::MEMPOOL_DISTRIBUTION_CACHE;
use dashmap::{DashMap, DashSet};
use once_cell::sync::Lazy;
use std::sync::Arc;
use futures::future::join_all;
use tokio::task;

const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC

pub static DUST_FREE_TX_CACHE: Lazy<Arc<DashMap<String, MempoolEntry>>> =
    Lazy::new(|| Arc::new(DashMap::with_capacity(100_000)));

pub static DUST_CACHE: Lazy<Arc<DashSet<String>>> =
    Lazy::new(|| Arc::new(DashSet::with_capacity(150_000)));


    pub async fn initial_mempool_load(config: &RpcConfig) -> Result<(), MyError> {
        let client = Client::new();
    
        // Step 1: Split transaction IDs into chunks
        let chunk_size = 10_000; // Adjust based on your needs
        let tx_ids: Vec<String> = MEMPOOL_CACHE.iter().map(|tx_id| tx_id.clone()).collect();
        let chunks: Vec<Vec<String>> = tx_ids.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();
    
        // Step 2: Fetch and process chunks in parallel
        let fetch_tasks: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                let client = client.clone();
                let config = config.clone();
                task::spawn(async move { fetch_mempool_entries(&client, &config, chunk).await })
            })
            .collect();
    
        let results = join_all(fetch_tasks).await;
    
        // Step 3: Process fetched entries and update caches
        let cache = &DUST_FREE_TX_CACHE;
        let dust_cache = &DUST_CACHE;
    
        for result in results {
            let entries = result??; // Handle task and result errors
            process_mempool_entries(entries, cache, dust_cache);
        }
    
        // Step 4: Update metrics
        let mut dist = MempoolDistribution::default();
        dist.update_metrics(cache);
    
        // Step 5: Update the distribution cache
        let mut cache = MEMPOOL_DISTRIBUTION_CACHE.write().await;
        *cache = dist;
    
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