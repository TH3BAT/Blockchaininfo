//
// rpc/mempool.rs
//
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::mempool_info::{MempoolInfoJsonWrap, MempoolInfo};
use crate::models::errors::{RpcConfig, MyError};

// Makes an RPC request to fetch mempool information.
pub async fn fetch_mempool_info(config: &RpcConfig) -> Result<MempoolInfo, MyError> {
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

    Ok(response.result)
}