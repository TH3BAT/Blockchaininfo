//! RPC handlers for block-related Bitcoin Core methods.
//!
//! This module is responsible for:
//! - Fetching block hashes by height (`getblockhash`)
//! - Fetching block data with verbose=1 (header + txids)
//! - Fetching full block data with verbose=2 (header + full tx objects)
//! - Determining the miner via coinbase parsing
//! - Updating `BLOCK_HISTORY` for the Hash Rate Distribution chart
//!
//! This file represents one of the most critical paths in the dashboard,
//! powering epoch calculations, 24h difficulty drift, miner extraction,
//! and the UI’s block/txid displays.

use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::errors::MyError;
use crate::models::block_info::Transaction;
use crate::config::RpcConfig;
use crate::rpc::client::build_rpc_client;

use crate::models::block_info::{
    BlockHash,
    BlockInfo,
    BlockInfoJsonWrap,
    MinersData,
    BlockInfoFull,
    BlockInfoFullJsonWrap,
};

use crate::utils::{BLOCK_HISTORY, squash_alnum_lower};
use crate::consensus::satoshi_math::*;

/// Fetch block information at a specific height using `getblock` with verbose=1.
///
/// ### Purpose
/// This RPC is used in two contexts:
/// - **Epoch Start Block (mode = 1)**  
///   Determines the starting block of the difficulty epoch by rounding down
///   to the nearest 2016-block boundary.
/// - **Past 24 Hours Block (mode = 2)**  
///   Used for 24h difficulty drift calculations by moving back ~144 blocks.
///
/// Returns:
/// - `BlockInfo` (header + vector of txids)
///
/// Errors:
/// - Timeout
/// - Reqwest network error
/// - JSON parsing error
/// - Custom error for invalid mode
pub async fn fetch_block_data_by_height(
    config: &RpcConfig,
    blocks: u64,
    mode: u16, // 1 = Epoch Start Block, 2 = 24 Hours Ago Block
) -> Result<BlockInfo, MyError> {

    // Determine target block height
    let block_height = match mode {
        1 => {
            // Find first block in the current difficulty epoch
            ((blocks - 1) / DIFFICULTY_ADJUSTMENT_INTERVAL) * DIFFICULTY_ADJUSTMENT_INTERVAL
        }
        2 => {
            // Approx. block height 24 hours ago (~144 blocks)
            blocks.saturating_sub((BLOCKS_PER_HOUR * HOURS_PER_DAY) - 1)
        }
        _ => {
            return Err(MyError::CustomError(
                "Invalid mode. Use 1 for Epoch Start Block or 2 for 24H Block.".to_string(),
            ));
        }
    };

    // RPC client with timeouts tailored for TUI responsiveness
    let client = build_rpc_client()?;

    // ──────────────────────────────
    // Step 1: getblockhash
    // ──────────────────────────────
    let getblockhash_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblockhash",
        "params": [block_height]
    });

    let block_hash_response: BlockHash = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblockhash_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblockhash'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockHash>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblockhash.".to_string())
        })?;

    let blockhash = block_hash_response.result;

    // ──────────────────────────────
    // Step 2: getblock (verbose = 1)
    // ──────────────────────────────
    let getblock_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblock",
        "params": [blockhash] // default verbose=1
    });

    let block_response: BlockInfoJsonWrap = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblock_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblock'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockInfoJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblock.".to_string())
        })?;

    Ok(block_response.result)
}

