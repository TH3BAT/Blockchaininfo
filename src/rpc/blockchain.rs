//
// rpc/blockchain.rs
//
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::errors::{RpcConfig, MyError};


// Makes an RPC request to fetch blockchain information.
pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblockchaininfo",
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
        .json::<BlockchainInfo>()
        .await?;

    Ok(response)
}