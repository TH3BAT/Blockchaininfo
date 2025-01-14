
// rpc/mempool_distro.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;

pub async fn fetch_mempool_distribution(
    config: &RpcConfig,
    sample_ids: Vec<String>,
) -> Result<((usize, usize, usize), (usize, usize, usize), (usize, usize)), MyError> {
    let client = Client::new();
    let mut small = 0;
    let mut medium = 0;
    let mut large = 0;
    let mut young = 0;
    let mut moderate = 0;
    let mut old = 0;
    let mut rbf_count = 0;
    let mut non_rbf_count = 0;

    for tx_id in sample_ids {
        let json_rpc_request = json!({
            "jsonrpc": "1.0",
            "id": "1",
            "method": "getmempoolentry",
            "params": [tx_id]
        });

        let response = client
            .post(&config.address)
            .basic_auth(&config.username, Some(&config.password))
            .header(CONTENT_TYPE, "application/json")
            .json(&json_rpc_request)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(entry) = response.get("result") {
            // Categorize by transaction size
            if let Some(vsize) = entry.get("vsize").and_then(|v| v.as_u64()) {
                match vsize {
                    0..=249 => small += 1,
                    250..=1000 => medium += 1,
                    _ => large += 1,
                }
            }

            // Categorize by age
            if let Some(time) = entry.get("time").and_then(|t| t.as_u64()) {
                let age = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    Ok(duration) => duration.as_secs() - time,
                    Err(e) => {
                        eprintln!("Failed to compute age: {}", e);
                        continue; // Skip this transaction if there's an error
                    }
                };

                match age {
                    0..=300 => young += 1,             // < 5 minutes
                    301..=3600 => moderate += 1,       // 5 minutes to 1 hour
                    _ => old += 1,                     // > 1 hour
                }
            }

            // Monitor RBF status
            if let Some(rbf) = entry.get("bip125-replaceable").and_then(|r| r.as_bool()) {
                if rbf {
                    rbf_count += 1;
                } else {
                    non_rbf_count += 1;
                }
            }
        }
    }

    // Return both size, age distributions, and RBF stats
    Ok(((small, medium, large), (young, moderate, old), (rbf_count, non_rbf_count)))
}

