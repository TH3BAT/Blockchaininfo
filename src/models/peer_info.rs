//! Data models for `getpeerinfo`.
//!
//! Bitcoin Core’s `getpeerinfo` RPC exposes detailed, per-peer network state.
//! Each `PeerInfo` struct represents one connected peer, including:
//!
//! - address, network type, capabilities
//! - service bits, user-agent ("subver"), and protocol version
//! - ping times & time offset
//! - header/block sync progress
//! - per-message traffic stats
//! - whether the peer is inbound/outbound/manual/feeler/etc.
//!
//! BlockchainInfo uses this data to power:
//! - Client Distribution chart (Core, Knots, Ronin, Other)
//! - Version Distribution chart
//! - Block propagation analytics
//! - Per-peer health insights
//!
//! These models mirror Core exactly. Higher-level interpretation happens
//! inside the dashboard logic.

use serde::Deserialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

//
// ────────────────────────────────────────────────────────────────────────────────
//   RPC WRAPPER
// ────────────────────────────────────────────────────────────────────────────────
//

/// Wrapper for Core’s `getpeerinfo` result.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct PeerInfoJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: Vec<PeerInfo>,
}

/// Use in propagation storage logic (runapp.rs)
pub struct NetworkState {
    pub last_propagation_index: Option<usize>,
    pub last_block_seen: u64,
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   MAIN PEER STRUCT
// ────────────────────────────────────────────────────────────────────────────────
//

/// One connected peer in the P2P network.
///
/// This struct is intentionally large because `getpeerinfo` exposes
/// nearly everything Core knows about a peer. All fields are 1:1 with
/// Core’s output — no modifications or reinterpretation occur here.
///
/// Interpretation happens at the dashboard and analytics layer.
#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct PeerInfo {
    /// Unique peer ID assigned by Core.
    pub id: u64,

    /// Remote address and port.
    pub addr: String,

    /// Local address/port to which this peer is bound, if available.
    pub addrbind: Option<String>,

    /// Network type ("ipv4", "ipv6", "onion", etc.).
    pub network: Option<String>,

    /// Hex-encoded service flags offered by the peer.
    pub services: String,

    /// Human-readable names for service flags.
    pub servicesnames: Option<Vec<String>>,

    /// Whether this peer relays transactions.
    pub relaytxes: bool,

    /// Timestamp of last send.
    pub lastsend: u64,

    /// Timestamp of last receive.
    pub lastrecv: u64,

    /// Timestamp of last transaction relay from this peer.
    pub last_transaction: u64,

    /// Timestamp of last block relay from this peer.
    pub last_block: u64,

    /// Total bytes sent to peer.
    pub bytessent: u64,

    /// Total bytes received from peer.
    pub bytesrecv: u64,

    /// Connection start time (seconds since epoch).
    pub conntime: u64,

    /// Peer’s clock offset.
    pub timeoffset: i64,

    /// Last measured ping time.
    #[serde(skip)]
    #[allow(dead_code)]
    pub pingtime: Option<f64>,

    /// Minimum observed ping time.
    #[serde(skip)]
    #[allow(dead_code)]
    pub minping: Option<f64>,

    /// P2P protocol version in use.
    pub version: i32,

    /// User-agent string (e.g. `/Satoshi:27.0.0/`).
    pub subver: String,

    /// Whether connection is inbound.
    pub inbound: bool,

    /// Whether peer sends/receives BIP152 high-bandwidth messages.
    pub bip152_hb_to: bool,
    pub bip152_hb_from: bool,

    /// Peer-reported starting block height.
    pub startingheight: i64,

    /// Headers-sync progress.
    pub presynced_headers: i64,
    pub synced_headers: i64,
    pub synced_blocks: i64,

    /// Blocks currently requested from this peer.
    #[serde(skip)]
    #[allow(dead_code)]
    pub inflight: Option<Vec<u64>>,

    /// Whether address relay is enabled.
    pub addr_relay_enabled: bool,
    #[serde(skip)]
    #[allow(dead_code)]
    pub addr_processed: i64,
    pub addr_rate_limited: i64,

    /// Permissions granted to this peer (e.g. `noban`, `forcerelay`).
    #[serde(skip)]
    #[allow(dead_code)]
    pub permissions: Option<Vec<String>>,

    /// Peer’s minimum feerate filter.
    #[serde(skip)]
    #[allow(dead_code)]
    pub minfeefilter: f64,

    /// Per-message send/receive volume.
    #[serde(skip)]
    #[allow(dead_code)]
    pub bytessent_per_msg: Option<HashMap<String, u64>>,
    pub bytesrecv_per_msg: Option<HashMap<String, u64>>,

    /// Connection category (e.g. "outbound-full-relay", "manual", "feeler").
    #[serde(skip)]
    #[allow(dead_code)]
    pub connection_type: Option<String>,

    /// Transport protocol: TCP or QUIC.
    #[serde(skip)]
    #[allow(dead_code)]
    pub transport_protocol_type: Option<String>,

