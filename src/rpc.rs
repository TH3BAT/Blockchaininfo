
// rpc.rs

/// Module handles RPC method calls for getblockchaininfo.
mod blockchain;
/// Module handles RPC method calls for getmempoolinfo & getrawmempool.
mod mempool;
/// Module handles RPC method calls for getnetworkinfo.
mod network;
/// Module handles RPC method calls for getblockhash & getblock.
mod block;
/// Module handles RPC method calls for getchaintips.
mod chain_tips;
/// Module handles RPC method calls for getnettotals.
mod network_totals; 
/// Module handles RPC method calls for getpeerinfo.
mod network_peers;
/// Module handles RPC method calls for getmempoolentry. Processes and updates mempool distribution metrics.
mod mempool_distro;
/// Module handles RPC method calls for getrawtransaction & getmempoolentry.
mod transaction;

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::{BlockInfo,  MinersData};
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::chaintips_info::ChainTip;
use crate::models::network_totals::NetTotals; 
use crate::models::peer_info::PeerInfo;
use crate::models::errors::MyError;
use crate::config::RpcConfig;

/// Calls getblockchaininfo method and deserializes into BlockchainInfo struct.
pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    blockchain::fetch_blockchain_info(config).await
}

/// Calls getmempoolinfo method and deserializes into MempoolInfo struct.
pub async fn fetch_mempool_info(config: &RpcConfig) -> Result<MempoolInfo, MyError> {
    mempool::fetch_mempool_info(config).await
}

/// Calls getnetworkinfo method and deserializes into NetworkInfo struct.
pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    network::fetch_network_info(config).await
}

/// Capture block info with verbose = 1. 
/// Returns block information with Vec of TxIDs.  
pub async fn fetch_block_data_by_height(
    config: &RpcConfig,
    blocks: u64,
    mode: u16, // 1 = Epoch Start Block, 2 = 24 Hours Ago Block
) -> Result<BlockInfo, MyError> {
    block::fetch_block_data_by_height(config, blocks, mode).await
}

/// Calls getchaintips method and deserializes into ChainTips struct. Used for Fork Monitoring.
pub async fn fetch_chain_tips(config: &RpcConfig) -> Result<Vec<ChainTip>, MyError> {
    chain_tips::fetch_chain_tips(config).await
}

/// Calls getnettotals method and deserialzes into NetTotals struct.
pub async fn fetch_net_totals(config: &RpcConfig) -> Result<NetTotals, MyError> {
    network_totals::fetch_net_totals(config).await
}

/// Calls getpeerinfo method and deserializes into PeerInfo struct. Used for block propagation times.
pub async fn fetch_peer_info(config: &RpcConfig) -> Result<Vec<PeerInfo>, MyError> {
    network_peers::fetch_peer_info(config).await
}

/// Fetches mempool transactions, processes dust-free, and collects the metrics into MempoolDistribution struct.
pub async fn fetch_mempool_distribution(
    config: &RpcConfig,
    dust_free: bool, 
) -> Result<(), MyError> {
    mempool_distro::fetch_mempool_distribution(config, dust_free).await
}

/// Fetches transaction and returns either confirmed on-chain tx or mempool status.
/// Used in Transaction Lookup pop-up.
pub async fn fetch_transaction(config: &RpcConfig, txid: &str) -> Result<String, MyError> {
    transaction::fetch_transaction(config, txid).await
}

/// Reads miners.json file into MinersData struct. Used for Best block miner and Hash Rate Distribution chart.
pub async fn fetch_miner(
    config: &RpcConfig,
    miners_data: &MinersData,
    current_block: &u64,
) -> Result<(), MyError> {
    block::fetch_miner(config, miners_data, current_block).await
}