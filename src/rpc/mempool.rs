
// rpc/mempool.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use rand::seq::SliceRandom;
use crate::models::mempool_info::{MempoolInfoJsonWrap, MempoolInfo, 
    RawMempoolTxsJsonWrap};
use crate::models::errors::MyError;
use crate::config::RpcConfig;

// Fetches mempool information and samples raw transactions.
pub async fn fetch_mempool_info(
    config: &RpcConfig,
    sample_percentage: f64, // Percentage of transactions to sample (0.0 to 100.0)
) -> Result<(MempoolInfo, Vec<String>), MyError> {
    // Step 1: Fetch mempool information (to get the transaction count).
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getmempoolinfo",
        "params": []
    });

    let client = Client::new();
    let response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await?
        .json::<MempoolInfoJsonWrap>()
        .await?;

    let mempool_info = response.result;
    let total_transactions = mempool_info.size; // Number of transactions in the mempool.

    // Step 2: Calculate the sample size based on the percentage provided.
    let sample_size = ((sample_percentage / 100.0) * total_transactions as f64).round() as usize;

    // Step 3: Fetch raw mempool transactions (limited to the sample size).
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "2",
        "method": "getrawmempool",
        "params": [false] // false to return transaction IDs only.
    });

    let raw_mempool_response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await?
        .json::<RawMempoolTxsJsonWrap>() // Use generic JSON to handle a large list.
        .await?;

    // Extract transaction IDs (Vec<String>) from the response.
    let all_tx_ids = raw_mempool_response.result;

    // Step 4: Randomly sample the transactions (if sample size is smaller than total transactions).
    let mut rng = &mut rand::thread_rng();
    let sampled_tx_ids = if sample_size < all_tx_ids.len() {
        all_tx_ids.choose_multiple(&mut rng, sample_size).cloned().collect()
    } else {
        all_tx_ids
    };

    Ok((mempool_info, sampled_tx_ids))
}
