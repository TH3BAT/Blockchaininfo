
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
) -> Result<((usize, usize, usize), (usize, usize, usize), (usize, usize), f64, f64), MyError> {
    let client = Client::new();
    let mut small = 0;
    let mut medium = 0;
    let mut large = 0;
    let mut young = 0;
    let mut moderate = 0;
    let mut old = 0;
    let mut rbf_count = 0;
    let mut non_rbf_count = 0;
    let mut total_fee = 0.0;
    let mut count = 0;
    let mut fees: Vec<f64> = Vec::new(); // Store all fees for median calculation.

    // Define a dust threshold (e.g., 546 satoshis for standard transactions).
    const DUST_THRESHOLD: f64 = 0.00000546; // 546 satoshis in BTC.
    
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
    
        // Access the result directly.
        let mempool_entry = response.result;

        // Exclude dust transactions
        if mempool_entry.fees.base < DUST_THRESHOLD {
            continue;
        }
    
        // Categorize by transaction size.
        match mempool_entry.vsize {
            0..=249 => small += 1,
            250..=1000 => medium += 1,
            _ => large += 1,
        }
    
        // Categorize by age.
        let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => {
                eprintln!("Failed to get current time: {}", e);
                continue; // Skip this transaction if there's an error.
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
            0..=300 => young += 1,             // < 5 minutes.
            301..=3600 => moderate += 1,       // 5 minutes to 1 hour.
            _ => old += 1,                     // > 1 hour.
        }
    
        // Monitor RBF status.
        if mempool_entry.bip125_replaceable {
            rbf_count += 1;
        } else {
            non_rbf_count += 1;
        }

        // Accumulate fees for average and median calculation.
        total_fee += mempool_entry.fees.base;
        fees.push(mempool_entry.fees.base);
        count += 1;
    }

    // Calculate the average fee.
    let average_fee = if count > 0 { total_fee / count as f64 } else { 0.0 };

    // Calculate the median fee.
    let median_fee = if !fees.is_empty() {
        fees.sort_by(|a, b| a.partial_cmp(b).unwrap()); // Sort the fees.
        let mid = fees.len() / 2;
        if fees.len() % 2 == 0 {
            // Average of two middle values if even number of elements.
            (fees[mid - 1] + fees[mid]) / 2.0
        } else {
            // Middle value if odd number of elements.
            fees[mid]
        }
    } else {
        0.0
    };

    // Return size, age distributions, RBF stats, average fee, and median fee.
    Ok(((small, medium, large), (young, moderate, old), (rbf_count, non_rbf_count), average_fee, median_fee))
}