/// Fetch full block data with verbose=2.
///
/// ### Purpose
/// This internal helper retrieves:
/// - Complete transaction objects
/// - Useful for miner extraction through coinbase parsing
///
/// Not exposed publicly because full block data is used only internally
/// for miner identification.
async fn fetch_full_block_data_by_height(
    config: &RpcConfig,
    blocks: &u64,
) -> Result<BlockInfoFull, MyError> {

    let client = build_rpc_client()?;

    // ──────────────────────────────
    // Step 1: getblockhash
    // ──────────────────────────────
    let getblockhash_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblockhash",
        "params": [*blocks]
    });

    let block_hash_response: BlockHash = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblockhash_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblockhash'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockHash>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblockhash.".to_string())
        })?;

    let blockhash = block_hash_response.result;

    // ──────────────────────────────
    // Step 2: getblock (verbose = 2)
    // ──────────────────────────────
    let getblock_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblock",
        "params": [blockhash, 2]  // Return full tx objects
    });

    let block_response: BlockInfoFullJsonWrap = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&getblock_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblock'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<BlockInfoFullJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblock.".to_string())
        })?;

    Ok(block_response.result)
}

/// Parse the miner for the current block and append them to `BLOCK_HISTORY`.
///
/// ### Workflow:
/// 1. Fetch full block data using verbose=2  
/// 2. Extract the coinbase transaction  
/// 3. Parse wallet addresses from the coinbase output  
/// 4. Match the address to known miners from `miners.json`  
/// 5. Append result to rolling `BlockHistory` (used for hash rate distribution chart)
///
/// If no miner match is found, `"Unknown"` is used.
pub async fn fetch_miner(
    config: &RpcConfig,
    miners_data: &MinersData,
    current_block: &u64,
) -> Result<(), MyError> {

    // Always fetch with verbose=2 for miner identification
    let block = fetch_full_block_data_by_height(config, &current_block).await?;

    // Coinbase is always tx[0]
    let coinbase_tx = &block.tx[0];
    let coinbase_tx_addresses = coinbase_tx.extract_wallet_addresses();

    // Attempt miner lookup (wallet-based)
    let wallet_miner = find_miner_by_wallet(coinbase_tx_addresses, miners_data).await;

    let need_coinbase = wallet_miner.is_none()
        || matches!(wallet_miner.as_deref(), Some("OCEAN"));

    let miner: String = if !need_coinbase {
        // Normal path: trust wallet
        wallet_miner.clone().unwrap_or_else(|| "Unknown".to_string())
    } else if let Some((primary_raw, secondary_raw)) = classify_miner_from_coinbase(coinbase_tx) {
        let primary = clean_coinbase_label(&primary_raw);
        let secondary = clean_secondary(secondary_raw);

        // B) Wallet says OCEAN → coinbase can reveal upstream identity
        if matches!(wallet_miner.as_deref(), Some("OCEAN")) {
            // If coinbase primary is more specific than OCEAN, show it (optionally "via OCEAN")
            if !primary.is_empty() && primary != "OCEAN" {
                format!("{primary} (via OCEAN)")
                // or just: primary
            } else {
                "OCEAN".to_string()
            }
        } else {
            // A) Wallet unknown → coinbase fallback (with optional "via <pool>")
            match secondary {
                Some(pool) => format!("{primary} (via {pool})"),
                None => {
                    if primary.is_empty() { "Unknown".to_string() } else { primary }
                }
            }
        }
    } else {
        // coinbase parse failed → fallback to wallet or Unknown
        wallet_miner.clone().unwrap_or_else(|| "Unknown".to_string())
    };

    // Append into rolling history
    let block_history = BLOCK_HISTORY.write().await;
    block_history.add_block(Some(miner.into()));

    Ok(())
}

/// Matches extracted coinbase addresses to known miners from miners.json.
///
/// Returns:
/// - `Some(miner_name)` if a match is found  
/// - `None` otherwise  
///
/// Miner identification relies entirely on wallet labels provided in miners.json.
async fn find_miner_by_wallet(addresses: Vec<String>, miners_data: &MinersData) -> Option<String> {
    for address in addresses {
        if let Some(miner) = miners_data.miners.iter()
            .find(|miner| miner.wallet == address)
            .map(|miner| miner.name.clone())
        {
            return Some(miner);
        }
    }
    None
}

