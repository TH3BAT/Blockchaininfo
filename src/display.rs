
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
/// that emphasizes what's most important at a glance.
///
pub mod display_blockchain_info;
pub mod display_mempool_info;
pub mod display_network_info;
pub mod display_consensus_security_info;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::BlockInfo;
use crate::models::mempool_info::{MempoolInfo, MempoolDistribution};
use crate::models::network_info::NetworkInfo;
use crate::models::chaintips_info::ChainTip;
use crate::models::network_totals::NetTotals;
use crate::models::errors::MyError;
use std::collections::VecDeque;

// Displays the blockchain information.
pub fn display_blockchain_info<B: Backend>(
    frame: &mut Frame<B>,
    blockchain_info: &BlockchainInfo,
    block_info: &BlockInfo,
    area: Rect
) -> Result<(), MyError> {
    display_blockchain_info::display_blockchain_info(frame, blockchain_info, block_info, area)
}

// Displays the mempool information.
pub fn display_mempool_info<B: Backend>(
    frame: &mut Frame<B>,
    mempool_info: &MempoolInfo,
    distribution: &MempoolDistribution,
    area: Rect, // Add the 'area' parameter.
) -> Result<(), MyError> {
    display_mempool_info::display_mempool_info(frame, mempool_info, distribution, area)
}

// Displays the network information.
pub fn display_network_info<B: Backend>(
    frame: &mut Frame<B>,
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    version_counts: &[(String, usize)],
    avg_block_propagate_time: &i64, 
    propagation_times: &VecDeque<i64>,
    area: Rect,
) -> Result<(), MyError> {
    display_network_info::display_network_info(frame, network_info, net_totals, version_counts, 
        avg_block_propagate_time, propagation_times, area)
}


// Displays the consensus security information.
pub fn display_consensus_security_info<B: Backend>(
    frame: &mut tui::Frame<B>,
    chaintips_info: &[ChainTip], // Accepts a slice of ChainTip.
    area: tui::layout::Rect,
) -> Result<(), MyError> {
    display_consensus_security_info::display_consensus_security_info(frame, chaintips_info, area)
}
