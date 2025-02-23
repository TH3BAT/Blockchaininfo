
// rpc/mempool_distro.rs

use reqwest::Client;
	use reqwest::header::CONTENT_TYPE;
	use serde_json::json;
	use crate::models::errors::MyError;
	use crate::config::RpcConfig;
	use crate::models::mempool_info::MempoolEntryJsonWrap;
	use tokio::sync::Mutex;
	use std::sync::Arc;
	use rand::rngs::StdRng;
	use rand::SeedableRng; 
    use rand::prelude::SliceRandom;
	use crate::utils::{log_error, LOGGED_TXS};
	use crate::rpc::mempool::MEMPOOL_CACHE; 
	use crate::utils::{BLOCKCHAIN_INFO_CACHE, MEMPOOL_DISTRIBUTION_CACHE};
    use tokio::time::sleep;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration; 
    use crate::rpc::initial_mempool_distro::{DUST_CACHE, DUST_FREE_TX_CACHE};
	
	const DUST_THRESHOLD: f64 = 0.00000546; // 546 sats in BTC
	const MAX_CACHE_SIZE: usize = 100_000; // Rolling cache size
	const MAX_DUST_CACHE_SIZE: usize = 150_000; // Rolling cache size
	
   	static LAST_BLOCK_NUMBER: once_cell::sync::Lazy<Mutex<u64>> = 
	once_cell::sync::Lazy::new(|| Mutex::new(0));
	
	
	pub async fn fetch_mempool_distribution(
	    config: &RpcConfig,
        initial_load_complete: Arc<AtomicBool>,
	) -> Result<(), MyError> {
        // Wait for initial load to complete
        while !initial_load_complete.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(10)).await;
        }

	    let client = Client::new();
	    // Lock and read the blockchain info from the cache
	    let blockchain_info = BLOCKCHAIN_INFO_CACHE.read().await;
        
        let cache = &DUST_FREE_TX_CACHE;
	    let dust_cache = &DUST_CACHE;

        let new_tx_ids: Vec<String> = MEMPOOL_CACHE.iter()
            .filter(|txid| !cache.contains_key(txid.as_str()) && !dust_cache.contains(txid.as_str()))
            .map(|txid| txid.clone())
            .collect(); 
        
        // Lock block number tracking
        let mut last_block = LAST_BLOCK_NUMBER.lock().await;
        
        // Step 0: Remove Expired TXs if Block Changed
        if *last_block != blockchain_info.blocks {
            *last_block = blockchain_info.blocks; // Update last seen block number
        
            // Keep only TXs that still exist in mempool
            dust_cache.retain(|txid| MEMPOOL_CACHE.contains(txid));
            cache.retain(|txid, _| MEMPOOL_CACHE.contains(txid));
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
                        cache.remove(tx_id);
                        let logged_txs_read = LOGGED_TXS.read().await;
                        if !logged_txs_read.contains(tx_id) {
                            log_error(&format!(
                            "JSON parse error for TX {}: {:?}",
                            tx_id, e
                            ));
                            drop(logged_txs_read);
                            let mut logged_txs_write = LOGGED_TXS.write().await;
                            logged_txs_write.insert(tx_id.to_string()); // Mark as logged
                        }
                        return Err(MyError::JsonParsingError(tx_id.clone(), e.to_string()));  // Return CustomError
                    }
                },
                Err(e) => {
                    cache.remove(tx_id);
                    let logged_txs_read = LOGGED_TXS.read().await;
                    if !logged_txs_read.contains(tx_id) {
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
            };
    
        // Now `mempool_entry` contains a valid response or we’ve logged the failure.
            let mempool_entry = response;
    
            if mempool_entry.fees.base < DUST_THRESHOLD {
                if dust_cache.len() == MAX_DUST_CACHE_SIZE {
                    // Collect keys into a Vec
                    let mut keys: Vec<_> = dust_cache.iter().map(|key| key.clone()).collect();
            
                    // Shuffle the keys using a seeded RNG (consistent randomness)
                    let mut rng = StdRng::seed_from_u64(42);
                    keys.shuffle(&mut rng);
            
                    // Remove the first key after shuffle
                    if let Some(random_key) = keys.first() {
                        dust_cache.remove(random_key);
                    }
                }
                dust_cache.insert(tx_id.clone()); // Store it for future lookups
                continue; // Skip processing
            }
            
            // Ensure we don’t exceed the max cache size before inserting.
            if cache.len() == MAX_CACHE_SIZE {
                // Collect keys into a Vec
                let mut keys: Vec<_> = cache.iter().map(|entry| entry.key().clone()).collect();
            
                // Shuffle the keys using a seeded RNG (consistent randomness)
                let mut rng = StdRng::seed_from_u64(42);
                keys.shuffle(&mut rng);
            
                // Remove the first key after shuffle
                if let Some(random_key) = keys.first() {
                    cache.remove(random_key);
                }
            }
            // Now insert the new entry after making space
            cache.insert(tx_id.clone(), mempool_entry);
    
        }
         
    // Step 3: Calculate Metrics
    let mut dist = MEMPOOL_DISTRIBUTION_CACHE.write().await;
    dist.update_metrics(&cache);

    Ok(())
}