/// Classify a miner name from the coinbase transaction tag.
///
/// This inspects the **coinbase scriptSig hex** (txin[0].coinbase) and extracts
/// printable ASCII “runs” (e.g., `/Foundry USA Pool/`, `Mined by AntPool`,
/// `< OCEAN.XYZ > NiceHash`, etc.). It then applies lightweight heuristics to
/// derive a human-readable miner label.
///
/// ## Return value
/// Returns `Some((primary, secondary))` where:
/// - `primary`: the best miner label to display (often the pool name)
/// - `secondary`: optional pool / coordinator context when the tag contains both
///   a pool and an upstream hash provider / sub-miner (e.g., `NiceHash (via OCEAN)`).
///
/// Returns `None` if the transaction has no usable coinbase tag.
///
/// ## Design notes
/// - This is a **best-effort fallback**. The primary miner identification signal
///   remains the coinbase payout address lookup (`miners.json`).
/// - Coinbase tags are not standardized and may include arbitrary bytes, emojis,
///   padding, or non-printable delimiters. We intentionally search for printable
///   ASCII sequences and ignore the rest.
/// - Some pools embed additional identifiers (e.g., OCEAN sub-miner labels).
///   For these, we try to extract a short “human-ish” token as `primary` and
///   return the pool name as `secondary`.
///
/// ## Heuristics (high-level)
/// - Extract printable ASCII runs (min length configurable, typically 4).
/// - Detect strong signatures for common pools (Foundry, AntPool, etc.).
/// - Special-case pools that embed upstream/miner identifiers (e.g., OCEAN).
/// - Filter out junk runs: very short tokens, `mm...` padding, long hex blobs,
///   and strings without letters.
/// - Prefer short, readable labels (<= 32 chars) to avoid UI truncation.
///
/// ## Caveats
/// - A coinbase tag can lie. This is informational only.
/// - Some tags include “Mined by …” prefixes; callers may want to normalize or
///   prefer wallet-based identification when available.
/// - This function performs **no** consensus-critical parsing—display use only.
/// - This is intentionally extensible: add new signature rules conservatively to
///   avoid false positives.

