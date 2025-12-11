//! Data models for Bitcoin Core's `getnetworkinfo` RPC.
//!
//! These structures provide insight into the node’s identity, capabilities,
//! supported services, connection limitations, relay policies, and warnings.
//!
//! BlockchainInfo uses this data to display:
//! - node software version & subversion,
//! - active connections,
//! - supported local services,
//! - relay fee policies,
//! - network reachability state,
//! - Core-issued warnings.
//!
//! This module intentionally mirrors Core’s RPC format without modifying values.

use serde::Deserialize;

/// Wrapper for the `getnetworkinfo` RPC response.
///
/// Core always returns the actual payload inside `result`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetworkInfoJsonWrap {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: NetworkInfo,
}

/// Main `getnetworkinfo` structure.
///
/// These fields describe the node’s network identity, supported protocol
/// features, fee relay configuration, and high-level connection counts.
#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetworkInfo {
    /// Binary version number of the node (e.g., `270000`).
    pub version: u32,

    /// User-agent string (e.g. `/Satoshi:27.0.0/`).
    pub subversion: String,

    /// P2P protocol version in use.
    pub protocolversion: u32,

    /// Bitfield representing service flags (in hex).
    pub localservices: String,

    /// Human-readable names of services offered by this node.
    /// Example: ["NETWORK", "WITNESS", "COMPACT_FILTERS"]
    pub localservicesnames: Vec<String>,

    /// Whether this node relays transactions to peers.
    pub localrelay: bool,

    /// Local clock offset relative to UTC (in seconds).
    pub timeoffset: i32,

    /// Whether the network is enabled for outbound connections.
    pub networkactive: bool,

    /// Total active P2P connections.
    pub connections: u32,

    /// Inbound-only connections (peers → us).
    pub connections_in: u32,

    /// Outbound-only connections (us → peers).
    pub connections_out: u32,

    /// Detailed view of reachability for each network type (IPv4/IPv6/Onion/etc.).
    pub networks: Vec<Network>,

    /// Relay fee (BTC/kB). Core will not relay transactions below this value.
    pub relayfee: f64,

    /// Minimum incremental fee (BTC/kB) used in fee bumping logic.
    pub incrementalfee: f64,

    /// Local addresses this node advertises to peers.
    pub localaddresses: Vec<LocalAddress>,

    /// Core warnings (issues, alerts, or conditions requiring attention).
    pub warnings: String,
}

/// Metadata about a particular address family (IPv4/IPv6/i2p/onion).
///
/// Mirrors Core’s `networks` array from `getnetworkinfo`.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct Network {
    /// Name of the network type (`ipv4`, `ipv6`, `onion`, etc.).
    pub name: String,

    /// Whether connections on this network type are limited.
    pub limited: bool,

    /// Whether this network type is reachable.
    pub reachable: bool,

    /// Proxy address used for this network type, if any.
    pub proxy: String,

    /// Whether the node randomizes proxy credentials.
    pub proxy_randomize_credentials: bool,
}

/// Local address announced to peers.
///
/// Core includes these when the node has detected or bound to
/// public addresses suitable for advertisement.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct LocalAddress {
    pub address: String,
    pub port: u16,
    pub score: u32,
}
