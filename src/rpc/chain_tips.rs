
// rpc/chain_tips.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::chaintips_info::{ChainTip, ChainTipsResponse};
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use std::time::Duration;

// Makes an RPC request to fetch chain tips information.
pub async fn fetch_chain_tips(config: &RpcConfig) -> Result<Vec<ChainTip>, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getchaintips",
        "params": []
    });

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getchaintips'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<ChainTipsResponse>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getchaintips.".to_string())
        })?;

    Ok(response.result)
}
