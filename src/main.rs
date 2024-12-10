//
// main.rs
//
mod config;
mod rpc;
mod models;
mod utils;
mod display;
mod alarm;

use config::load_config;
use alarm::check_and_activate_alarm;
use rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info};
use models::errors::MyError;
use display::{display_blockchain_info, display_mempool_info, display_network_info};

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
    
    // Fetch initial data.
    let (blockchain_info, mempool_info, network_info) = tokio::try_join!(
        fetch_blockchain_info(&config.bitcoin_rpc),
        fetch_mempool_info(&config.bitcoin_rpc),
        fetch_network_info(&config.bitcoin_rpc)
    )?;

    // Display display.
    display_blockchain_info(&blockchain_info)?;
    display_mempool_info(&mempool_info)?;
    display_network_info(&network_info)?;

    // Check if we should activate the alarm and activate it.
    check_and_activate_alarm(blockchain_info.blocks, &config).await?;

    Ok(())
}





