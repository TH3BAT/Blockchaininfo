
// config.rs

use std::fs;
use std::env;
use toml;
use crate::models::errors::MyError;                // Custom MyError routines.
use crate::utils::get_rpc_password_from_keychain;  // Custom utility function.
use serde::Deserialize;                            // For deserialization.

// Configuration structure for Bitcoin RPC.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BitcoinRpcConfig {
     pub bitcoin_rpc: RpcConfig, // Contains username, password, and address.
}

// Structure for RPC connection details.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RpcConfig {
     pub username: String, // RPC username.
     pub password: String, // RPC password.
     pub address: String,  // RPC server address.
}

impl BitcoinRpcConfig {
    // Fetches the RPC password from the keychain (if necessary).
    pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
        get_rpc_password_from_keychain()
    }
}

// Loads the configuration from a TOML file or environment variables.
pub fn load_config(file_path: &str) -> Result<BitcoinRpcConfig, MyError> {
    // Attempt to read the config from the TOML file.
    let config: BitcoinRpcConfig = if let Ok(config_str) = fs::read_to_string(file_path) {
        // If the config file exists, parse it.
        toml::de::from_str(&config_str)?
    } else {
        // If the file doesn't exist, fall back to reading from environment variables or Keychain.
        let password = env::var("RPC_PASSWORD")
            .or_else(|_| BitcoinRpcConfig::get_rpc_password_from_keychain())?; 

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

