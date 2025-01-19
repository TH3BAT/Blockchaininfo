
// rpc/mempool_distro.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::mempool_info::MempoolEntryJsonWrap;
use std::time::{SystemTime, UNIX_EPOCH};

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
            .json::<MempoolEntryJsonWrap>()
            .await?;
    
        // Access the result directly
        let mempool_entry = response.result;
    
        // Categorize by transaction size
        match mempool_entry.vsize {
            0..=249 => small += 1,
            250..=1000 => medium += 1,
            _ => large += 1,
        }
    
        // Categorize by age
        let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => {
                eprintln!("Failed to get current time: {}", e);
                continue; // Skip this transaction if there's an error
            }
        };

        let age = match mempool_entry.time {
            time if time <= current_time => current_time - time,
            _ => {
                eprintln!("Invalid time field: {}", mempool_entry.time);
                continue;
            }
        };
    
        match age {
            0..=300 => young += 1,             // < 5 minutes
            301..=3600 => moderate += 1,       // 5 minutes to 1 hour
            _ => old += 1,                     // > 1 hour
        }
    
        // Monitor RBF status
        if mempool_entry.bip125_replaceable {
            rbf_count += 1;
        } else {
            non_rbf_count += 1;
        }
    }

    // Return both size, age distributions, and RBF stats
    Ok(((small, medium, large), (young, moderate, old), (rbf_count, non_rbf_count)))
}
