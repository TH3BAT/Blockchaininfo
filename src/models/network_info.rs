
// models/network_info.rs

use serde::Deserialize; // For struct serialization and deserialization.

/// Wrapper Struct - The Bitcoin RPC response wraps the actual getnetworkinfo data inside the result field.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetworkInfoJsonWrap {
      pub error: Option<String>,    // Optional for any error message.
      pub id: Option<String>,       // Optional Request ID.
      pub result: NetworkInfo,   // The actual result field which contains the data we need.
}

/// This struct holds data from getnetworkinfo RPC method.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct NetworkInfo {
      pub version: u32,                          // The version number of the node software.
      pub subversion: String,                    // The subversion of the node software.
      pub protocolversion: u32,                  // The protocol version of the node.
      pub localservices: String,                 // A bitfield representing supported services hex
      pub localservicesnames: Vec<String>,       // List of service names supported by the node. 
      pub localrelay: bool,                      // Boolean flag indicating if node is local relay.
      pub timeoffset: i32,                       // Time offset between node's system clock & UTC. 
      pub networkactive: bool,                   // Boolean flag indicating if network is active.
      pub connections: u32,                      // The total number of active connections to node.
      pub connections_in: u32,                   // The number of incoming connections to node.
      pub connections_out: u32,                  // The number of outgoing connections from node.
      pub networks: Vec<Network>,                // List of network configurations types.
      pub relayfee: f64,                         // The fee in BTC the node will relay transactions.
      pub incrementalfee: f64,                   // The incremental fee in BTC the node will relay.
      pub localaddresses: Vec<LocalAddress>,     // List of local addresses used by the node.
      pub warnings: String,                      // Any warnings issued by the node. 
}

/// This struct holds the Network data from NetworkInfo struct.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct Network {
      pub name: String,                          // The name of the network (e.g., "ipv4").
      pub limited: bool,                         // Boolean flag indicating if network is limited. 
      pub reachable: bool,                       // Boolean flag indicating if network is reachable.
      pub proxy: String,                         // The proxy address used for this network.
      pub proxy_randomize_credentials: bool,     // Boolean flag for randomizing proxy credentials.
}

/// This struct holds the LocalAddress data from NetworkInfo struct.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct LocalAddress {
      pub address: String,                       // The address of the local node.
      pub port: u16,                             // The port number used for the connection.
      pub score: u32,                            // The node's quality or preference score.
}