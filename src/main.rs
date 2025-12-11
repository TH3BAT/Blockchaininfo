//! Application entrypoint for the BlockchainInfo TUI.
//!
//! This file is intentionally lightweight — it orchestrates:
//! 1. Loading configuration needed for Bitcoin Core RPC
//! 2. Initializing the terminal in TUI mode
//! 3. Running the application event loop (`run_app`)
//! 4. Cleaning up the terminal on exit
//!
//! All heavy logic is delegated to modules under:
//! - `runapp`    → Core event loop and update cycle
//! - `rpc`       → All Bitcoin Core RPC calls
//! - `display`   → Rendering the TUI components
//! - `models`    → Typed structs for RPC responses
//! - `utils`     → Shared helpers and global caches
//!
//! This ensures `main.rs` stays minimal, predictable, and easy to audit.

mod config;
mod rpc;
mod models;
mod utils;
mod display;
mod runapp;

use config::load_config;
use models::errors::MyError;
use runapp::{setup_terminal, cleanup_terminal, run_app};

/// Tokio async runtime entrypoint.
///
/// ### Flow:
/// 1. **Load RPC configuration**  
///    Reads from TOML, CLI flags, env vars, or prompts the user.
///    Ensures the node address and credentials are valid.
///
/// 2. **Initialize TUI terminal state**  
///    Switches to raw mode and prepares Crossterm for rendering.
///
/// 3. **Run the main application loop**  
///    This continuously:
///    - fetches RPC data
///    - updates global caches
///    - renders TUI frames
///    - handles keyboard input
///
/// 4. **Restore terminal state on exit**  
///    Even in error cases, the terminal is returned to normal mode.
///
/// ### Errors:
/// Returns `MyError` if:
/// - Config is missing or invalid  
/// - Terminal setup fails  
/// - Application loop encounters a fatal error  
#[tokio::main]
async fn main() -> Result<(), MyError> {
    // Load RPC credentials and node address from config/system.
    let config = load_config()?;

    // Validate minimum configuration requirements.
    if config.username.is_empty()
        || config.password.is_empty()
        || config.address.is_empty()
    {
        return Err(MyError::Config("Invalid config data".to_string()));
    }

    // Switch terminal into alternate-screen TUI mode.
    let mut terminal = setup_terminal()?;

    // Run the async update/render loop.
    let result = run_app(&mut terminal, &config).await;

    // Restore terminal to normal mode regardless of success or failure.
    cleanup_terminal(&mut terminal)?;

    result
}


