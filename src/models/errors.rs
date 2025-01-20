
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
    Toml(toml::de::Error),
    Keychain(String),
    Config(String),
    InvalidChainworkHexString(String),
    InvalidMedianTime(u64),
    InvalidBlockTime(u64),   
    InvalidBlockHeight(u64), // Difficulty adjustment calc error.
    CustomError(String),     // Format scientific superscript error.
    // Terminal(String),        // Terminal-related errors.
    // Audio(String),           // Audio-related errors.
}

// Implementation of `fmt::Display` for custom error messages.
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Reqwest(err) => write!(f, "Request error: {}", err),
            MyError::SerdeJson(err) => write!(f, "JSON parsing error: {}", err),
            MyError::Io(err) => write!(f, "I/O error: {}", err),
            MyError::Toml(err) => write!(f, "TOML parsing error: {}", err),
            MyError::Keychain(err) => write!(f, "Keychain error: {}", err),
            MyError::Config(err) => write!(f, "Configuration error: {}", err),
            MyError::InvalidChainworkHexString(err) => write!(f, 
                "Invalid chainwork hex string: {}", err),
            MyError::InvalidMedianTime(time) => write!(f, "Invalid median time: {}", time),
            MyError::InvalidBlockTime(time) => write!(f, "Invalid block time: {}", time),
            MyError::InvalidBlockHeight(time) => write!(f, "Invalid block height: {}", time),
            MyError::CustomError(err) => write!(f, "Custom error: {}", err),
            // MyError::Terminal(err) => write!(f, "Terminal error: {}", err),
            // MyError::Audio(err) => write!(f, "Audio error: {}", err),
        }
    }
}

// Automatic conversion between error types.
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

impl From<toml::de::Error> for MyError {
    fn from(err: toml::de::Error) -> MyError {
        MyError::Toml(err)
    }
}

impl From<String> for MyError {
    fn from(err: String) -> MyError {
        MyError::Keychain(err)
    }
}

// Add a method to convert String into CustomError.
impl MyError {
    pub fn from_custom_error(err: String) -> MyError {
        MyError::CustomError(err)
    }
}


impl From<std::env::VarError> for MyError {
    fn from(err: std::env::VarError) -> MyError {
        MyError::Config(format!("Environment variable error: {}", err))
    }
}

/*
impl From<rodio::decoder::DecoderError> for MyError {
    fn from(err: rodio::decoder::DecoderError) -> MyError {
        MyError::Audio(format!("Audio decoder error: {}", err))
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, rodio::Sink>>> for MyError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, rodio::Sink>>) -> MyError {
        MyError::Audio("Mutex poisoned while accessing Sink".to_string())
    }
}
*/

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
    InvalidMedianTime(u64),
    InvalidBlockTime(u64),
    InvalidBlockHeight(u64),
}

// Implement From<MyU64Error> for MyError.
impl From<MyU64Error> for MyError {
    fn from(err: MyU64Error) -> MyError {
        match err {
            MyU64Error::InvalidMedianTime(err) => MyError::InvalidMedianTime(err),
            MyU64Error::InvalidBlockTime(err) => MyError::InvalidBlockTime(err),
            MyU64Error::InvalidBlockHeight(err) => MyError::InvalidBlockHeight(err),
        }
    }
}
