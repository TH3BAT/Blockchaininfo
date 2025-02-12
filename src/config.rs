
// config.rs

use std::fs;
use std::env;
use std::path::Path;
use crate::models::errors::MyError;                // Custom MyError routines.
use crate::utils::get_rpc_password_from_keychain;  // Custom utility function.
use serde::Deserialize;                            // For deserialization.

// Configuration structure for Bitcoin RPC.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct BitcoinRpcConfig {
    pub bitcoin_rpc: RpcConfig, // Contains username, password, and address.
}

// Structure for RPC connection details.
#[derive(Debug, Deserialize, Clone)]
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

// Determines config path from CLI args, env variable, or default location.
pub fn get_config_path() -> String {
    // 1️Check CLI Args (`--config` flag)
    let args: Vec<String> = env::args().collect();
    if let Some(pos) = args.iter().position(|arg| arg == "--config") {
        if let Some(config_path) = args.get(pos + 1) {
            return config_path.clone(); // Use CLI-provided path
        }
    }

    // Check Environment Variable (`BLOCKCHAININFO_CONFIG`)
    if let Ok(env_path) = env::var("BLOCKCHAININFO_CONFIG") {
        return env_path;
    }

    // Fallback to Default
    "./target/release/config.toml".to_string()
}

// Loads the configuration from a TOML file or environment variables.
pub fn load_config() -> Result<BitcoinRpcConfig, MyError> {
    let file_path = get_config_path(); // Get config location dynamically

    // Attempt to read the config from the TOML file.
    let config: BitcoinRpcConfig = if Path::new(&file_path).exists() {
        let config_str = fs::read_to_string(file_path)?;
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

