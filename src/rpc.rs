//
// rpc.rs
//
pub mod blockchain;
pub mod mempool;
pub mod network;

use crate::models::{BlockchainInfo, MempoolInfo, NetworkInfo, MyError, RpcConfig};

// Use Namespacing
pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    blockchain::fetch_blockchain_info(config).await
}

pub async fn fetch_mempool_info(config: &RpcConfig) -> Result<MempoolInfo, MyError> {
    mempool::fetch_mempool_info(config).await
}

pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {
    network::fetch_network_info(config).await
}

