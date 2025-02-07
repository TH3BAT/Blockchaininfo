
// rpc/mempool_distro.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::models::mempool_info::MempoolEntryJsonWrap;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub async fn fetch_mempool_distribution(
    config: &RpcConfig,
    all_ids: Vec<String>,
) -> Result<((usize, usize, usize), (usize, usize, usize), (usize, usize), f64, f64, f64), MyError> {
    let client = Client::new();
    let mut dust_free_tx_ids: Vec<String> = Vec::new(); // Store valid transactions.

    // Dust threshold (e.g., 546 satoshis).
    const DUST_THRESHOLD: f64 = 0.00000546; 

    // ✅ **First loop: Filter out dust transactions**
    for tx_id in &all_ids {
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

        let mempool_entry = response.result;

        // ✅ **Filter out dust transactions**
        if mempool_entry.fees.base >= DUST_THRESHOLD {
            dust_free_tx_ids.push(tx_id.clone());
        }
    }

    // ✅ **Second loop: Perform 5% sampling using `partial_shuffle`**
    let sample_size = (dust_free_tx_ids.len() as f64 * 0.05).ceil() as usize;
    let mut rng = StdRng::from_rng(&mut rand::rng()); // ✅ Keep `rand 0.9.0` compatible

    let sampled_tx_ids = if sample_size < dust_free_tx_ids.len() {
        dust_free_tx_ids.partial_shuffle(&mut rng, sample_size).0.to_vec()
    } else {
        dust_free_tx_ids.clone() // If fewer than 20 TXs, take all.
    };

    // ✅ Now we calculate based on the sampled transactions.
    let mut small = 0;
    let mut medium = 0;
    let mut large = 0;
    let mut young = 0;
    let mut moderate = 0;
    let mut old = 0;
    let mut rbf_count = 0;
    let mut non_rbf_count = 0;
    let mut total_fee = 0.0;
    let mut total_vsize = 0;
    let mut count = 0;
    let mut fees: Vec<f64> = Vec::new();

    for tx_id in sampled_tx_ids {
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

        let mempool_entry = response.result;

        // ✅ **Categorize by transaction size**
        match mempool_entry.vsize {
            0..=249 => small += 1,
            250..=1000 => medium += 1,
            _ => large += 1,
        }

        // ✅ **Categorize by age**
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let age = current_time.saturating_sub(mempool_entry.time);
        match age {
            0..=300 => young += 1,            
            301..=3600 => moderate += 1,       
            _ => old += 1,                     
        }

        // ✅ **Monitor RBF status**
        if mempool_entry.bip125_replaceable {
            rbf_count += 1;
        } else {
            non_rbf_count += 1;
        }

        // ✅ **Accumulate fees and sizes**
        let total_entry_fee = mempool_entry.fees.base
            + mempool_entry.fees.ancestor
            + mempool_entry.fees.modified
            + mempool_entry.fees.descendant;

        total_fee += total_entry_fee;
        total_vsize += mempool_entry.vsize;
        fees.push(total_entry_fee);
        count += 1;
    }

    // ✅ **Calculate fees and fee rates**
    let average_fee = if count > 0 { total_fee / count as f64 } else { 0.0 };
    let median_fee = if !fees.is_empty() {
        fees.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = fees.len() / 2;
        if fees.len() % 2 == 0 {
            (fees[mid - 1] + fees[mid]) / 2.0
        } else {
            fees[mid]
        }
    } else {
        0.0
    };

    let average_fee_rate = if total_vsize > 0 {
        (total_fee * 100_000_000.0) / total_vsize as f64
    } else {
        0.0
    };

    // ✅ **Return results**
    Ok((
        (small, medium, large),
        (young, moderate, old),
        (rbf_count, non_rbf_count),
        average_fee,
        median_fee,
        average_fee_rate, 
    ))
}




