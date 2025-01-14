
// models/network_totals.rs

use serde::Deserialize;

// Wrapper for the full JSON response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetTotalsJsonWrap {
    pub error: Option<String>,    // Optional for any error message.
    pub id: Option<String>,       // Optional Request ID.
    pub result: NetTotals,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetTotals {
    pub totalbytesrecv: u64,    // Total bytes received across all connections.
    pub totalbytessent: u64,    // Total bytes sent across all connections.
    pub timemillis: u64,        // Current timestamp in milliseconds.
    pub uploadtarget: UploadTarget, // Upload target information and statistics.
}

#[derive(Debug, Deserialize)]
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



