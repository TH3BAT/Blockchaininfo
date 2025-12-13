//! Handles the mempool distribution pipeline.
//!
//! This module is responsible for:
//! - Fetching individual mempool entries via `getmempoolentry`
//! - Maintaining a rolling TX cache (TX_CACHE)
//! - Respecting the "Dust-Free" toggle by filtering low-fee transactions
//! - Limiting RPC concurrency to avoid node overload
//! - Evicting cached TXs using deterministic random selection
//! - Computing aggregated mempool distribution metrics
//!
//! This module powers the **Mempool Distribution Chart**, one of the most
//! important real-time analytical tools in the dashboard.
//!
//! ### High-Level Flow
//! 1. Identify new TXIDs from the global mempool cache (`MEMPOOL_CACHE`)
//! 2. Spawn limited-concurrency tasks to fetch missing entries
//! 3. Insert or filter entries depending on `dust_free` mode
//! 4. Maintain a rolling TX cache with a fixed max size
//! 5. Update global `MempoolDistribution` metrics

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::mempool_info::{MempoolEntryJsonWrap, MempoolEntry};

use rand::rngs::StdRng;
use rand::SeedableRng; 
use rand::prelude::SliceRandom;

use crate::utils::log_error;
use crate::rpc::mempool::MEMPOOL_CACHE; 
use crate::utils::MEMPOOL_DISTRIBUTION_CACHE;

use once_cell::sync::Lazy;
use dashmap::DashMap;

use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;
use std::time::Duration;
use hex::ToHex;

/// The dust threshold (546 sats), expressed in BTC.
/// Any TX with fees below this threshold is considered "dust" when filtering.
const DUST_THRESHOLD: f64 = 0.00000546;

/// Maximum number of mempool entries to retain in our rolling TX cache.
///
/// This cap protects memory usage and ensures predictable UI performance.
const MAX_TX_CACHE_SIZE: usize = 250_000;

/// Rolling mempool entry cache.
///
/// Stores complete `MempoolEntry` objects keyed by TXID.
///
/// - Backed by `DashMap` for thread-safe concurrent read/write
/// - Initialized lazily
/// - Used by the "Dust-Free" toggle and distribution metrics
static TX_CACHE: Lazy<Arc<DashMap<[u8; 32], MempoolEntry>>> =
    Lazy::new(|| Arc::new(DashMap::with_capacity(250_000)));

/// Main entry point for computing mempool distribution.
///
/// This function performs **three responsibilities**:
///
/// ### 1. Maintain TX cache consistency
/// - Clears or prunes the cache depending on `dust_free` mode  
/// - Removes expired TXs (those no longer in Bitcoin Core's mempool)
///
/// ### 2. Fetch missing mempool entries via RPC
/// - Identifies TXIDs lacking entries in TX_CACHE  
/// - Spawns a bounded number of concurrent RPC calls (`Semaphore` limited to 10)  
/// - Ensures we do not overwhelm the node with many parallel RPCs  
///
/// ### 3. Update distribution metrics
/// - Aggregates all cached mempool entries  
/// - Updates the global `MempoolDistribution` object used by the dashboard
///
/// ### RPC Notes
/// - Uses `getmempoolentry` for each TXID  
/// - Applies deterministic random eviction when cache reaches MAX_TX_CACHE_SIZE  
///
/// ### Error Behavior
/// Errors for individual transactions do **not** stop the entire distribution process.
/// They are logged or returned silently to avoid disruption to the UI.
pub async fn fetch_mempool_distribution(config: &RpcConfig, dust_free: bool) -> Result<(), MyError> {

    // Build lightweight RPC client
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // ─────────────────────────────────────────────────────────────
    // Handle Dust-Free toggle behavior
    // ─────────────────────────────────────────────────────────────

    if dust_free {
        // Retain only entries still present in mempool
        TX_CACHE.retain(|tx_id, _| MEMPOOL_CACHE.contains(tx_id));
    } else {
        // Reset to full sampling mode
        TX_CACHE.clear();
    }

    // Identify TXIDs that require fetching
    let new_tx_ids: Vec<[u8; 32]> = MEMPOOL_CACHE.iter()
       .filter(|txid| !TX_CACHE.contains_key(&**txid))
        .map(|txid| *txid)
        .collect();


    // ─────────────────────────────────────────────────────────────
    // Step 1: RPC fetch with concurrency control
    // ─────────────────────────────────────────────────────────────

    let semaphore = Arc::new(Semaphore::new(10)); // Limit: 10 concurrent RPCs
    let mut tasks = Vec::new();

    for tx_id_bytes in new_tx_ids {
        let tx_id_hex = tx_id_bytes.encode_hex::<String>();
        let permit = semaphore.clone().acquire_owned().await?;
        let client = client.clone();
        let config = config.clone();

        // Spawn a task for each TXID
        tasks.push(task::spawn(async move {
            let _permit = permit; // Ensure permit is held for task lifetime

            // Build JSON-RPC request
            let json_rpc_request = json!({
                "jsonrpc": "1.0",
                "id": "1",
                "method": "getmempoolentry",
                "params": [tx_id_hex]
            });

            // Execute request and attempt to parse entry
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
                        MyError::RpcRequestError(tx_id_hex.clone(), e.to_string())
                    }
                })?
                .json::<MempoolEntryJsonWrap>()
                .await
                .map_err(|e| MyError::JsonParsingError(tx_id_hex.clone(), e.to_string()))
                .map(|wrap| wrap.result);

            match result {
                Ok(mempool_entry) => {
                    
                    // Evict oldest entry if cache is full
                    if TX_CACHE.len() == MAX_TX_CACHE_SIZE {
                        let mut keys: Vec<_> = TX_CACHE.iter().map(|entry| entry.key().clone()).collect();
                        let mut rng = StdRng::seed_from_u64(42); // deterministic shuffle
                        keys.shuffle(&mut rng);

                        if let Some(random_key) = keys.first() {
                            TX_CACHE.remove(random_key);
                        }
                    }

                    // Dust-Free mode: retain only entries >= dust threshold
                    if dust_free {
                        if mempool_entry.fees.base >= DUST_THRESHOLD {
                            TX_CACHE.insert(tx_id_bytes.clone(), mempool_entry);
                        }

                        // prune any lingering dust
                        TX_CACHE.retain(|_, mempool_entry| mempool_entry.fees.base >= DUST_THRESHOLD);

                    } else {
                        // Full mode: store everything
                        TX_CACHE.insert(tx_id_bytes.clone(), mempool_entry);
                    }

                    Ok(())
                }

                Err(e) => {
                    // Propagate RPC error with transaction context
                    Err(MyError::RpcRequestError(tx_id_hex.clone(), e.to_string()))
                }
            }
        }));
    }

    // ─────────────────────────────────────────────────────────────
    // Await task completion and log any join failures
    // ─────────────────────────────────────────────────────────────
    for task in tasks {
        match task.await {
            Ok(result) => {
                if let Err(_e) = result {
                    // error already captured in result; silently ignore
                }
            }
            Err(e) => {
                // Log join errors (rare)
                if let Err(log_err) = log_error(&format!("Task join failed: {}", e)) {
                    let _ = log_err;
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Step 2: Recompute and store aggregated mempool distribution metrics
    // ─────────────────────────────────────────────────────────────
    let mut dist = MEMPOOL_DISTRIBUTION_CACHE.write().await;
    dist.update_metrics(&TX_CACHE);

    Ok(())
}

