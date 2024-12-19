
// rpc.rs

mod blockchain;
mod mempool;
mod network;
mod block;

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::BlockInfo;
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::errors::MyError;
use crate::config::RpcConfig;

pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    blockchain::fetch_blockchain_info(config).await
}

pub async fn fetch_mempool_info(config: &RpcConfig) -> Result<MempoolInfo, MyError> {
    mempool::fetch_mempool_info(config).await
}

pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    network::fetch_network_info(config).await
}

pub async fn fetch_block_data_by_height(config: &RpcConfig, block_height: u64) -> Result<BlockInfo, MyError> {
    block::fetch_block_data_by_height(config, block_height).await
}


