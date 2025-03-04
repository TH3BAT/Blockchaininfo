
// rpc/transaction.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::config::RpcConfig;
use crate::models::errors::MyError;
use chrono::{DateTime, Utc};
use crate::models::transaction_info::GetRawTransactionResponse;
use crate::models::mempool_info::MempoolEntry;
use std::time::Duration;

pub async fn fetch_transaction(config: &RpcConfig, txid: &str) -> Result<String, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "lookup",
        "method": "getrawtransaction",
        "params": [txid, true]  // Fetch verbose details
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
                    "Request to {} timed out for method 'getrawtransaction'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<serde_json::Value>()
        .await?;

    // Deserialize the response into our struct
    let tx: GetRawTransactionResponse = serde_json::from_value(response["result"].clone())
    .map_err(|_e| {
        MyError::CustomError("Transaction not found".to_string())
    })?;

     // Check if the transaction is confirmed
     if let Some(blocktime) = tx.blocktime {
        let datetime = DateTime::<Utc>::from_timestamp(blocktime as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or("Invalid timestamp".to_string());

        // Decipher OP_RETURN messages (if any)
        // let op_return_messages = tx.get_op_return_msg();

        return Ok(format!(
            "Transaction ID: {}\nTotal Amount: {:.8} BTC\nStatus: Confirmed\nTimestamp: {}\nInputs: {}\nOutputs: {}\nOP_RETURN Outputs: {} ({:.8} BTC)",
            tx.txid,
            tx.total_output_value(),
            datetime,
            tx.vin.len(),
            tx.vout.len(),
            tx.has_op_return(),
            tx.total_op_return_value().abs(),
            // op_return_messages,
        ));
    }

    // If not confirmed, fetch from mempool
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
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getmempoolentry'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<serde_json::Value>()
        .await?;

    // Deserialize the response into MempoolEntry
    let mempool_entry: MempoolEntry = serde_json::from_value(mempool_response["result"].clone())
        .map_err(|_e| {
            MyError::CustomError("Transaction not found".to_string())
        })?;

    let datetime = if mempool_entry.time > 0 {
        DateTime::<Utc>::from_timestamp(mempool_entry.time as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or("Invalid timestamp".to_string())
    } else {
        "Unknown timestamp".to_string()
    };

    return Ok(format!(
        "Transaction ID: {}\nStatus: Unconfirmed (In Mempool)\nFee: {:.0} sats\nTimestamp: {}\nOP_RETURN Outputs: {} ({:.8} BTC)",
        txid,
        mempool_entry.fees.base * 100_000_000.0, // Convert BTC to sats
        datetime,
        tx.has_op_return(),
        tx.total_op_return_value().abs(),
    ));

}

