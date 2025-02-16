
// models/peer_info.rs

use serde::Deserialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Wrapper for JSON-RPC response.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct PeerInfoResponse {
    pub error: Option<String>,    // Optional for any error message.
    pub id: Option<String>,       // Optional Request ID.
    pub result: Vec<PeerInfo>, 
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
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

impl PeerInfo {
    /// Normalize the version from the `subver` field to `major.minor.patch`.
    pub fn normalize_version(subver: &str) -> String {
        let version_pattern = regex::Regex::new(r"/Satoshi:(\d+\.\d+\.\d+)").unwrap();
        if let Some(captures) = version_pattern.captures(subver) {
            captures.get(1).map_or_else(|| "Unknown".to_string(), |m| m.as_str().to_string())
        } else {
            "Unknown".to_string()
        }
    }

    /// Aggregate and sort Node Version Distribution by peer count.
    pub fn aggregate_and_sort_versions(peer_info: &[PeerInfo]) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
    
        // Aggregate peer counts for normalized versions
        for peer in peer_info.iter().filter(|peer| peer.subver.contains("Satoshi")) {
            let normalized_version = PeerInfo::normalize_version(&peer.subver); // Use `normalize_version`.
            *counts.entry(normalized_version).or_insert(0) += 1;
        }
    
        // Convert HashMap to Vec for sorting
        let mut sorted_counts: Vec<(String, usize)> = counts.into_iter().collect();
    
        // Sort: First by count (descending), then by version (numeric comparison)
        sorted_counts.sort_by(|a, b| {
            b.1.cmp(&a.1) // First: Sort by peer count (desc)
                .then_with(|| Self::compare_versions(&a.0, &b.0)) // Second: Sort by version (asc)
        });
    
        sorted_counts
    }
    
    // Version Comparison (parses version numbers correctly)
    fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        let parse_version = |s: &str| {
            s.split('.')
                .map(|part| part.parse::<u32>().unwrap_or(0)) // Parse safely
                .collect::<Vec<u32>>()
        };
    
        let ver_a = parse_version(a);
        let ver_b = parse_version(b);
    
        ver_b.cmp(&ver_a) // Descending order
    }
    
    /// Calculate block propagation time in minutes.
    pub fn calculate_block_propagation_time(
        peer_info: &[PeerInfo],
        best_block_time: u64,
        best_block: u64,
    ) -> i64 {
        let mut propagation_times: Vec<i64> = Vec::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
    
        let best_block_i64 = best_block as i64;
    
        // Iterate over peers to calculate propagation time.
        for peer in peer_info.iter().filter(|peer| {
            peer.subver.contains("Satoshi") &&
            peer.last_block > 0 &&
            peer.last_block <= current_time &&
            peer.synced_blocks == best_block_i64
        }) {
            let peer_last_block_timestamp = peer.last_block as i64;
    
            // Calculate propagation time in milliseconds.
            // Discard where peer's last_block timestamp is 'invalid'. 
            let propagation_time_in_ms = (peer_last_block_timestamp - best_block_time as i64) * 1000;
            if propagation_time_in_ms.abs() <= 600_000 {
                propagation_times.push(propagation_time_in_ms);
            } else {
                continue; // Skip peers with invalid timestamps / bad clocks.
            }
        }
    
        // Calculate the average propagation time.
        let total_peers = propagation_times.len();
        if total_peers == 0 {
            return 0; // Return 0 if no valid peers
        }
    
        let total_time: i64 = propagation_times.iter().sum();
        let average_propagation_time_in_ms = total_time / total_peers as i64;
    
        average_propagation_time_in_ms / 6000 // Return in seconds.
    }

}

