
// rpc.rs

mod blockchain;
mod mempool;
mod network;
mod block;
mod chain_tips;
mod network_totals; 
mod network_peers;
mod mempool_distro;

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::BlockInfo;
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::chaintips_info::ChainTip;
use crate::models::network_totals::NetTotals; 
use crate::models::peer_info::PeerInfo;
use crate::models::errors::MyError;
use crate::config::RpcConfig;

pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    blockchain::fetch_blockchain_info(config).await
}

pub async fn fetch_mempool_info(config: &RpcConfig, sample_percentage: f64) -> Result<(MempoolInfo, Vec<String>), MyError> {
    mempool::fetch_mempool_info(config, sample_percentage).await
}

pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    network::fetch_network_info(config).await
}

pub async fn fetch_block_data_by_height(config: &RpcConfig, block_height: u64) -> Result<BlockInfo, MyError> {
    block::fetch_block_data_by_height(config, block_height).await
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
    sample_ids: Vec<String>,
) -> Result<((usize, usize, usize), (usize, usize, usize), (usize, usize), f64, f64), MyError> {
    mempool_distro::fetch_mempool_distribution(config, sample_ids).await
}
