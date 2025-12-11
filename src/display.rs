//! High-level interface for rendering all TUI dashboard sections.
//!
//! This module serves as the *public display API* of the application,
//! delegating actual rendering logic to specialized submodules:
//!
//! - `display_blockchain_info`        → Blockchain metrics + Hash Rate Distribution
//! - `display_mempool_info`           → Mempool metrics + fee distribution
//! - `display_network_info`           → Network connections, peers, versions, clients
//! - `display_consensus_security_info`→ Fork awareness and chain tip health
//!
//! The functions here act as thin wrappers around the internal renderers,
//! keeping the `runapp` module clean while preserving clear boundaries
//! between layout orchestration and section-specific drawing logic.
//
//  ─────────────────────────────────────────────────────────────────────────────
//  Color Strategy Summary
//  -----------------------------------------------------------------------------
//  ✔ Green        → Healthy, stable, expected values (block height, peers)
//  ✔ Yellow       → Important but normal metrics (relay fee, verification)
//  ✔ BrightYellow → Accumulated effort metrics (chainwork)
//  ✔ BrightRed    → Critical consensus metrics (difficulty)  
//  ✔ Red          → Urgent timing metrics (time-since-block)
//  ✔ White        → Neutral, descriptive values (sizes, timestamps, fees)
//  
//  This palette ensures visual hierarchy and immediate signal recognition.
//  ─────────────────────────────────────────────────────────────────────────────

/// Blockchain section renderer (base metrics + charts).
pub mod display_blockchain_info;
/// Mempool section renderer.
pub mod display_mempool_info;
/// Network section renderer.
pub mod display_network_info;
/// Consensus security / fork monitoring renderer.
pub mod display_consensus_security_info;

use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;

use crate::models::block_info::BlockInfo;
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::chaintips_info::ChainTip;
use crate::models::mempool_info::{MempoolDistribution, MempoolInfo};
use crate::models::network_info::NetworkInfo;
use crate::models::network_totals::NetTotals;

use std::collections::VecDeque;
use std::sync::Arc;

/// Render the `[Blockchain]` section: block metadata, difficulty epoch,
/// verification progress, latest miner, etc.
///
/// Internally delegates to `display_blockchain_info::display_blockchain_info`.
pub fn display_blockchain_info<B: Backend>(
    blockchain_info: &BlockchainInfo,
    block_info: &BlockInfo,
    block24_info: &BlockInfo,
    last_miner: &Arc<str>,
    frame: &mut Frame<B>,
    area: Rect,
) {
    let _ = display_blockchain_info::display_blockchain_info(
        blockchain_info,
        block_info,
        block24_info,
        last_miner,
        frame,
        area,
    );
}

/// Render the Hash Rate Distribution chart (toggled via `h`).
///
/// This function acts as a stable public wrapper, preventing the TUI layer
/// from depending on internal module paths.
pub fn render_hashrate_distribution_chart<B: Backend>(
    distribution: &Vec<(Arc<str>, u64)>,
    frame: &mut Frame<B>,
    area: Rect,
) {
    let _ = display_blockchain_info::render_hashrate_distribution_chart(
        distribution,
        frame,
        area,
    );
}

/// Render the `[Mempool]` section: mempool stats, fee distribution,
/// dust filtering mode, etc.
///
/// Delegates to `display_mempool_info::display_mempool_info`.
pub fn display_mempool_info<B: Backend>(
    mempool_info: &MempoolInfo,
    distribution: &MempoolDistribution,
    dust_free: bool,
    frame: &mut Frame<B>,
    area: Rect,
) {
    let _ = display_mempool_info::display_mempool_info(
        mempool_info,
        distribution,
        dust_free,
        frame,
        area,
    );
}

/// Render the `[Network]` section: node info, version/client distribution charts,
/// peer count, data in/out, block propagation, and optional client distribution view.
pub fn display_network_info<B: Backend>(
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    frame: &mut Frame<B>,
    version_counts: &[(String, usize)],
    client_counts: &[(String, usize)],
    avg_block_propagate_time: &i64,
    propagation_times: &VecDeque<i64>,
    show_client_distribution: bool,
    area: Rect,
) {
    let _ = display_network_info::display_network_info(
        network_info,
        net_totals,
        frame,
        version_counts,
        client_counts,
        avg_block_propagate_time,
        propagation_times,
        show_client_distribution,
        area,
    );
}

/// Render the `[Consensus Security]` section: fork visibility, active tips,
/// and chain health.
///
/// Delegates to `display_consensus_security_info`.
pub fn display_consensus_security_info<B: Backend>(
    chaintips_info: &Vec<ChainTip>,
    frame: &mut Frame<B>,
    area: Rect,
) {
    let _ = display_consensus_security_info::display_consensus_security_info(
        chaintips_info, frame, area,
    );
}
