//
// rpc/chain_tips.rs
//
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::chaintips_info::{ChainTip, ChainTipsResponse};
use crate::models::errors::MyError;
use crate::config::RpcConfig;

// Makes an RPC request to fetch chain tips information.
pub async fn fetch_chain_tips(config: &RpcConfig) -> Result<Vec<ChainTip>, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getchaintips",
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
        .json::<ChainTipsResponse>()
        .await?;

    Ok(response.result)
}