fn classify_miner_from_coinbase(tx: &Transaction) -> Option<(String, Option<String>)> {
    let runs = tx.extract_coinbase_ascii_runs(4);
    if runs.is_empty() {
        return None;
    }
   
    // Pre-scan: is Ocean present anywhere?
    let ocean_present = runs.iter().any(|r| {
        let sig = squash_alnum_lower(r);
        sig.contains("oceanxyz") || sig == "ocean"
    });

    let mut pool: Option<String> = None;
    let mut best_miner: Option<String> = None;

    // First pass: strong signatures (but don't short-circuit if Ocean is present)
    for r in &runs {
        let sig = squash_alnum_lower(r);

        if sig.contains("oceanxyz") || sig == "ocean" {
            pool = Some("OCEAN".to_string());
            continue;
        }

        // If Ocean is present, we want these tokens to be candidates for sub-miner,
        // not an immediate "primary miner" return.
        if ocean_present {
            continue;
        }

        if sig.contains("nicehash") {
            return Some(("NiceHash".to_string(), None));
        }
        if sig.contains("antpool") {
            return Some(("AntPool".to_string(), None));
        }
        if sig.contains("foundryusapool") || sig.contains("2cdw") {
            return Some(("Foundry USA".to_string(), None));
        }
        if sig.contains("f2pool") {
            return Some(("F2Pool".to_string(), None));
        }
        if sig.contains("viabtc") {
            return Some(("ViaBTC".to_string(), None));
        }
        if sig.contains("luxor") {
            return Some(("Luxor".to_string(), None));
        }
        if sig.contains("braiins") || sig.contains("slush") {
            return Some(("Braiins Pool".to_string(), None));
        }
        if sig.contains("btccom") {
            return Some(("BTC.com".to_string(), None));
        }
        if sig.contains("poolin") {
            return Some(("Poolin".to_string(), None));
        }
        if sig.contains("binance") {
            return Some(("Binance Pool".to_string(), None));
        }
        if sig.contains("secpool") {
            return Some(("SECPOOL".to_string(), None));
        }
        if sig.contains("marapool") || sig.contains("maramadeinusa"){
            return Some(("MARA Pool".to_string(), None));
        }
          if sig.contains("spiderpool") {
            return Some(("SpiderPool".to_string(), None));
        }
          if sig.contains("whitepool") {
            return Some(("WhitePool".to_string(), None));
        }
          if sig.contains("sbicrypto") {
            return Some(("SBI Crypto".to_string(), None));
        }
          if sig.contains("ultimus") {
            return Some(("ULTIMUSPOOL".to_string(), None));
        }
          if sig.contains("gdpool") || sig.contains("luckypool") {
            return Some(("GDPool".to_string(), None));
        }
          if sig.contains("redrock") {
            return Some(("RedRock Pool".to_string(), None));
        }
          if sig.contains("innopolis") {
            return Some(("Innopolis Tech".to_string(), None));
        }
          if sig.contains("miningdutch") {
            return Some(("Mining-Dutch".to_string(), None));
        }
          if sig.contains("bitfufu") {
            return Some(("BitFuFuPool".to_string(), None));
        }
          if sig.contains("est3lar") {
            return Some(("Est3lar".to_string(), None));
        }
          if sig.contains("1thash") {
            return Some(("1THash".to_string(), None));
        }
          if sig.contains("maxipool") {
            return Some(("MaxiPool".to_string(), None));
        }
          if sig.contains("publicpool") {
            return Some(("Public Pool".to_string(), None));
        }
          if sig.contains("kano") {
            return Some(("KanoPool".to_string(), None));
        }
          if sig.contains("miningsquared") || sig.contains("bsquared") {
            return Some(("Mining Squared".to_string(), None));
        }
          if sig.contains("phoenix") {
            return Some(("Phoenix".to_string(), None));
        }
          if sig.contains("neopool") {
            return Some(("Neopool".to_string(), None));
        }
        // Solo / small pools
        if sig.contains("solockpoolorg") {
            return Some(("Solo CK".to_string(), None));
        }
        if sig.contains("solopoolcom") {
            return Some(("SoloPool".to_string(), None));
        }
        if sig.contains("apollo") || sig.contains("minedbyasolofuturebitapollo") {
            return Some(("FutureBit Apollo Solo".to_string(), None));
        }

    }

    // Second pass: if Ocean present, pick best human-ish token as sub-miner.
    if pool.is_some() {
        for r in &runs {
            let sig = squash_alnum_lower(r);

            if sig.contains("oceanxyz") || sig == "ocean" {
                continue;
            }

            if sig.starts_with("mm") || sig.len() < 4 {
                continue;
            }

            let looks_like_hex =
                sig.len() >= 32 && sig.chars().all(|c| c.is_ascii_hexdigit());
            if looks_like_hex {
                continue;
            }

            if !r.chars().any(|c| c.is_ascii_alphabetic()) {
                continue;
            }

            let trimmed = r.trim();
            if trimmed.len() > 32 {
                continue;
            }

            best_miner = Some(trimmed.to_string());
            break; // <- pick first good candidate (avoid last-run wins)
        }

        if let Some(m) = best_miner {
            return Some((m, pool));
        }

        return Some(("OCEAN".to_string(), None));
    }

    runs.into_iter()
        .find(|r| r.chars().any(|c| c.is_ascii_alphabetic()))
        .map(|r| (r, None))
}

fn clean_coinbase_label(s: &str) -> String {
    let filtered: String = s
        .chars()
        .filter(|c| c.is_ascii() && !c.is_ascii_control())
        .collect();

    filtered.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clean_secondary(opt: Option<String>) -> Option<String> {
    opt.and_then(|s| {
        let s = clean_coinbase_label(&s);
        if s.is_empty() || s == "0" { None } else { Some(s) }
    })
}

