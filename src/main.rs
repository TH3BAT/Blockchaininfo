//
// main.rs
//
mod config;
mod rpc;
mod models;
mod utils;
mod display; 

use config::load_config;
use rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info};
use models::MyError;
use display::{display_blockchain_info, display_mempool_info, display_network_info};

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let config_file = "config.toml";
    let config = load_config(config_file)?;

    if config.bitcoin_rpc.username.is_empty()
        || config.bitcoin_rpc.password.is_empty()
        || config.bitcoin_rpc.address.is_empty() {
        return Err(MyError::Config("Invalid config data".to_string()));
    }

    let (blockchain_info, mempool_info, network_info) = tokio::try_join!(
        fetch_blockchain_info(&config.bitcoin_rpc),
        fetch_mempool_info(&config.bitcoin_rpc),
        fetch_network_info(&config.bitcoin_rpc)
    )?;

    display_blockchain_info(&blockchain_info)?;
    display_mempool_info(&mempool_info)?;
    display_network_info(&network_info)?;

    Ok(())
}


