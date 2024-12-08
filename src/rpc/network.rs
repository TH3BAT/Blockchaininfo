//
// rpc/network.rs
//
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::{NetworkInfo, NetworkInfoResponse, RpcConfig, MyError};


// Makes an RPC request to fetch network information
pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getnetworkinfo",
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
        .json::<NetworkInfoResponse>()
        .await?;

    Ok(response.result)
}
