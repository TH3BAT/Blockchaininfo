// rpc/initial_mempool_distro.rs

use reqwest::Client;
	use reqwest::header::CONTENT_TYPE;
	use serde_json::json;
	use crate::models::errors::MyError;
	use crate::config::RpcConfig;
	use crate::models::mempool_info::{MempoolEntryJsonWrap, MempoolEntry};
	use tokio::sync::Mutex;
	use std::sync::Arc;
	use std::collections::{HashMap, HashSet};
	use crate::rpc::mempool::MEMPOOL_CACHE; 
	use crate::utils::MEMPOOL_DISTRIBUTION_CACHE;
    	
	const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC
	
	pub static DUST_FREE_TX_CACHE: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, MempoolEntry>>>> = 
	    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
	
	pub static DUST_CACHE: once_cell::sync::Lazy<Arc<Mutex<HashSet<String>>>> = 
	once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));
	
	
	pub async fn initial_mempool_load(config: &RpcConfig) -> Result<(), MyError> {
	    let client = Client::new();
	    
        // Step 1: Fetch all transaction IDs from the mempool
        let all_tx_ids = MEMPOOL_CACHE.read().unwrap().clone();

        // Step 2: Prepare Batch RPC Requests
        let batch_requests: Vec<serde_json::Value> = all_tx_ids.iter()
            .map(|tx_id| {
                json!({
                    "jsonrpc": "1.0",
                    "id": tx_id,
                    "method": "getmempoolentry",
                    "params": [tx_id]
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
        let mut cache = DUST_FREE_TX_CACHE.lock().await;
        let mut dust_cache = DUST_CACHE.lock().await;

        for entry_wrap in batch_response {
            let mempool_entry = entry_wrap.result;

            if mempool_entry.fees.base < DUST_THRESHOLD {
                dust_cache.insert(mempool_entry.wtxid.clone());
            } else {
                cache.insert(mempool_entry.wtxid.clone(), mempool_entry);
            }
        }
         
    // Step 3: Calculate Metrics
    let mut dist = MEMPOOL_DISTRIBUTION_CACHE.write().await;
    dist.update_metrics(&cache);

    Ok(())
}