
// rpc/network_peers.rs

use crate::models::peer_info::{PeerInfo, PeerInfoResponse};
use crate::config::RpcConfig;
use crate::models::errors::MyError;
use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::json;

pub async fn fetch_peer_info(config: &RpcConfig) -> Result<Vec<PeerInfo>, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getpeerinfo",
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
        .json::<PeerInfoResponse>() 
        .await?;

    Ok(response.result) 
}
