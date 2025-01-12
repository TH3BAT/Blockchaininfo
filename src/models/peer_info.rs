
// models/peer_info.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Wrapper for JSON-RPC response.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct PeerInfoResponse {
    pub error: Option<String>,    // Optional for any error message.
    pub id: Option<String>,       // Optional Request ID.
    pub result: Vec<PeerInfo>, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct PeerInfo {
    pub id: u64,                           // Unique identifier for the peer.
    pub addr: String,                      // IP address and port of the peer.
    pub addrbind: Option<String>,          // Local address the connection is bound to.
    pub network: Option<String>,           // Network type (e.g., IPv4, IPv6, onion).
    pub services: String,                  // Advertised services offered by the peer.
    pub servicesnames: Option<Vec<String>>, // Human-readable names of the services.
    pub relaytxes: bool,                   // Whether the peer relays transactions.
    pub lastsend: u64,                     // Timestamp of the last data sent to the peer.
    pub lastrecv: u64,                     // Timestamp of the last data received from the peer.
    pub last_transaction: u64,             // Timestamp of the last transaction relay.
    pub last_block: u64,                   // Timestamp of the last block relay.
    pub bytessent: u64,                    // Total bytes sent to the peer.
    pub bytesrecv: u64,                    // Total bytes received from the peer.
    pub conntime: u64,                     // Connection establishment time.
    pub timeoffset: i64,                   // Time offset between the peer and the local node.
    pub pingtime: Option<f64>,             // Last recorded ping time in seconds.
    pub minping: Option<f64>,              // Minimum observed ping time in seconds.
    pub version: i32,                      // Protocol version used by the peer.
    pub subver: String,                    // User agent string of the peer.
    pub inbound: bool,                     // Whether the connection is inbound.
    pub bip152_hb_to: bool,                // Whether this peer sends BIP152 high-bandwidth blocks.
    pub bip152_hb_from: bool,              // Whether this peer receives BIP152 high-bandwidth blocks.
    pub startingheight: i64,               // Peer-reported starting block height.
    pub presynced_headers: i64,            // Number of headers presynced with the peer.
    pub synced_headers: i64,               // Number of headers fully synced with the peer.
    pub synced_blocks: i64,                // Number of blocks fully synced with the peer.
    pub inflight: Option<Vec<u64>>,        // Blocks currently in-flight from the peer.
    pub addr_relay_enabled: bool,          // Whether address relay is enabled for the peer.
    pub addr_processed: i64,               // Number of addresses processed from the peer.
    pub addr_rate_limited: i64,            // Number of addresses rate-limited from the peer
    pub permissions: Option<Vec<String>>,  // Permissions granted to the peer.
    pub minfeefilter: f64,                 // Minimum fee rate accepted by the peer (BTC/kB).
    pub bytessent_per_msg: Option<HashMap<String, u64>>, // Bytes sent per message type.
    pub bytesrecv_per_msg: Option<HashMap<String, u64>>, // Bytes received per message type.
    pub connection_type: Option<String>,   // Type of connection (e.g., outbound, manual).
    pub transport_protocol_type: Option<String>, // Transport protocol type (e.g., TCP, QUIC).
    pub session_id: Option<String>,        // Unique session identifier for the peer.
}


