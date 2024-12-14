
// display.rs

/// Summary of Color Usage:
/// Green: Used for healthy values like block count, transactions, and connections.
///
/// Yellow: Used for values that are important but not urgent, like chain info, verification progress, 
/// and min relay transaction fees.
///
/// Bright Yellow: For metrics like chainwork that represent cumulative effort.
///
/// Bright Red: Used for metrics like difficulty to make them stand out, signaling critical importance.
///
/// Red: Used for critical time-related fields like "Time since block" to signal urgency.
///
/// White: Used for neutral data, like timestamps, sizes, and fees, where urgency is not a factor.
///
/// This approach ensures that critical, urgent, and less critical information is presented in a way 
/// that emphasizes what’s most important at a glance.
///
pub mod display_blockchain_info;
pub mod display_mempool_info;
pub mod display_network_info;

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::errors::MyError;

// Displays the blockchain information.
pub fn display_blockchain_info(blockchain_info: &BlockchainInfo) -> Result<(), MyError> {
    display_blockchain_info::display_blockchain_info(blockchain_info)
    
}

// Displays the mempool information.
pub fn display_mempool_info(mempool_info: &MempoolInfo) -> Result<(), MyError> {
    display_mempool_info::display_mempool_info(mempool_info)
    
}

// Displays the network information.
pub fn display_network_info(network_info: &NetworkInfo) -> Result<(), MyError> {
    display_network_info::display_network_info(network_info)
    
}
