
// main.rs

mod config;
mod rpc;
mod models;
mod utils;
mod display;
mod runapp;

use config::load_config;

use models::errors::MyError;
use runapp::{setup_terminal, cleanup_terminal, run_app};


#[tokio::main]
async fn main() -> Result<(), MyError> {
    // Parse and load RPC configuration or environment variables to connect to node.
    let config_file = "config.toml";
    let config = load_config(config_file)?;

    if config.bitcoin_rpc.username.is_empty()
        || config.bitcoin_rpc.password.is_empty()
        || config.bitcoin_rpc.address.is_empty()
    {
        return Err(MyError::Config("Invalid config data".to_string()));
    }

    // Setup terminal in TUI mode.
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, &config).await;

    // Clean up terminal.
    cleanup_terminal(&mut terminal)?;

    result
}

