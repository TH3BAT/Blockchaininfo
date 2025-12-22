// src/rpc/client.rs
use reqwest::{Client, Proxy};
use std::time::Duration;

pub fn build_rpc_client() -> Result<Client, reqwest::Error> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5));

    if let Ok(proxy) = std::env::var("BCI_RPC_PROXY") {
        builder = builder.proxy(Proxy::all(&proxy)?);
    }

    builder.build()
}
