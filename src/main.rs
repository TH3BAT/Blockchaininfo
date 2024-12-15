
// main.rs

mod config;
mod rpc;
mod models;
mod utils;
mod display;
mod alarm;

// use blockchaininfo::models::{block_info, blockchain_info};
use config::load_config;
use alarm::check_and_activate_alarm;
use rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info, fetch_block_data_by_height};
use models::errors::MyError;
use display::{display_blockchain_info, display_mempool_info, display_network_info};
use crate::utils::DIFFICULTY_ADJUSTMENT_INTERVAL;
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), MyError> {
    // Parse and load RPC configuration or environment variables to connect to node.
    let config_file = "config.toml";
    let config = load_config(config_file)?;

    if config.bitcoin_rpc.username.is_empty()
        || config.bitcoin_rpc.password.is_empty()
        || config.bitcoin_rpc.address.is_empty() {
        return Err(MyError::Config("Invalid config data".to_string()));
    }
    
    // Fetch blockchain info first since `blocks` is needed for the next call.
    let blockchain_info = fetch_blockchain_info(&config.bitcoin_rpc).await?;

    // Extract the block height from BlockchainInfo
    let epoc_start_block = (
        (blockchain_info.blocks - 1) / DIFFICULTY_ADJUSTMENT_INTERVAL) * DIFFICULTY_ADJUSTMENT_INTERVAL;
    
    // Concurrently fetch mempool info, network info, and block info (dependent on block height)
    let (mempool_info, network_info, block_info) = try_join!(
        fetch_mempool_info(&config.bitcoin_rpc),
        fetch_network_info(&config.bitcoin_rpc),
        fetch_block_data_by_height(&config.bitcoin_rpc, epoc_start_block)
    )?;

    // Display display.
    display_blockchain_info(&blockchain_info, &block_info)?;
    display_mempool_info(&mempool_info)?;
    display_network_info(&network_info)?;

    // Check if we should activate the alarm and activate it.
    check_and_activate_alarm(blockchain_info.blocks, &config).await?;

    Ok(())
}





