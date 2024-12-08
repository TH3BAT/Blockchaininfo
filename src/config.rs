//
// config.rs
//
use std::fs;
use std::env;
use toml;
use crate::models::errors::{BitcoinRpcConfig, RpcConfig, MyError};


// Loads the configuration from a TOML file or environment variables
pub fn load_config(file_path: &str) -> Result<BitcoinRpcConfig, MyError> {
    // Attempt to read the config from the TOML file
    let config: BitcoinRpcConfig = if let Ok(config_str) = fs::read_to_string(file_path) {
        // If the config file exists, parse it
        toml::de::from_str(&config_str)?
    } else {
        // If the file doesn't exist, fall back to reading from environment variables or Keychain
        let password = env::var("RPC_PASSWORD")
            .or_else(|_| BitcoinRpcConfig::get_rpc_password_from_keychain())?; // Now calling the method under impl BitcoinRpcConfig

        BitcoinRpcConfig {
            bitcoin_rpc: RpcConfig {
                username: env::var("RPC_USER")?,
                password,
                address: env::var("RPC_ADDRESS")?,
            }
        }
    };

    Ok(config)
}

