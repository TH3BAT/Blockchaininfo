//! High-level RPC interface layer.
//!
//! This module acts as the public-facing RPC API for the rest of the application.
//! Each function here delegates to a more specialized submodule (`rpc/blockchain.rs`,
//! `rpc/mempool.rs`, etc.), keeping the call surface clean and consistent.
//!
//! All RPCs return deserialized strongly-typed model structs and convert errors
//! into `MyError`, ensuring uniform error handling across the entire codebase.

// ─────────────────────────────────────────────────────────────────────────────
// Submodules for each logical Bitcoin RPC method group.
// These remain private, exposing only typed wrapper functions from this file.
// ─────────────────────────────────────────────────────────────────────────────

/// Handles RPC calls for `getblockchaininfo`.
mod blockchain;

/// Handles RPC calls for `getmempoolinfo` and `getrawmempool`.
mod mempool;

/// Handles RPC calls for `getnetworkinfo`.
mod network;

/// Handles RPC calls for `getblockhash` and `getblock`.
mod block;

/// Handles RPC calls for `getchaintips`.
/// Used for fork and reorg monitoring.
mod chain_tips;

/// Handles RPC calls for `getnettotals`.
mod network_totals;

/// Handles RPC calls for `getpeerinfo`.
/// Used for block propagation calculations.
mod network_peers;

/// Handles RPC calls for `getmempoolentry`.
/// Computes mempool distribution metrics after fetching entries.
mod mempool_distro;

/// Handles RPC calls for `getrawtransaction` and optional mempool lookups.
mod transaction;

// ─────────────────────────────────────────────────────────────────────────────
// Imports for returned model types.
// ─────────────────────────────────────────────────────────────────────────────

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::{BlockInfo, MinersData};
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::chaintips_info::ChainTip;
use crate::models::network_totals::NetTotals;
use crate::models::peer_info::PeerInfo;
use crate::models::errors::MyError;
use crate::config::RpcConfig;

// ─────────────────────────────────────────────────────────────────────────────
// Public RPC wrapper functions.
// Each function provides a clean, typed interface to the internal RPC modules.
// ─────────────────────────────────────────────────────────────────────────────

/// Calls `getblockchaininfo` and returns a fully deserialized `BlockchainInfo` object.
///
/// This RPC is used for:
/// - chain height
/// - difficulty
/// - chainwork
/// - IBD status
/// - time / mediantime
pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    blockchain::fetch_blockchain_info(config).await
}

/// Calls `getmempoolinfo` and returns current mempool statistics.
///
/// Does **not** fetch transaction details — that is handled separately.
pub async fn fetch_mempool_info(config: &RpcConfig) -> Result<MempoolInfo, MyError> {
    mempool::fetch_mempool_info(config).await
}

/// Calls `getnetworkinfo` and returns node-level network metadata.
///
/// Includes:
/// - version / subversion
/// - services bitfield
/// - peer counts
/// - relay/min-fee values
pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    network::fetch_network_info(config).await
}

/// Fetches block data (verbose=1) by height.
///
/// Returns:
/// - Header info
/// - Block metadata
/// - Vector of txids (not full transaction bodies)
///
/// `mode` is used internally to determine whether this height is:
/// - `1` → epoch start block  
/// - `2` → 24-hours-ago block  
pub async fn fetch_block_data_by_height(
    config: &RpcConfig,
    blocks: u64,
    mode: u16, // 1 = Epoch Start Block, 2 = 24 Hours Ago Block
) -> Result<BlockInfo, MyError> {
    block::fetch_block_data_by_height(config, blocks, mode).await
}

/// Calls `getchaintips`.
///
/// Returns all known chain tips including valid forks, stale forks,
/// or unknown headers. Critical for fork detection and monitoring.
pub async fn fetch_chain_tips(config: &RpcConfig) -> Result<Vec<ChainTip>, MyError> {
    chain_tips::fetch_chain_tips(config).await
}

/// Calls `getnettotals`.
///
/// Provides total bytes sent/received and upload target information.
pub async fn fetch_net_totals(config: &RpcConfig) -> Result<NetTotals, MyError> {
    network_totals::fetch_net_totals(config).await
}

/// Calls `getpeerinfo`.
///
/// Used heavily for:
/// - version distribution  
/// - client distribution  
/// - block propagation timing calculations  
pub async fn fetch_peer_info(config: &RpcConfig) -> Result<Vec<PeerInfo>, MyError> {
    network_peers::fetch_peer_info(config).await
}

/// Fetches mempool entries and calculates the complete mempool distribution.
///
/// `dust_free` controls whether low-vsize “dust” transactions are filtered out.
/// Results are stored in `MempoolDistribution` cache via the distro module.
pub async fn fetch_mempool_distribution(
    config: &RpcConfig,
    dust_free: bool,
) -> Result<(), MyError> {
    mempool_distro::fetch_mempool_distribution(config, dust_free).await
}

/// Fetches a transaction either by:
/// - `getrawtransaction` (confirmed blockchain transactions)
/// - `getmempoolentry` (unconfirmed mempool transactions)
///
/// Returns a serialized JSON string for display in the Transaction Lookup popup.
pub async fn fetch_transaction(config: &RpcConfig, txid: &str) -> Result<String, MyError> {
    transaction::fetch_transaction(config, txid).await
}

/// Reads miner data and determines the miner for the currently best block.
///
/// Used for:
/// - Best Block Miner (top-left of Dashboard)
/// - Hash Rate Distribution chart (rolling 144 blocks)
pub async fn fetch_miner(
    config: &RpcConfig,
    miners_data: &MinersData,
    current_block: &u64,
) -> Result<(), MyError> {
    block::fetch_miner(config, miners_data, current_block).await
}
