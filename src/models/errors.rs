
// models/errors.rs

use reqwest;                // For Reqwest errors.
use serde_json;             // For JSON error handling.
use toml;                   // For TOML parsing errors.
use std::io;                // For I/O errors.
use std::fmt;               // For Display trait.

// Custom error enum for handling various types of errors.
#[derive(Debug)]
pub enum MyError {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    Io(io::Error),
    TomlDeserialize(toml::de::Error), // Renamed for clarity
    TomlSerialize(toml::ser::Error),  // NEW: Handles TOML serialization errors
    Keychain(String),
    Config(String),
    InvalidChainworkHexString(String),
    InvalidMedianTime(u64),
    InvalidBlockTime(u64),   
    InvalidBlockHeight(u64), 
    CustomError(String),
    RpcRequestError(String, String),
    JsonParsingError(String, String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Reqwest(err) => write!(f, "Request error: {}", err),
            MyError::SerdeJson(err) => write!(f, "JSON parsing error: {}", err),
            MyError::Io(err) => write!(f, "I/O error: {}", err),
            MyError::TomlDeserialize(err) => write!(f, "TOML deserialization error: {}", err),
            MyError::TomlSerialize(err) => write!(f, "TOML serialization error: {}", err), // ✅ NEW
            MyError::Keychain(err) => write!(f, "Keychain error: {}", err),
            MyError::Config(err) => write!(f, "Configuration error: {}", err),
            MyError::InvalidChainworkHexString(err) => write!(f, "Invalid chainwork hex string: {}", err),
            MyError::InvalidMedianTime(time) => write!(f, "Invalid median time: {}", time),
            MyError::InvalidBlockTime(time) => write!(f, "Invalid block time: {}", time),
            MyError::InvalidBlockHeight(time) => write!(f, "Invalid block height: {}", time),
            MyError::CustomError(err) => write!(f, "Custom error: {}", err),
            MyError::RpcRequestError(tx_id, err) => write!(f, "RPC request failed for TX {}: {}", tx_id, err),
            MyError::JsonParsingError(tx_id, err) => write!(f, "JSON parsing error for TX {}: {}", tx_id, err),
        }
    }
}

// NEW: Handle TOML serialization errors
impl From<toml::ser::Error> for MyError {
    fn from(err: toml::ser::Error) -> MyError {
        MyError::TomlSerialize(err)
    }
}

// Rename TOML deserialization error handler for clarity
impl From<toml::de::Error> for MyError {
    fn from(err: toml::de::Error) -> MyError {
        MyError::TomlDeserialize(err)
    }
}

impl From<String> for MyError {
    fn from(err: String) -> MyError {
        MyError::CustomError(err)  // Default to `CustomError`
    }
}


impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> MyError {
        MyError::Reqwest(err)
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

// Method for converting a string into a `CustomError`
impl MyError {
    pub fn from_custom_error(err: String) -> MyError {
        MyError::CustomError(err)
    }
}

// Add conversions for environment variable errors
impl From<std::env::VarError> for MyError {
    fn from(err: std::env::VarError) -> MyError {
        MyError::Config(format!("Environment variable error: {}", err))
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MyStringError {
    Keychain(String),
    InvalidChainworkHexString(String),
}

// Implement From<MyStringError> for MyError.
impl From<MyStringError> for MyError {
    fn from(err: MyStringError) -> MyError {
        match err {
            MyStringError::Keychain(err) => MyError::Keychain(err),
            MyStringError::InvalidChainworkHexString(err) => MyError::InvalidChainworkHexString(err),
        }
    }
}

// Similarly for u64-based errors.
#[derive(Debug)]
#[allow(dead_code)]
pub enum MyU64Error {
    MedianTime(u64),
    BlockTime(u64),
    BlockHeight(u64),
}

// Implement From<MyU64Error> for MyError.
impl From<MyU64Error> for MyError {
    fn from(err: MyU64Error) -> MyError {
        match err {
            MyU64Error::MedianTime(err) => MyError::InvalidMedianTime(err),
            MyU64Error::BlockTime(err) => MyError::InvalidBlockTime(err),
            MyU64Error::BlockHeight(err) => MyError::InvalidBlockHeight(err),
        }
    }
}
