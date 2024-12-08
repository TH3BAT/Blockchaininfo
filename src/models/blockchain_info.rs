//
// models/blockchain_info.rs
//
use serde::{Deserialize, Serialize};  // For serializing and deserializing structures
use chrono::{TimeZone, Utc};          // For handling and formatting timestamps
use crate::models::errors::MyError;   // Custom error type from the errors module

// Data structure to deserialize blockchain information from the RPC response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockchainInfo {
    pub error: Option<String>,        // Optional field for any error message
    pub id: String,                   // Request ID
    pub result: BlockchainResult,     // The actual blockchain information
}

// Nested structure containing detailed blockchain information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockchainResult {
    pub bestblockhash: String,        // Hash of the best block
    pub blocks: u64,                  // Total number of blocks in the chain
    pub chain: String,                // Chain type (e.g., "main", "test", "regtest")
    pub chainwork: String,            // Total amount of work on the chain
    pub difficulty: f64,              // Current mining difficulty
    pub headers: u64,                 // Number of block headers
    pub initialblockdownload: bool,   // Indicates if the node is downloading blocks
    pub mediantime: u64,              // Median block time in UNIX timestamp format
    pub pruned: bool,                 // Whether the node uses a pruned blockchain
    pub size_on_disk: u64,            // Disk space used by the blockchain
    pub time: u64,                    // Current block time in UNIX timestamp
    pub verificationprogress: f64,    // Progress of chain verification
    pub warnings: String,             // Warnings related to the chain
}

impl BlockchainResult {
    // Converts the chainwork from a hexadecimal string to bits
    pub fn formatted_chainwork_bits(&self) -> Result<String, MyError> {
        u128::from_str_radix(&self.chainwork, 16)
        .map_or_else(
            |_| Err(MyError::InvalidChainworkHexString(self.chainwork.clone())), 
            |decimal_chainwork| {
                let bits = (decimal_chainwork as f64).log2();
                Ok(format!("{:.2} bits", bits))
            }
        )
    }

    pub fn format_scientific(value: f64) -> Result<String, MyError> {
        if value == 0.0 {
            return Ok("0.0".to_string()); // Handle zero separately
        }

        let mut exponent = 0;
        let mut scaled_value = value;

        // Scale the value to get 1–3 digits before the decimal
        while scaled_value.abs() >= 1000.0 {
            scaled_value /= 10.0;
            exponent += 1;
        }
        while scaled_value.abs() < 100.0 {
            scaled_value *= 10.0;
            exponent -= 1;
        }

        // Adjust the exponent to represent groups of 3 (10^3, 10^6, etc.)
        let group_exponent = exponent / 3 * 3; // Groups of 3 for scientific notation
        scaled_value *= 10_f64.powi(exponent % 3); // Adjust the scaled_value accordingly

        // Precompute the superscript exponent using a map
        let superscript_map = [
            '⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹', '⁻',
        ];

        // Check if the exponent is within the valid range for the superscript map
        let exponent_str = group_exponent.to_string();
        let superscript_exponent: String = exponent_str.chars().filter_map(|c| 
            c.to_digit(10).map(|d| superscript_map[d as usize])).collect();

        if superscript_exponent.is_empty() {
            return Err(MyError::from_custom_error("Exponent out of range for superscript formatting".to_string()));
        }

        // Return formatted scientific notation with "×" symbol
        Ok(format!("{:.1}×10{}", scaled_value, superscript_exponent))
    }

    // Format the `difficulty` field as a scientific notation string
    pub fn formatted_difficulty(&self) -> Result<String, MyError> {
        BlockchainResult::format_scientific(self.difficulty)
    }


    // Parse and format UNIX timestamps into Datetime
    pub fn parse_mediantime(&self) -> Result<String, MyError> {
        Utc.timestamp_opt(self.mediantime as i64, 0)
        .single()
        .map_or_else(
            || Err(MyError::InvalidMedianTime(self.mediantime)),
            |t| Ok(t.to_string())
        )
    }

    pub fn parse_time(&self) -> Result<String, MyError> {
        Utc.timestamp_opt(self.time as i64, 0)
        .single()
        .map_or_else(
            || Err(MyError::InvalidBlockTime(self.time)),
            |t| Ok(t.to_string())
        )
    }
   
   // Calculate time since last block was produced.
    pub fn calculate_time_diff(&self) -> Result<String, MyError> {
        let current_time = Utc::now();
        Utc.timestamp_opt(self.time as i64, 0)
        .single()
        .map_or_else(
            || Err(MyError::InvalidBlockTime(self.time)),
            |block_time| {
                let duration = current_time.signed_duration_since(block_time);
                Ok(format!(
                    "{} hours, {} minutes ago", 
                    duration.num_hours(), 
                    duration.num_minutes() % 60
                ))
            }
        )
    }
}
