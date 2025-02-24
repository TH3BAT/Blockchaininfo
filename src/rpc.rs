
// rpc.rs

mod blockchain;
mod mempool;
mod network;
mod block;
mod chain_tips;
mod network_totals; 
mod network_peers;
mod mempool_distro;
mod transaction;
mod initial_mempool_distro;

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::BlockInfo;
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::chaintips_info::ChainTip;
use crate::models::network_totals::NetTotals; 
use crate::models::peer_info::PeerInfo;
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    blockchain::fetch_blockchain_info(config).await
}

pub async fn fetch_mempool_info(config: &RpcConfig,  tx: mpsc::Sender<Vec<String>>,) -> Result<MempoolInfo, MyError> {
    mempool::fetch_mempool_info(config, tx).await
}

pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    network::fetch_network_info(config).await
}

pub async fn fetch_block_data_by_height(
    config: &RpcConfig,
    blocks: u64,
    mode: u16, // 1 = Epoch Start Block, 2 = 24 Hours Ago Block
) -> Result<BlockInfo, MyError> {
    block::fetch_block_data_by_height(config, blocks, mode).await
}

pub async fn fetch_chain_tips(config: &RpcConfig) -> Result<Vec<ChainTip>, MyError> {
    chain_tips::fetch_chain_tips(config).await
}

pub async fn fetch_net_totals(config: &RpcConfig) -> Result<NetTotals, MyError> {
    network_totals::fetch_net_totals(config).await
}

pub async fn fetch_peer_info(config: &RpcConfig) -> Result<Vec<PeerInfo>, MyError> {
    network_peers::fetch_peer_info(config).await
}

pub async fn fetch_mempool_distribution(
    config: &RpcConfig, 
    initial_load_complete: Arc<AtomicBool>,
) -> Result<(), MyError> {
    mempool_distro::fetch_mempool_distribution(config, initial_load_complete).await
}

pub async fn initial_mempool_load(
    config: &RpcConfig, 
) -> Result<(), MyError> {
    initial_mempool_distro::initial_mempool_load(config).await
}

pub async fn fetch_transaction(config: &RpcConfig, txid: &str) -> Result<String, MyError> {
    transaction::fetch_transaction(config, txid).await
}
