
// rpc/block.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::block_info::{BlockHash, BlockInfo, BlockInfoJsonWrap};

// Fetch block data based on the block height.
pub async fn fetch_block_data_by_height(config: &RpcConfig, block_height: u64
) -> Result<BlockInfo, MyError> {
    let client = Client::new();

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
        .await?
        .json::<BlockHash>()
        .await?;
    
    // Extract the block hash.
    let blockhash = block_hash_response.result;
    
    // Step 2: Get the block data using the block hash.
    let getblock_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblock",
        "params": [blockhash]
    });   
    
    let block_response: BlockInfoJsonWrap = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblock_request)
        .send()
        .await?
        .json::<BlockInfoJsonWrap>()
        .await?;
    
     Ok(block_response.result)
    
}