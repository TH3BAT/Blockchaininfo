/// ----------------------------------------------------------------------------
/// RPC: getnetworkhashps
/// ----------------------------------------------------------------------------
/// Fetches an estimated network hashrate (in hashes per second) using Bitcoin
/// Core’s `getnetworkhashps` RPC method.
///
/// Parameters:
/// - `nblocks`:
///     Number of blocks to use for estimation.
///     Common usage in BCI:
///         144 ≈ ~24 hours (stable, operator-friendly window)
///     Note:
///         -1 uses blocks since last difficulty adjustment (not used here).
///
/// - `height`:
///     Block height at which to calculate the estimate.
///     Allows sampling hashrate at specific points in the chain (used for
///     epoch-phase checkpoints).
///
/// Behavior:
/// - Sends a JSON-RPC request to the configured node.
/// - Expects a scalar numeric response (f64) representing hashes per second.
/// - No wrapper struct is used since the RPC result is a single value.
///
/// Error handling:
/// - Distinguishes timeout errors from general request failures.
/// - Validates that the returned JSON contains a numeric `result`.
///
/// Return:
/// - `Ok(f64)`:
///     Estimated network hashrate in H/s.
///     Typically converted to EH/s (÷ 1e18) at render time.
/// - `Err(MyError)`:
///     On request failure, timeout, or invalid/missing response.
///
/// Notes:
/// - This is an *estimate*, not an exact measurement.
/// - Used in BCI for epoch-phase sampling (10/25/50/75/100%).
/// - Designed for observational context, not precise accounting.
/// ----------------------------------------------------------------------------
use crate::models::errors::MyError;
use crate::rpc::client::build_rpc_client;
use crate::config::RpcConfig;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

pub async fn getnetworkhashps(
    config: &RpcConfig,
    nblocks: i64,
    height: i64,
) -> Result<f64, MyError> {

    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getnetworkhashps",
        "params": [nblocks, height]
    });

    let client = build_rpc_client()?;

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
                    "Request to {} timed out for method 'getnetworkhashps'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getnetworkhashps.".to_string())
        })?;

    let hashrate = response["result"]
        .as_f64()
        .ok_or_else(|| {
            MyError::CustomError("Invalid hashrate value returned.".to_string())
        })?;

    Ok(hashrate)
}