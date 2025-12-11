//! Top-level data models for BlockchainInfo.
//!
//! This module organizes all structures that mirror Bitcoin Core’s RPC
//! response schema. Each submodule corresponds to a specific RPC method,
//! keeping the data layer transparent, predictable, and easy to maintain.
//!
//! These models stay as close as possible to Core’s JSON format so that:
//! - upstream RPC changes can be identified quickly,
//! - parsing logic remains simple and robust,
//! - contributors can easily navigate Bitcoin Core’s data structures,
//! - higher-level modules (TUI, mempool engine, network view, etc.)
//!   operate on normalized, well-documented types.
//!
//! Helper methods exist only where interpretation is required
//! (e.g., formatting difficulty, extracting miner addresses).

/// Models for `getblockchaininfo` and related helpers.
/// Provides chain-state, difficulty, chainwork, and timestamp utilities.
pub mod blockchain_info;

/// Models for mempool RPC methods:
/// - `getmempoolinfo`
/// - `getrawmempool`
/// - `getmempoolentry`
///
/// Also includes `MempoolDistribution`, used by the analytics engine.
pub mod mempool_info;

/// Models for `getnetworkinfo`, including peer counts, services,
/// relay policy, and node configuration metadata.
pub mod network_info;

/// Custom error type used throughout the application to unify RPC,
/// I/O, parsing, and configuration errors under a single enum.
pub mod errors;

/// Models for block-level RPC calls:
/// - `getblockhash`
/// - `getblock` (verbose 1 & 2)
///
/// Includes miner extraction logic and 24-hour block history structures.
pub mod block_info;

/// Models for `getchaintips`, used to track forks, side-branches,
/// and chain-status metadata from Core.
pub mod chaintips_info;

/// Models for `getnettotals`, including upload/download statistics and
/// bandwidth-cycle data.
pub mod network_totals;

/// Models for `getpeerinfo`, including peer metadata, version parsing,
/// client identification, and propagation-time helpers.
pub mod peer_info;

/// Models for `getrawtransaction` and related output/input structures.
/// Includes OP_RETURN decoding and script classification helpers.
pub mod transaction_info;

/// Data structures powering the flashing visual indicators in the TUI.
/// Tracks changing values (blocks, mempool size, connections, miners)
/// and applies temporary highlight styles.
pub mod flashing_text;

