
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
use crate::models::block_info::BlockInfo;
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::chaintips_info::ChainTip;
use crate::models::mempool_info::{MempoolDistribution, MempoolInfo};
use crate::models::network_info::NetworkInfo;
// use crate::models::errors::MyError;
use crate::models::network_totals::NetTotals;
use std::collections::VecDeque;

// Displays the blockchain information.
pub fn display_blockchain_info<B: Backend>(
    blockchain_info: &BlockchainInfo,
    block_info: &BlockInfo,
    block24_info: &BlockInfo,
    frame: &mut Frame<B>,
    area: Rect
) {
   let _ = display_blockchain_info::display_blockchain_info(blockchain_info, block_info, block24_info, frame, area);
}

// Displays the mempool information.
pub fn display_mempool_info<B: Backend>(
    mempool_info: &MempoolInfo,
    distribution: &MempoolDistribution,
    frame: &mut Frame<B>,
    area: Rect
) {
    let _ = display_mempool_info::display_mempool_info(mempool_info, distribution, frame, area);
}

// Displays the network information.
pub fn display_network_info<B: Backend>(
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    frame: &mut Frame<B>,
    version_counts: &[(String, usize)],
    avg_block_propagate_time: &i64, 
    propagation_times: &VecDeque<i64>,
    area: Rect,
) {
    let _ = display_network_info::display_network_info(network_info, net_totals, frame, version_counts, 
        avg_block_propagate_time, propagation_times, area);
}

// Displays the consensus security information.
pub fn display_consensus_security_info<B: Backend>(
    chaintips_info: &Vec<ChainTip>,
    frame: &mut Frame<B>,
    area: Rect
) {
    let _ = display_consensus_security_info::display_consensus_security_info(chaintips_info, frame, area);
}