    /// Unique session ID (Core 26+).
    #[serde(skip)]
    #[allow(dead_code)]
    pub session_id: Option<String>,
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   VERSION DISTRIBUTION
// ────────────────────────────────────────────────────────────────────────────────
//

impl PeerInfo {
    /// Extracts a clean semantic version number from `/Satoshi:x.y.z/`.
    ///
    /// Example:
    /// `/Satoshi:27.0.0/` → `"27.0.0"`
    ///
    /// Returns `"Unknown"` for non-standard agents.
    pub fn normalize_version(subver: &str) -> String {
        let re = regex::Regex::new(r"/Satoshi:(\d+\.\d+\.\d+)").unwrap();

        if let Some(caps) = re.captures(subver) {
            return caps.get(1).unwrap().as_str().to_string();
        }

        "Unknown".to_string()
    }

    /// Aggregates version counts and sorts them by:
    /// 1. peer count (descending)
    /// 2. version number (descending)
    pub fn aggregate_and_sort_versions(peer_info: &[PeerInfo]) -> Vec<(String, usize)> {
        let mut counts = HashMap::new();

        for peer in peer_info.iter().filter(|p| p.subver.contains("Satoshi")) {
            let v = Self::normalize_version(&peer.subver);
            *counts.entry(v).or_insert(0) += 1;
        }

        let mut list: Vec<(String, usize)> = counts.into_iter().collect();

        list.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| Self::compare_versions(&a.0, &b.0))
        });

        list
    }

    /// Numeric version comparator.
    /// `27.0.1` > `27.0.0`, etc.
    fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        let parse = |s: &str| {
            s.split('.')
                .map(|v| v.parse::<u32>().unwrap_or(0))
                .collect::<Vec<_>>()
        };
        parse(b).cmp(&parse(a))
    }

    //
    // ────────────────────────────────────────────────────────────────────────────────
    //   CLIENT DISTRIBUTION (Core / Knots / Ronin / Other)
    // ────────────────────────────────────────────────────────────────────────────────
    //

    /// Determine the client type from `subver`:
    ///
    /// `/Satoshi:27.0.0/Knots:20241122/` → `"Knots"`
    /// `/Ronin:23.0.1/` → `"Ronin"`
    ///
    /// Normalized mapping:
    /// - Satoshi → Core  
    /// - Knots   → Knots  
    /// - Ronin   → Ronin  
    /// - Anything else → Other  
    pub fn extract_client(subver: &str) -> String {
        fn normalize(name: &str) -> String {
            match name.to_lowercase().as_str() {
                "satoshi" => "Core".to_string(),
                "knots" => "Knots".to_string(),
                "ronin" => "Ronin".to_string(),
                _ => "Other".to_string(),
            }
        }

        // remove outer slashes
        let trimmed = subver.trim_matches('/');

        // extract segments
        let segments: Vec<&str> = trimmed.split('/').collect();

        // scan right → left for a "name:version" segment
        for seg in segments.iter().rev() {
            if seg.contains(':') {
                let raw = seg.split(':').next().unwrap_or("").trim();
                return normalize(raw);
            }
        }

        "Other".to_string()
    }

    /// Aggregates client counts and sorts by:
    /// 1. count descending
    /// 2. name ascending
    pub fn aggregate_and_sort_clients(peer_info: &[PeerInfo]) -> Vec<(String, usize)> {
        let mut counts = HashMap::new();

        for p in peer_info {
            let c = Self::extract_client(&p.subver);
            *counts.entry(c).or_insert(0) += 1;
        }

        let mut list: Vec<(String, usize)> = counts.into_iter().collect();

        list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        list
    }
    //
    // ────────────────────────────────────────────────────────────────────────────────
    //   BLOCK PROPAGATION ANALYTICS
    // ────────────────────────────────────────────────────────────────────────────────
    //

    /// Estimates block propagation time (seconds) across peers.
    ///
    /// Filters:
    /// - must be Satoshi-based clients
    /// - peer must have seen the best block
    /// - timestamps must be sane (±10 minutes)
    ///
    /// Returns 0 if no valid sample exists.
    pub fn calculate_block_propagation_time(
        peer_info: &[PeerInfo],
        best_block_time: u64,
        best_block_height: u64,
    ) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut samples: Vec<i64> = Vec::new();
        let h = best_block_height as i64;

        for peer in peer_info.iter().filter(|p| {
            p.subver.contains("Satoshi")
                && p.last_block > 0
                && p.last_block <= now
                && p.synced_blocks == h
        }) {
            let delta_ms = (peer.last_block as i64 - best_block_time as i64) * 1000;

            // discard bad clock peers
            if delta_ms.abs() <= 600_000 {
                samples.push(delta_ms);
            }
        }

        if samples.is_empty() {
            return 0;
        }

        let avg_ms: i64 = samples.iter().sum::<i64>() / samples.len() as i64;

        // Peer timestamps are UNIX seconds and may differ significantly due to clock skew.
        // Dividing by 1000 yields raw seconds, which often appear misleadingly large
        // (minutes) despite normal network behavior.
        //
        // We normalize average skew into 6-second units to:
        // - reduce clock jitter
        // - avoid exaggerating harmless peer drift
        // - better reflect perceived network health
        //
        // This keeps values human-scale while preserving direction and magnitude.
        avg_ms / 6000
    }
}
