// lib.rs

// Publicly expose the functions at the crate root
/// This crate stores all the data structures and implementations for JSON returned data.
pub mod models;
/// This crate stores globally related variabes and helper functions.
pub mod utils;
/// This crate stores all the data structures and implementations for app configuration (e.g. RPC).
pub mod config;
/// This crate stores all the RPC function calls to Bitcoin node.
pub mod rpc;
/// This crate stores all  the functions to display the TUI dashboard.
pub mod display;
