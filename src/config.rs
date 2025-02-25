
// config.rs

use std::fs;
use std::env;
use std::path::Path;
use crate::models::errors::MyError;
use crate::utils::get_rpc_password_from_keychain;
use serde::{Deserialize, Serialize}; // Added `Serialize` for writing config to TOML

// Structure for RPC connection details.
#[derive(Debug, Deserialize, Serialize, Clone)] // `Serialize` added for writing to TOML
#[serde(rename_all = "snake_case")]
pub struct RpcConfig {
    pub username: String, 
    pub password: String, 
    pub address: String,  
}

impl RpcConfig {
    /// Fetches the RPC password from the keychain (if necessary).
    pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
        get_rpc_password_from_keychain() // Ensuring Keychain is used
    }
}

// Determines config path from CLI args, env variable, or default location.
pub fn get_config_path() -> String {
    // Check CLI Args (`--config` flag)
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

    // Fallback to Default Path
    "./target/release/config.toml".to_string()
}

// Loads the configuration from a TOML file or environment variables.
pub fn load_config() -> Result<RpcConfig, MyError> {
    let file_path = get_config_path(); // Get config location dynamically

    // Check if the config file exists
    let config: RpcConfig = if Path::new(&file_path).exists() {
        // Load config from file
        let config_str = fs::read_to_string(file_path)?;
        toml::from_str(&config_str)?
    } else {
        // No Config Found? Fall back to ENV or Prompt for User Input
        let username = env::var("RPC_USER").unwrap_or_else(|_| {
            print!("Enter RPC Username: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });

        let password = env::var("RPC_PASSWORD")
            .or_else(|_| RpcConfig::get_rpc_password_from_keychain()) 
            .unwrap_or_else(|_| {
                print!("Enter RPC Password: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                input.trim().to_string()
            });

        let address = env::var("RPC_ADDRESS").unwrap_or_else(|_| {
            print!("Enter RPC Address (e.g., http://127.0.0.1:8332): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });

        // Create a valid config
        let config = RpcConfig { username, password, address };

        // ✨ Auto-generate `config.toml` ONLY if no environment variables are set
        if env::var("RPC_USER").is_err()
            && env::var("RPC_PASSWORD").is_err()
            && env::var("RPC_ADDRESS").is_err()
        {
            if let Ok(toml_string) = toml::to_string_pretty(&config) {
                fs::write(&file_path, toml_string)?;
                println!("✅ Config saved to `{}`", file_path);
            }
        }

        config // Return the generated config
    };

    Ok(config)
}
