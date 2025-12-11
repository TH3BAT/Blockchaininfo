//! Data models for Bitcoin Core’s `getnettotals` RPC.
//!
//! This endpoint provides high-level byte statistics for the node,
//! including all data sent/received since startup and the state of the
//! upload-target algorithm (if enabled).
//!
//! BlockchainInfo uses these values to show:
//! - total network bandwidth consumption
//! - time-aligned network throughput
//! - upload-target enforcement state
//!
//! These structs intentionally mirror Core’s response exactly.

use serde::Deserialize;

/// Wrapper for the `getnettotals` RPC response.
///
/// Core wraps the actual payload inside the `result` field.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetTotalsJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: NetTotals,
}

/// Represents aggregate network byte counters and upload-target details.
///
/// These counters accumulate for the lifetime of the node process.
/// Restarting the node resets them.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetTotals {
    /// Total bytes received by this node since startup.
    pub totalbytesrecv: u64,

    /// Total bytes sent by this node since startup.
    pub totalbytessent: u64,

    /// Current node system time in milliseconds.
    pub timemillis: u64,

    /// Upload-target state describing bandwidth throttling behavior.
    pub uploadtarget: UploadTarget,
}

/// Information about the node’s upload-target window.
///
/// Upload targets are a mechanism to restrict outbound bandwidth usage
/// over a rolling time window. When enabled (rare in most deployments),
/// Core enforces a maximum byte count for serving block data.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct UploadTarget {
    /// Time window in seconds for the upload target.
    pub timeframe: u64,

    /// Maximum allowed upload volume (bytes) during the window.
    pub target: u64,

    /// Whether the node has reached the target for this cycle.
    pub target_reached: bool,

    /// Whether the node continues serving historical blocks.
    /// “Historical blocks” refers specifically to blocks older than the last ~288 blocks (the default serving 
    /// window). When upload-target throttling activates, these are no longer served.
    pub serve_historical_blocks: bool,

    /// Remaining allowable bytes in the current upload cycle.
    pub bytes_left_in_cycle: u64,

    /// Remaining time (seconds) in the upload-target window.
    pub time_left_in_cycle: u64,
}



