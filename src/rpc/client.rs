// src/rpc/client.rs/// Builds a preconfigured JSON-RPC HTTP client for Bitcoin RPC calls.
///
/// This client adapts its timeout behavior based on whether RPC traffic
/// is routed through a proxy (e.g., Tor).
///
/// ## Behavior
///
/// - If the `BCI_RPC_PROXY` environment variable is set:
///   • Requests are routed through the configured proxy  
///   • Timeouts are extended to tolerate Tor latency and circuit churn  
///
/// - If no proxy is configured:
///   • Direct LAN / localhost connections are assumed  
///   • Shorter, responsive timeouts are used for TUI snappiness  
///
/// ## Environment
///
/// - `BCI_RPC_PROXY`  
///   Optional. When set, must contain a valid proxy URL
///   (e.g., `socks5h://127.0.0.1:9050`).
///
/// ## Design Notes
///
/// - Timeouts are intentionally asymmetric:
///   • `connect_timeout` governs circuit establishment  
///   • `timeout` governs total request lifetime  
///
/// - Proxy configuration is explicit and does **not** rely on
///   system proxy auto-detection to avoid ambiguity.
///
/// - This function performs no I/O; it only constructs the client.
///
/// ## Errors
///
/// Returns an error if the proxy URL is invalid or the client
/// cannot be constructed.
use reqwest::{Client, Proxy};
use std::time::Duration;

pub fn build_rpc_client() -> Result<Client, reqwest::Error> {
    let is_proxied = std::env::var("BCI_RPC_PROXY").is_ok();

    let timeout = if is_proxied {
        Duration::from_secs(60)   // Tor breathing room
    } else {
        Duration::from_secs(10)   // LAN / local
    };

    let connect_timeout = if is_proxied {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(5)
    };

    let mut builder = Client::builder()
        .timeout(timeout)
        .connect_timeout(connect_timeout);

    if let Ok(proxy) = std::env::var("BCI_RPC_PROXY") {
        builder = builder.proxy(Proxy::all(&proxy)?);
    }

    builder.build()
}
