//
// rpc.rs
//
pub mod blockchain;
pub mod mempool;
pub mod network;

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::errors::RpcConfig;
use crate::models::errors::MyError;

pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    blockchain::fetch_blockchain_info(config).await
}

pub async fn fetch_mempool_info(config: &RpcConfig) -> Result<MempoolInfo, MyError> {
    mempool::fetch_mempool_info(config).await
}

pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    network::fetch_network_info(config).await
}

