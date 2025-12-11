//! Unified application error model.
//!
//! BlockchainInfo interacts with several subsystems:
//! - Bitcoin Core RPC (network I/O + JSON parsing),
//! - filesystem (log rotation, miner files),
//! - configuration loading (TOML, env vars),
//! - async task management,
//! - OS-specific keychain access.
//!
//! Instead of scattering error types throughout the codebase, this module
//! consolidates them into a single `MyError` enum. This keeps all failures
//! visible, traceable, and consistent across the project.
//!
//! Conversions (`From<T> for MyError`) make it easy to bubble errors upward
//! using the `?` operator, avoiding repetitive boilerplate and allowing each
//! module to focus on its own logic.

use reqwest;
use serde_json;
use toml;
use std::io;
use std::fmt;
use tokio::sync::AcquireError;

/// The central error type for BlockchainInfo.
///
/// Each variant represents a different failure domain that may occur across
/// the application.  
///
/// Key goals:  
/// - unify RPC, parsing, file, and async errors,  
/// - simplify propagation with `?`,  
/// - ensure user-facing logs remain clear and structured.
///
/// Variants containing `String` allow detailed context messages to pass through
/// without losing information.
#[derive(Debug)]
#[allow(dead_code)]
pub enum MyError {
    /// Network or HTTP-layer failures from `reqwest`.
    Reqwest(reqwest::Error),

    /// JSON parsing/decoding errors.
    SerdeJson(serde_json::Error),

    /// Standard I/O errors (read/write failures, permissions, missing files).
    Io(io::Error),

    /// TOML parsing errors (loading configuration).
    TomlDeserialize(toml::de::Error),

    /// TOML serialization errors (writing configuration files).
    TomlSerialize(toml::ser::Error),

    /// macOS Keychain or OS keyring-related failures.
    Keychain(String),

    /// Application configuration errors (bad values, missing fields, env var issues).
    Config(String),

    /// Chainwork hex parsing failure.
    InvalidChainworkHexString(String),

    /// Timestamp interpretation errors.
    InvalidMedianTime(u64),
    InvalidBlockTime(u64),

    /// Invalid or unexpected block height.
    InvalidBlockHeight(u64),

    /// General-purpose error with context.
    CustomError(String),

    /// RPC call failed for a specific transaction ID.
    RpcRequestError(String, String),

    /// JSON parsing failure for a transaction.
    JsonParsingError(String, String),

    /// Async task join failure.
    Join(tokio::task::JoinError),

    /// Semaphore acquisition error (lock contention, cancellation).
    SemaphoreError(String),

    /// Timeout error (often from RPC).
    TimeoutError(String),

    /// Generic file read/write error.
    FileError(String),

    /// File was not found (special case for clarity).
    FileNotFound(String),
}

// -----------------------------------------------------------------------------
// Automatic conversions for ergonomic `?` usage
// -----------------------------------------------------------------------------

impl From<tokio::task::JoinError> for MyError {
    fn from(err: tokio::task::JoinError) -> Self {
        MyError::Join(err)
    }
}

impl From<AcquireError> for MyError {
    fn from(err: AcquireError) -> MyError {
        MyError::SemaphoreError(format!("Semaphore acquisition failed: {:?}", err))
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Reqwest(err) => write!(f, "Request error: {}", err),
            MyError::SerdeJson(err) => write!(f, "JSON parsing error: {}", err),
            MyError::Io(err) => write!(f, "I/O error: {}", err),
            MyError::TomlDeserialize(err) => write!(f, "TOML deserialization error: {}", err),
            MyError::TomlSerialize(err) => write!(f, "TOML serialization error: {}", err),
            MyError::Keychain(err) => write!(f, "Keychain error: {}", err),
            MyError::Config(err) => write!(f, "Configuration error: {}", err),
            MyError::InvalidChainworkHexString(s) => write!(f, "Invalid chainwork hex string: {}", s),
            MyError::InvalidMedianTime(t) => write!(f, "Invalid median time: {}", t),
            MyError::InvalidBlockTime(t) => write!(f, "Invalid block time: {}", t),
            MyError::InvalidBlockHeight(h) => write!(f, "Invalid block height: {}", h),
            MyError::CustomError(msg) => write!(f, "Error: {}", msg),
            MyError::RpcRequestError(tx, err) => write!(f, "RPC request failed for TX {}: {}", tx, err),
            MyError::JsonParsingError(tx, err) => write!(f, "TX {}: JSON parsing error: {}", tx, err),
            MyError::Join(err) => write!(f, "Task join error: {}", err),
            MyError::SemaphoreError(err) => write!(f, "Semaphore error: {}", err),
            MyError::TimeoutError(msg) => write!(f, "Error: {}", msg),
            MyError::FileError(msg) => write!(f, "File Error: {}", msg),
            MyError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
        }
    }
}

impl From<toml::ser::Error> for MyError {
    fn from(err: toml::ser::Error) -> MyError {
        MyError::TomlSerialize(err)
    }
}

impl From<toml::de::Error> for MyError {
    fn from(err: toml::de::Error) -> MyError {
        MyError::TomlDeserialize(err)
    }
}

impl From<String> for MyError {
    fn from(err: String) -> MyError {
        MyError::CustomError(err)
    }
}

impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            MyError::TimeoutError("Request timed out".into())
        } else {
            MyError::Reqwest(err)
        }
    }
}

impl From<serde_json::Error> for MyError {
    fn from(err: serde_json::Error) -> MyError {
        MyError::SerdeJson(err)
    }
}

impl From<io::Error> for MyError {
    fn from(err: io::Error) -> MyError {
        MyError::Io(err)
    }
}

/// Convenience constructor for manually raised user-facing errors.
impl MyError {
    pub fn from_custom_error(err: String) -> MyError {
        MyError::CustomError(err)
    }
}

/// Convert environment variable errors into configuration failures.
impl From<std::env::VarError> for MyError {
    fn from(err: std::env::VarError) -> MyError {
        MyError::Config(format!("Environment variable error: {}", err))
    }
}

// -----------------------------------------------------------------------------
// Optional typed sub-errors (string-based and u64-based categories)
// -----------------------------------------------------------------------------

#[derive(Debug)]
#[allow(dead_code)]
pub enum MyStringError {
    Keychain(String),
    InvalidChainworkHexString(String),
}

impl From<MyStringError> for MyError {
    fn from(err: MyStringError) -> MyError {
        match err {
            MyStringError::Keychain(e) => MyError::Keychain(e),
            MyStringError::InvalidChainworkHexString(e) => MyError::InvalidChainworkHexString(e),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MyU64Error {
    MedianTime(u64),
    BlockTime(u64),
    BlockHeight(u64),
}

impl From<MyU64Error> for MyError {
    fn from(err: MyU64Error) -> MyError {
        match err {
            MyU64Error::MedianTime(t) => MyError::InvalidMedianTime(t),
            MyU64Error::BlockTime(t) => MyError::InvalidBlockTime(t),
            MyU64Error::BlockHeight(h) => MyError::InvalidBlockHeight(h),
        }
    }
}
