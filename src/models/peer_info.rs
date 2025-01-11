
// models/peer_info.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Wrapper for JSON-RPC response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerInfoResponse {
    pub result: Vec<PeerInfo>, // PeerInfo array in the result field
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeerInfo {
    pub id: u64,
    pub addr: String,
    pub addrbind: Option<String>,
    pub network: Option<String>,
    pub services: String,
    pub servicesnames: Option<Vec<String>>,
    pub relaytxes: bool,
    pub lastsend: u64,
    pub lastrecv: u64,
    pub last_transaction: u64,
    pub last_block: u64,
    pub bytessent: u64,
    pub bytesrecv: u64,
    pub conntime: u64,
    pub timeoffset: i64,
    pub pingtime: Option<f64>,
    pub minping: Option<f64>,
    pub version: i32,
    pub subver: String,
    pub inbound: bool,
    pub bip152_hb_to: bool,
    pub bip152_hb_from: bool,
    pub startingheight: i64,
    pub presynced_headers: i64,
    pub synced_headers: i64,
    pub synced_blocks: i64,
    pub inflight: Option<Vec<u64>>,
    pub addr_relay_enabled: bool,
    pub addr_processed: i64,
    pub addr_rate_limited: i64,
    pub permissions: Option<Vec<String>>,
    pub minfeefilter: f64,
    pub bytessent_per_msg: Option<HashMap<String, u64>>,
    pub bytesrecv_per_msg: Option<HashMap<String, u64>>,
    pub connection_type: Option<String>,
    pub transport_protocol_type: Option<String>,
    pub session_id: Option<String>,
}

