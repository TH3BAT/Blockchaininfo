
// rpc/network_totals.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::network_totals::{NetTotalsJsonWrap, NetTotals};
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use std::time::Duration;

// Fetch total network bytes sent and received.
pub async fn fetch_net_totals(config: &RpcConfig) -> Result<NetTotals, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getnettotals",
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
                    "Request to {} timed out for method 'getnettotals'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<NetTotalsJsonWrap>() 
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getnettotals.".to_string())
        })?;

    Ok(response.result)
}
