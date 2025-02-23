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

const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC

pub static DUST_FREE_TX_CACHE: Lazy<Arc<DashMap<String, MempoolEntry>>> =
    Lazy::new(|| Arc::new(DashMap::with_capacity(100_000)));

pub static DUST_CACHE: Lazy<Arc<DashSet<String>>> =
    Lazy::new(|| Arc::new(DashSet::with_capacity(150_000)));


pub async fn initial_mempool_load(config: &RpcConfig) -> Result<(), MyError> {
    let client = Client::new();

    // Step 2: Prepare Batch RPC Requests
    let batch_requests: Vec<serde_json::Value> = MEMPOOL_CACHE.iter()
    .map(|tx_id| {
        json!({
            "jsonrpc": "1.0",
            "id": *tx_id, // Dereference tx_id to get &String
            "method": "getmempoolentry",
            "params": [*tx_id] // Dereference tx_id here as well
        })
    })
    .collect();

    // Step 3: Send Batch Request
    let batch_response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&batch_requests)
        .send()
        .await?
        .json::<Vec<MempoolEntryJsonWrap>>()
        .await?;

    // Step 4: Process Batch Response and Populate Caches
    let cache = &DUST_FREE_TX_CACHE;
    let dust_cache = &DUST_CACHE;

    for entry_wrap in batch_response {
        let mempool_entry = entry_wrap.result;

        if mempool_entry.fees.base < DUST_THRESHOLD {
            dust_cache.insert(mempool_entry.wtxid.clone());
        } else {
            cache.insert(mempool_entry.wtxid.clone(), mempool_entry);
        }
    }

let mut dist = MempoolDistribution::default();
dist.update_metrics(&cache); // Update the metrics

// Replace the cache with the new distribution
let mut cache = MEMPOOL_DISTRIBUTION_CACHE.write().await;
*cache = dist;

Ok(())
}