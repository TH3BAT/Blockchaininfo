//! RPC handlers for block-related Bitcoin Core methods.
//!
//! This module is responsible for:
//! - Fetching block hashes by height (`getblockhash`)
//! - Fetching block data with verbose=1 (header + txids)
//! - Fetching full block data with verbose=2 (header + full tx objects)
//! - Determining the miner via coinbase parsing
//! - Updating `BLOCK_HISTORY` for the Hash Rate Distribution chart
//!
//! This file represents one of the most critical paths in the dashboard,
//! powering epoch calculations, 24h difficulty drift, miner extraction,
//! and the UI’s block/txid displays.

use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::rpc::client::build_rpc_client;

use crate::models::block_info::{
    BlockHash,
    BlockInfo,
    BlockInfoJsonWrap,
    MinersData,
    BlockInfoFull,
    BlockInfoFullJsonWrap,
};

use crate::utils::BLOCK_HISTORY;
use crate::consensus::satoshi_math::*;

/// Fetch block information at a specific height using `getblock` with verbose=1.
///
/// ### Purpose
/// This RPC is used in two contexts:
/// - **Epoch Start Block (mode = 1)**  
///   Determines the starting block of the difficulty epoch by rounding down
///   to the nearest 2016-block boundary.
/// - **Past 24 Hours Block (mode = 2)**  
///   Used for 24h difficulty drift calculations by moving back ~144 blocks.
///
/// Returns:
/// - `BlockInfo` (header + vector of txids)
///
/// Errors:
/// - Timeout
/// - Reqwest network error
/// - JSON parsing error
/// - Custom error for invalid mode
pub async fn fetch_block_data_by_height(
    config: &RpcConfig,
    blocks: u64,
    mode: u16, // 1 = Epoch Start Block, 2 = 24 Hours Ago Block
) -> Result<BlockInfo, MyError> {

    // Determine target block height
    let block_height = match mode {
        1 => {
            // Find first block in the current difficulty epoch
            ((blocks - 1) / DIFFICULTY_ADJUSTMENT_INTERVAL) * DIFFICULTY_ADJUSTMENT_INTERVAL
        }
        2 => {
            // Approx. block height 24 hours ago (~144 blocks)
            blocks.saturating_sub(143)
        }
        _ => {
            return Err(MyError::CustomError(
                "Invalid mode. Use 1 for Epoch Start Block or 2 for 24H Block.".to_string(),
            ));
        }
    };

    // RPC client with timeouts tailored for TUI responsiveness
    let client = build_rpc_client()?;

    // ──────────────────────────────
    // Step 1: getblockhash
    // ──────────────────────────────
    let getblockhash_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblockhash",
        "params": [block_height]
    });

    let block_hash_response: BlockHash = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblockhash_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblockhash'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockHash>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblockhash.".to_string())
        })?;

    let blockhash = block_hash_response.result;

    // ──────────────────────────────
    // Step 2: getblock (verbose = 1)
    // ──────────────────────────────
    let getblock_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblock",
        "params": [blockhash] // default verbose=1
    });

    let block_response: BlockInfoJsonWrap = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblock_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblock'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockInfoJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblock.".to_string())
        })?;

    Ok(block_response.result)
}

/// Fetch full block data with verbose=2.
///
/// ### Purpose
/// This internal helper retrieves:
/// - Complete transaction objects
/// - Useful for miner extraction through coinbase parsing
///
/// Not exposed publicly because full block data is used only internally
/// for miner identification.
async fn fetch_full_block_data_by_height(
    config: &RpcConfig,
    blocks: &u64,
) -> Result<BlockInfoFull, MyError> {

    let client = build_rpc_client()?;

    // ──────────────────────────────
    // Step 1: getblockhash
    // ──────────────────────────────
    let getblockhash_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblockhash",
        "params": [*blocks]
    });

    let block_hash_response: BlockHash = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblockhash_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblockhash'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockHash>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblockhash.".to_string())
        })?;

    let blockhash = block_hash_response.result;

    // ──────────────────────────────
    // Step 2: getblock (verbose = 2)
    // ──────────────────────────────
    let getblock_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblock",
        "params": [blockhash, 2]  // Return full tx objects
    });

    let block_response: BlockInfoFullJsonWrap = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblock_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblock'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockInfoFullJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblock.".to_string())
        })?;

    Ok(block_response.result)
}

/// Parse the miner for the current block and append them to `BLOCK_HISTORY`.
///
/// ### Workflow:
/// 1. Fetch full block data using verbose=2  
/// 2. Extract the coinbase transaction  
/// 3. Parse wallet addresses from the coinbase output  
/// 4. Match the address to known miners from `miners.json`  
/// 5. Append result to rolling `BlockHistory` (used for hash rate distribution chart)
///
/// If no miner match is found, `"Unknown"` is used.
pub async fn fetch_miner(
    config: &RpcConfig,
    miners_data: &MinersData,
    current_block: &u64,
) -> Result<(), MyError> {

    // Always fetch with verbose=2 for miner identification
    let block = fetch_full_block_data_by_height(config, &current_block).await?;

    // Coinbase is always tx[0]
    let coinbase_tx = &block.tx[0];
    let coinbase_tx_addresses = coinbase_tx.extract_wallet_addresses();

    // Attempt miner lookup
    let miner = find_miner_by_wallet(coinbase_tx_addresses, miners_data).await
        .unwrap_or("Unknown".to_string());

    // Append into rolling history
    let block_history = BLOCK_HISTORY.write().await;
    block_history.add_block(Some(miner.into()));

    Ok(())
}

/// Matches extracted coinbase addresses to known miners from miners.json.
///
/// Returns:
/// - `Some(miner_name)` if a match is found  
/// - `None` otherwise  
///
/// Miner identification relies entirely on wallet labels provided in miners.json.
async fn find_miner_by_wallet(addresses: Vec<String>, miners_data: &MinersData) -> Option<String> {
    for address in addresses {
        if let Some(miner) = miners_data.miners.iter()
            .find(|miner| miner.wallet == address)
            .map(|miner| miner.name.clone())
        {
            return Some(miner);
        }
    }
    None
}
