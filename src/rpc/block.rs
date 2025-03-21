
// rpc/block.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::block_info::{BlockHash, BlockInfo, BlockInfoJsonWrap, MinersData, BlockInfoFull, BlockInfoFullJsonWrap};
use crate::utils::{DIFFICULTY_ADJUSTMENT_INTERVAL, BLOCK_HISTORY};
use std::time::Duration;

/// Capture block info with verbose = 1.
/// Returns block information with Vec of TxIDs.  
pub async fn fetch_block_data_by_height(
    config: &RpcConfig,
    blocks: u64,
    mode: u16, // 1 = Epoch Start Block, 2 = 24 Hours Ago Block
) -> Result<BlockInfo, MyError> {
    // Determine which block height to fetch
    let block_height = match mode {
        1 => {
            // Get first block of the current difficulty epoch
            ((blocks - 1) / DIFFICULTY_ADJUSTMENT_INTERVAL) * DIFFICULTY_ADJUSTMENT_INTERVAL
        }
        2 => {
            // Get the block from ~24 hours ago (144 blocks back)
            blocks.saturating_sub(143)
        }
        _ => {
            return Err(MyError::CustomError(
                "Invalid mode. Use 1 for Epoch Start Block or 2 for 24H Block.".to_string(),
            ));
        }
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // Step 1: Get the block hash by height.
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

    // Extract the block hash.
    let blockhash = block_hash_response.result;

        let getblock_request = 
            json!({
                "jsonrpc": "1.0",
                "id": "1",
                "method": "getblock",
                "params": [blockhash]  // verbose=2
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

/// Capture block info with verbose = 2.
/// Returns full transaction data with block information.  
async fn fetch_full_block_data_by_height(
    config: &RpcConfig,
    blocks: &u64,
) -> Result<BlockInfoFull, MyError> {
    // Determine which block height to fetch
    // let block_height = blocks;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // Step 1: Get the block hash by height.
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

    // Extract the block hash.
    let blockhash = block_hash_response.result;

    let getblock_request = 
        json!({
            "jsonrpc": "1.0",
            "id": "1",
            "method": "getblock",
            "params": [blockhash, 2]  // verbose=2
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

/// Fetches the miner for the block passed and adds them to BlockHistory.
pub async fn fetch_miner(
    config: &RpcConfig,
    miners_data: &MinersData,
    current_block: &u64,
) -> Result<(), MyError> {
    // Fetch the latest block data with verbose=2
    let block = fetch_full_block_data_by_height(config, &current_block).await?;

    // Extract the coinbase transaction directly from the block
    let coinbase_tx = &block.tx[0]; // First transaction is the coinbase
    let coinbase_tx_addresses = coinbase_tx.extract_wallet_addresses();

    // Find the miner associated with the wallet address
    let miner = find_miner_by_wallet(coinbase_tx_addresses, miners_data).await
        .unwrap_or("Unknown".to_string()); // Use "Unknown" if no miner is found

    // Add the miner to the BlockHistory (oldest block is automatically removed if full)
    let block_history = BLOCK_HISTORY.write().await; 
    block_history.add_block(Some(miner.into())); // Convert &str to String here

    Ok(())
}

/// Matches coinbase vout wallet adresess(es) against list of known miners and thier wallets from miners.json and returns the miner or none.
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
