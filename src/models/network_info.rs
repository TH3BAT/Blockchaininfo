//
// models/network_info.rs
//
use serde::{Deserialize, Serialize}; // For struct serialization and deserialization

// Represents the information returned from the `getnetworkinfo` RPC call.
// This struct represents the root response from the RPC call, which contains the `result` field
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NetworkInfoResponse {
      pub result: NetworkInfo,   // The actual result field which contains the data we need
      pub error: Option<String>, // Optional error field
      pub id: String,            // The ID of the request
}


// This struct encapsulates various details about the Bitcoin node's network status.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NetworkInfo {
      pub version: u32,                          // The version number of the node software
      pub subversion: String,                    // The subversion of the node software (version/protocol details)
      pub protocolversion: u32,                  // The protocol version of the node
      pub localservices: String,                 // A bitfield representing supported services (hexadecimal)
      pub localservicesnames: Vec<String>,       // List of service names supported by the node (e.g., "NETWORK", "BLOOM")
      pub localrelay: bool,                      // Boolean flag indicating if the node is a local relay
      pub timeoffset: i32,                       // The time offset between the node's system clock and UTC (seconds)
      pub networkactive: bool,                   // Boolean flag indicating if the network is active
      pub connections: u32,                      // The total number of active network connections to the node
      pub connections_in: u32,                   // The number of incoming network connections to the node
      pub connections_out: u32,                  // The number of outgoing network connections from the node
      pub networks: Vec<Network>,                // List of network configurations for different network types (IPv4, Onion, etc.)
      pub relayfee: f64,                         // The fee in BTC the node is willing to relay transactions for
      pub incrementalfee: f64,                   // The incremental fee in BTC for transactions the node will relay
      pub localaddresses: Vec<LocalAddress>,     // List of local addresses used by the node
      pub warnings: String,                      // Any warnings issued by the node (e.g., network or node setup issues)
}


// Represents a specific network type (e.g., IPv4, IPv6, Onion) and its properties.
// This struct is used in the `networks` field of the `NetworkInfo` struct.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Network {
      pub name: String,                          // The name of the network (e.g., "ipv4", "onion")
      pub limited: bool,                         // Boolean flag indicating if the network is limited in functionality
      pub reachable: bool,                       // Boolean flag indicating if the network is reachable
      pub proxy: String,                         // The proxy address used for this network, if applicable
      pub proxy_randomize_credentials: bool,     // Boolean flag for randomizing proxy credentials
}


// Represents a local address used by the node.
// This struct is used in the `localaddresses` field of the `NetworkInfo` struct.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LocalAddress {
      pub address: String,                       // The address of the local node (domain or IP address)
      pub port: u16,                             // The port number used for the connection
      pub score: u32,                            // A score representing the node's quality or preference (higher is better)
}