
// models/network_totals.rs

use serde::Deserialize;

/// Wrapper Struct - The Bitcoin RPC response wraps the actual getnettotals data inside the result field.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetTotalsJsonWrap {
    pub error: Option<String>,    // Optional for any error message.
    pub id: Option<String>,       // Optional Request ID.
    pub result: NetTotals,
}

/// This struct holds data from getnettotals RPC method.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetTotals {
    pub totalbytesrecv: u64,    // Total bytes received across all connections.
    pub totalbytessent: u64,    // Total bytes sent across all connections.
    pub timemillis: u64,        // Current timestamp in milliseconds.
    pub uploadtarget: UploadTarget, // Upload target information and statistics.
}

/// This struct holds UploadTarget data from NetTotals struct.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct UploadTarget {
    pub timeframe: u64,               // Timeframe for the upload target in seconds.
    pub target: u64,                  // Maximum upload target in bytes.
    pub target_reached: bool,         // Whether the upload target has been reached.
    pub serve_historical_blocks: bool, // Whether the node serves historical blocks.
    pub bytes_left_in_cycle: u64,     // Bytes remaining in the current upload cycle.
    pub time_left_in_cycle: u64,      // Time remaining in the current upload cycle in seconds.
}



