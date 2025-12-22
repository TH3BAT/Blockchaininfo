//! Handles transaction lookups via two RPC calls:
//!
//! - `getrawtransaction` — used to retrieve on-chain, confirmed TXs  
//! - `getmempoolentry` — fallback when a TX is unconfirmed  
//!
//! This module powers the **Transaction Lookup pop-up** on the dashboard,
//! giving the user quick access to:
//! - Confirmation status  
//! - Total output value  
//! - Fees (for mempool TXs)  
//! - Timestamp  
//! - Input/output counts  
//! - Presence and value of OP_RETURN outputs  
//!
//! Logic flow:
//! 1. Try `getrawtransaction` (verbose = true)  
//! 2. If TX is confirmed → return formatted on-chain summary  
//! 3. Else → call `getmempoolentry` to retrieve unconfirmed details  
//!
//! Any failure to parse either response returns `"Transaction not found"`.

use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::config::RpcConfig;
use crate::models::errors::MyError;

use chrono::{DateTime, Utc};

use crate::models::transaction_info::GetRawTransactionResponse;
use crate::models::mempool_info::MempoolEntryJsonWrap;
use crate::rpc::client::build_rpc_client;

/// Fetch transaction details from either:
/// - The blockchain (confirmed)  
/// - The mempool (unconfirmed)  
///
/// ### Returns
/// A **formatted multi-line string** that the TUI pop-up prints directly.
///
/// ### Behavior Summary
/// - Calls `getrawtransaction` with verbose mode enabled  
/// - If the TX includes a `blocktime` → confirmed  
/// - Output includes:
///     - TXID  
///     - Total output value  
///     - Confirmation timestamp  
///     - Count of inputs and outputs  
///     - Presence/value of OP_RETURN outputs  
///
/// - If no `blocktime`:
///     - Calls `getmempoolentry`  
///     - Returns fee, timestamp, and OP_RETURN summary  
///
/// ### Error Handling
/// - Timeout → `MyError::TimeoutError`  
/// - Network / RPC error → `MyError::Reqwest`  
/// - Missing or unparsable data → `"Transaction not found"`  
///
/// This function makes the lookup pane intuitive and resilient.
pub async fn fetch_transaction(config: &RpcConfig, txid: &str) -> Result<String, MyError> {

    // --- Attempt 1: Fetch confirmed TX using getrawtransaction ---

    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "lookup",
        "method": "getrawtransaction",
        "params": [txid, true]  // Fetch verbose details (vout, vin, blocktime, etc.)
    });

    // Build HTTP client with tight timeouts for TUI responsiveness
    let client = build_rpc_client()?;

    // Execute getrawtransaction
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

    // Deserialize into typed struct
    let tx: GetRawTransactionResponse = serde_json::from_value(response["result"].clone())
        .map_err(|_e| MyError::CustomError("Transaction not found".to_string()))?;

    // --- If blocktime exists → confirmed TX summary ---
    if let Some(blocktime) = tx.blocktime {

        // Convert UNIX timestamp → human-readable UTC
        let datetime = DateTime::<Utc>::from_timestamp(blocktime as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or("Invalid timestamp".to_string());

        return Ok(format!(
            "Transaction ID: {}\n\
             Total Amount: {:.8} BTC\n\
             Status: Confirmed\n\
             Timestamp: {}\n\
             Inputs: {}\n\
             Outputs: {}\n\
             OP_RETURN Outputs: {} ({:.8} BTC)",
            tx.txid,
            tx.total_output_value(),
            datetime,
            tx.vin.len(),
            tx.vout.len(),
            tx.has_op_return(),
            tx.total_op_return_value().abs(),
        ));
    }

    // --- Attempt 2: TX not in chain, try mempool ---

    let mempool_request = json!({
        "jsonrpc": "1.0",
        "id": "lookup",
        "method": "getmempoolentry",
        "params": [txid]
    });
  
    let wrap = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&mempool_request)
        .send()
        .await
        .map_err(|e| MyError::RpcRequestError(txid.to_string(), e.to_string()))?
        .json::<MempoolEntryJsonWrap>()
        .await
        .map_err(|e| MyError::JsonParsingError(txid.to_string(), e.to_string()))?;

    let mempool_entry = wrap.result;

    // Convert mempool timestamp (if available)
    let datetime = if mempool_entry.time > 0 {
        DateTime::<Utc>::from_timestamp(mempool_entry.time as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or("Invalid timestamp".to_string())
    } else {
        "Unknown timestamp".to_string()
    };

    // --- Return unconfirmed summary ---
    Ok(format!(
        "Transaction ID: {}\n\
         Status: Unconfirmed (In Mempool)\n\
         Fee: {:.0} sats\n\
         Timestamp: {}\n\
         OP_RETURN Outputs: {} ({:.8} BTC)",
        txid,
        mempool_entry.fees.base * 100_000_000.0, // BTC → sats
        datetime,
        tx.has_op_return(),
        tx.total_op_return_value().abs(),
    ))
}
