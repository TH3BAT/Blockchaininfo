//! Loads and manages RPC configuration for BlockchainInfo.
//!
//! This module supports **three tiers of configuration resolution**:
//!
//! 1️⃣ **Command-line argument**:  
//!     `--config <path>`  
//!
//! 2️⃣ **Environment variable**:  
//!     `BLOCKCHAININFO_CONFIG=/path/to/config.toml`  
//!
//! 3️⃣ **Default location**:  
//!     `./target/release/config.toml`  
//!
//! If no file exists at the resolved location, the loader will:
//! - Attempt to read credentials from `RPC_USER`, `RPC_PASSWORD`, `RPC_ADDRESS`  
//! - If missing, interactively prompt the user  
//! - Optionally auto-generate a `config.toml` for future runs  
//!
//! This hybrid strategy allows the dashboard to run **non-interactively** (ideal for systemd)
//! or **interactively** (ideal for first-time local users).

use std::fs;
use std::env;
use std::path::Path;
use std::io::{self, IsTerminal};
use crate::models::errors::MyError;
use crate::utils::get_rpc_password_from_keychain;

use serde::{Deserialize, Serialize};

/// RPC connection configuration for Bitcoin Core.
///
/// ### Fields
/// - `username` — RPC user  
/// - `password` — RPC password (may be loaded from Keychain)  
/// - `address` — RPC endpoint such as `http://127.0.0.1:8332`  
///
/// This struct can be loaded from TOML or constructed interactively.
/// `Serialize` is implemented so that missing config files can be
/// generated on-the-fly.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RpcConfig {
    pub username: String,
    pub password: String,
    pub address: String,
}

impl RpcConfig {
    /// Attempts to fetch the RPC password securely from macOS Keychain.
    ///
    /// This allows the user to avoid storing credentials on disk.
    /// Falls back to prompts or environment variables when unavailable.
    fn get_rpc_password_from_keychain() -> Result<String, MyError> {
        get_rpc_password_from_keychain()
    }
}

/// Determine the path to a config file based on:
/// 1. `--config` CLI argument  
/// 2. `BLOCKCHAININFO_CONFIG` environment variable  
/// 3. Default fallback location  
///
/// This resolution order mirrors typical Unix tool behavior and makes
/// the dashboard easy to embed in automated systems.
fn get_config_path() -> String {
    // --- 1. CLI argument: --config <path> ---
    let args: Vec<String> = env::args().collect();
    if let Some(pos) = args.iter().position(|arg| arg == "--config") {
        if let Some(config_path) = args.get(pos + 1) {
            return config_path.clone();
        }
    }

    // --- 2. Environment variable ---
    if let Ok(env_path) = env::var("BLOCKCHAININFO_CONFIG") {
        return env_path;
    }

    // --- 3. Default location ---
    "./target/release/config.toml".to_string()
}

/// Load RPC configuration from TOML, environment variables, or user input.
///
/// ### Behavior Summary
///
/// If a config file exists at the chosen path:
/// - Parse TOML → produce `RpcConfig`
///
/// Else:
/// - Attempt to load values from ENV:
///     - `RPC_USER`  
///     - `RPC_PASSWORD`  
///     - `RPC_ADDRESS`  
/// - Missing env variables trigger interactive prompts  
/// - A valid config is constructed from user input  
///
/// If **no ENV vars are set**, a fresh `config.toml` is generated automatically.
/// This is ideal for first-run UX while keeping CI/CD non-interactive.
///
/// ### Errors
/// - File read errors  
/// - TOML deserialization errors  
/// - Missing required fields  
pub fn load_config() -> Result<RpcConfig, MyError> {
    let file_path = get_config_path();

    // --- Path 1: Load existing config file ---
    let config: RpcConfig = if Path::new(&file_path).exists() {
        let config_str = fs::read_to_string(file_path)?;
        toml::from_str(&config_str)?
    } else {
        // --- Path 2: No config found → fallback to ENV or interactive prompts ---

        // RPC username
        let username = env::var("RPC_USER").unwrap_or_else(|_| {
            print!("Enter RPC Username: ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });

        // RPC password (ENV → Keychain → prompt)
        let password = resolve_rpc_password()?;

        // RPC address
        let address = env::var("RPC_ADDRESS").unwrap_or_else(|_| {
            print!("Enter RPC Address (e.g., http://127.0.0.1:8332): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        });

        let config = RpcConfig { username, password, address };

        // Auto-save config.toml only when NO env variables were set.
        if env::var("RPC_USER").is_err()
            && env::var("RPC_PASSWORD").is_err()
            && env::var("RPC_ADDRESS").is_err()
        {
            if let Ok(toml_string) = toml::to_string_pretty(&config) {
                let full_toml = format!("[bitcoin_rpc]\n{}", toml_string);
                fs::write(&file_path, full_toml)?;
                println!("✅ Config saved to `{}`", file_path);
            }
        }

        config
    };

    Ok(config)
}

fn resolve_rpc_password() -> Result<String, MyError> {
    // 1) ENV
    if let Ok(p) = std::env::var("RPC_PASSWORD") {
        let p = p.trim().to_string();
        if p.is_empty() {
            return Err(MyError::Config("RPC_PASSWORD is empty".into()));
        }
        return Ok(p);
    }

    // 2) Keychain / pass (macOS/Linux)
    match RpcConfig::get_rpc_password_from_keychain() {
        Ok(p) => {
            let p = p.trim().to_string();
            if p.is_empty() {
                Err(MyError::Keychain("Password retrieved but empty".into()))
            } else {
                Ok(p)
            }
        }
        Err(e) => {
            // 3) Optional interactive fallback only when stdin is a real terminal
            if io::stdin().is_terminal() {
                eprintln!("RPC password lookup failed: {e}");
                eprint!("Enter RPC Password: ");
                let p = rpassword::read_password()
                    .map_err(|_| MyError::Config("Failed to read RPC password".into()))?;

                let p = p.trim().to_string();
                if p.is_empty() {
                    return Err(MyError::Config("RPC password cannot be empty".into()));
                }
                if p.is_empty() {
                    Err(MyError::Config("RPC password cannot be empty".into()))
                } else {
                    Ok(p)
                }
            } else {
                // Non-interactive (TUI/service): fail fast instead of hanging
                Err(e)
            }
        }
    }
}