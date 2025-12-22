// src/rpc/client.rs
use reqwest::{Client, Proxy};
use std::time::Duration;

pub fn build_rpc_client() -> Result<Client, reqwest::Error> {
    let is_proxied = std::env::var("BCI_RPC_PROXY").is_ok();

    let timeout = if is_proxied {
        Duration::from_secs(30)   // Tor breathing room
    } else {
        Duration::from_secs(10)   // LAN / local
    };

    let connect_timeout = if is_proxied {
        Duration::from_secs(15)
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
