
// models.rs

/// Module defines all the structs and implementations for getblockchaininfo.
pub mod blockchain_info;
/// Module defines all the structs and implementations for getmempoolinfo & getrawmempool.
pub mod mempool_info;
/// Module defines all the structs and implementations for getnetworkinfo.
pub mod network_info;
/// Module defines all the structs and implementations for custom error handling.
pub mod errors;
/// Module defines all the structs and implementations for getblockhash & getblock.
pub mod block_info;
/// Module defines all the structs and implementations for getchaintips.
pub mod chaintips_info;
/// Module defines all the structs and implementations for getnettotals.
pub mod network_totals;
/// Module defines all the structs and implementations for getpeerinfo.
pub mod peer_info;
/// Module defines all the structs and implementations for getrawtransaction & getmempoolentry.
pub mod transaction_info;
/// Module defines all the structs and implementations for flashing visuals in Dashboard.
pub mod flashing_text;

