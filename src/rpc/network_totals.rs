
// rpc/network_totals.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::network_totals::{NetTotalsJsonWrap, NetTotals};
use crate::models::errors::MyError;
use crate::config::RpcConfig;

// Fetch total network bytes sent and received.
pub async fn fetch_net_totals(config: &RpcConfig) -> Result<NetTotals, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getnettotals",
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
        .json::<NetTotalsJsonWrap>() 
        .await?;

    Ok(response.result)
}
