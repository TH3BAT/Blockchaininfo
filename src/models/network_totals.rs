
//models/network_totals.rs

use serde::{Deserialize, Serialize};

// Wrapper for the full JSON response.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NetTotalsJsonWrap {
    pub result: NetTotals,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NetTotals {
    pub totalbytesrecv: u64,
    pub totalbytessent: u64,
    pub timemillis: u64,
    pub uploadtarget: UploadTarget,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UploadTarget {
    pub timeframe: u64,
    pub target: u64,
    pub target_reached: bool,
    pub serve_historical_blocks: bool,
    pub bytes_left_in_cycle: u64,
    pub time_left_in_cycle: u64,
}


