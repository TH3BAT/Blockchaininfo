//! Crate root for BlockchainInfo.
//!
//! This module simply re-exports the major internal modules so the rest of the
//! application (and any external tools) can access them through a unified,
//! well-organized namespace.
//!
//! The project is intentionally structured into clear domains:
//!
//! - **models**  → Typed JSON structures returned by Bitcoin Core RPC  
//! - **utils**   → Shared helpers, formatting utilities, global caches  
//! - **config**  → Loading, validating, and storing RPC configuration  
//! - **rpc**     → Direct interaction layer with Bitcoin Core  
//! - **display** → TUI rendering logic for the dashboard  
//!
//! Keeping `lib.rs` minimal makes the entire crate easier to explore,
//! especially for contributors or developers reading the codebase for the
//! first time.

/// Data structures representing all Bitcoin Core RPC response formats,
/// as well as associated helper implementations.
pub mod models;

/// Global shared caches, helpers, utility functions, and formatting tools.
/// Also includes TUI helpers such as header/footer rendering.
pub mod utils;

/// Loading and validating RPC configuration (TOML, ENV, CLI, Keychain).
pub mod config;

/// The RPC client layer — every Bitcoin Core RPC call lives here.
pub mod rpc;

/// TUI rendering system: tables, charts, panels, interactive views, etc.
pub mod display;
