
// rpc/block.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::block_info::{BlockHash, BlockInfo, BlockInfoJsonWrap};
use crate::utils::DIFFICULTY_ADJUSTMENT_INTERVAL;

// Fetch block data based on the block height.
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
