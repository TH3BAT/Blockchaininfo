
// rpc/transaction.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::config::RpcConfig;
use crate::models::errors::MyError;
use chrono::{DateTime, Utc};

pub async fn fetch_transaction(config: &RpcConfig, txid: &str) -> Result<String, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "lookup",
        "method": "getrawtransaction",
        "params": [txid, true]  // ✅ Fetch verbose details
    });

    let client = Client::new();
    let response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let tx = response["result"].as_object();

    if let Some(tx) = tx {
        let total_amount: f64 = tx["vout"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v["value"].as_f64().unwrap_or(0.0))
            .sum();

        // ✅ If blocktime exists, it's confirmed
        if let Some(timestamp) = tx.get("blocktime").and_then(|t| t.as_i64()) {
            let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or("Invalid timestamp".to_string());

            return Ok(format!(
                "Transaction ID: {}\nTotal Amount: {:.8} BTC\nStatus: Confirmed\nTimestamp: {}\nInputs: {}\nOutputs: {}",
                tx["txid"].as_str().unwrap_or("N/A"),
                total_amount,
                datetime,
                tx["vin"].as_array().map_or(0, |v| v.len()),
                tx["vout"].as_array().map_or(0, |v| v.len()),
            ));
        }
    }

    // ❌ No blocktime found -> It's a mempool TX, so we fetch from `getmempoolentry`
    let mempool_request = json!({
        "jsonrpc": "1.0",
        "id": "lookup",
        "method": "getmempoolentry",
        "params": [txid]
    });

    let mempool_response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&mempool_request)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(mempool_tx) = mempool_response["result"].as_object() {
        let timestamp = mempool_tx.get("time")
            .and_then(|t| t.as_i64())
            .unwrap_or(0);  // ✅ Ensure fallback

        let datetime = if timestamp > 0 {
            DateTime::<Utc>::from_timestamp(timestamp, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or("Invalid timestamp".to_string())
        } else {
            "Unknown timestamp".to_string()
        };

        return Ok(format!(
            "Transaction ID: {}\nStatus: Unconfirmed (In Mempool)\nFee: {:.0} sats\nTimestamp: {}",
            txid,
            mempool_tx["fees"]["base"].as_f64().unwrap_or(0.0) * 100_000_000.0, // Convert BTC to sats
            datetime,
        ));
    }

    Err(MyError::CustomError("Transaction not found".to_string()))
}


