// =================================================================================================
// runapp.rs — Main Runtime Engine
// BlockChainInfo v1.0.0
//
// This module coordinates the entire lifecycle of the TUI dashboard:
// • Terminal setup & teardown
// • All asynchronous RPC polling loops (blockchain, mempool, peers, chain tips, etc.)
// • State management for toggles and popups
// • Rendering pipeline for all dashboard sections
// • Dynamic input handling and popup UI logic
// • Fork detection alerts and consensus warnings
//
// HYBRID DOCUMENTATION MODE:
// - Full inline commentary explaining concepts, intent, and architectural reasoning
// - Light formatting improvements for long lines or dense logic
// - ABSOLUTELY ZERO functional behavior changes
//
// This is the heart of BlockChainInfo. The sovereign engine.
// =================================================================================================

use crate::config::RpcConfig;

// RPC fetch routines — each returns structured response data or MyError.
use crate::rpc::{
    fetch_blockchain_info,
    fetch_mempool_info,
    fetch_network_info,
    fetch_block_data_by_height,
    fetch_chain_tips,
    fetch_net_totals,
    fetch_peer_info,
    fetch_mempool_distribution,
    fetch_transaction,
    fetch_miner,
};

use crate::models::errors::MyError;

// UI render functions for each major dashboard section.
use crate::display::{
    display_blockchain_info,
    display_mempool_info,
    display_network_info,
    display_consensus_security_info,
    render_hashrate_distribution_chart,
    draw_last20_miners,
};

// Misc utilities: header/footer, miner loader, block history tracker.
use crate::utils::{render_header, render_footer, load_miners_data, BLOCK_HISTORY};

// For peer aggregation functions (versions, clients, etc.)
use crate::models::peer_info::{PeerInfo, NetworkState};

// TUI dependencies
use tui::{
    backend::{CrosstermBackend, Backend},
    layout::{Layout, Constraint, Direction, Margin, Rect, Alignment},
    widgets::{Block, Borders, Paragraph, Clear, Wrap, BorderType},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    Frame,
    Terminal,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io::{self, Stdout};
use std::collections::VecDeque;
use std::sync::Arc;

use tokio::time::{sleep, Duration, Instant};

use blockchaininfo::utils::log_error;
use crate::ui::colors::*;

use crate::models::chaintips_info::ChainTipsJsonWrap;

// DashSet is used for tracking unique block numbers (propagation-time updates)
use dashmap::DashSet;

// OnceCell provides a lazy static container.
use once_cell::sync::Lazy;

// Shared caches used across async tasks for concurrency-safe data access.
use crate::utils::{
    BLOCKCHAIN_INFO_CACHE,
    BLOCK_INFO_CACHE,
    MEMPOOL_INFO_CACHE,
    CHAIN_TIP_CACHE,
    BLOCK24_INFO_CACHE,
    PEER_INFO_CACHE,
    NETWORK_INFO_CACHE,
    NET_TOTALS_CACHE,
    MEMPOOL_DISTRIBUTION_CACHE,
};

// Atomic flags used for toggles (no locking overhead).
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};


/// Popup windows used in the application.
#[derive(PartialEq)]
pub enum PopupType {
    None,
    TxLookup,
    Help,
    ConsensusWarning,
}

/// Global application state.
/// Tracks UI mode, popup state, toggles, paste-detection, etc.
struct App {
    popup: PopupType,            // Which popup is currently open
    tx_input: String,            // TxID text buffer
    tx_result: Option<String>,   // RPC result for Tx lookup
    is_exiting: bool,            // Whether 'q' has been pressed for shutdown
    is_pasting: bool,            // Detect multi-character paste events
    show_hash_distribution: bool,// Toggle: Hashrate Distribution view
    dust_free: Arc<AtomicBool>,  // Toggle: Dust filtering for mempool distro
    show_client_distribution: bool, // NEW toggle: Version vs Client view
    last_fork_alert_height: Option<u64>, // For deduping fork warning popups
    show_propagation_avg: bool, // NEW toggle: Propagation average over 20 block period
    last_block: Arc<AtomicU64>, // last block to pass to mempool_distro
    show_last20_miners: bool,   // Toggle: Show last 20 blocks / miners.
    last20_miners: Vec<(u64, Option<Arc<str>>)>,
}

impl App {
    /// Creates fresh runtime state for the TUI.
    fn new() -> Self {
        Self {
            popup: PopupType::None,
            tx_input: String::new(),
            tx_result: None,
            is_exiting: false,
            is_pasting: false,
            show_hash_distribution: false,
            dust_free: Arc::new(AtomicBool::new(true)), // dust-free enabled by default
            show_client_distribution: false,            // default: show Version view
            last_fork_alert_height: None,
            show_propagation_avg: false,                //default: show sparkline view
            last_block: Arc::new(AtomicU64::new(0)),
            show_last20_miners: false,
            last20_miners: Vec::new(),
        }
    }
}

/// Tracks the last block number whose propagation time has been recorded.
/// DashSet gives us thread-safe "contains" and insert operations.
static LAST_BLOCK_NUMBER: Lazy<DashSet<u64>> = Lazy::new(|| DashSet::new());


// =================================================================================================
// TERMINAL SETUP / CLEANUP
// =================================================================================================

/// Enter TUI mode by enabling raw mode and swapping into the alternate screen.
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal to normal mode when exiting.
pub fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}


// =================================================================================================
// MAIN APPLICATION LOOP
// =================================================================================================

/// Main runtime entry point.
/// Spawns several background tasks to poll RPC endpoints, updates caches,
/// and renders the dashboard at interactive speed.
pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    config: &RpcConfig,
) -> Result<(), MyError> {

    // Rolling 20-sample window of block propagation times.
    let mut propagation_times: VecDeque<i64> = VecDeque::with_capacity(20);

    // Local UI state.
    let mut app = App::new();

    // Miner name/address lookup table.
    let miners_data = load_miners_data()?;

    // Shared default miner string for fallback cases.
    let default_miner = "Unknown".to_string();

    // Stores block-anchored propagation slots
    let mut network_state = NetworkState {
        last_propagation_index: None,
        last_block_seen: 0,
    };


    // Draw initial "Initializing…" screen.
    terminal.draw(|frame| {
        let area = frame.size();
        let block = Block::default().title("Initializing...").borders(Borders::ALL);
        frame.render_widget(block, area);
    })?;

    // =============================================================================================
    // RPC WORKER TASK: BLOCKCHAIN INFO + BLOCK & 24H BLOCK FETCH
    // =============================================================================================
    //
    // Runs every ~2 seconds. Updates:
    //  • Latest blockchain height
    //  • Latest block data
    //  • Block data from 24 hours ago
    //
    tokio::spawn({
        let config_clone = config.clone();

        async move {
            loop {
                let start = Instant::now();

                // --- Step 1: Fetch blockchain_info (height, difficulty, chain, etc.) ---
                match fetch_blockchain_info(&config_clone).await {
                    Ok(new_blockchain_info) => {
                        let mut cache = BLOCKCHAIN_INFO_CACHE.write().await;

                        // Avoid unnecessary updates to allow the UI to stay calm.
                        if *cache != new_blockchain_info {
                            *cache = new_blockchain_info;
                        } else {
                            // Data did not change — sleep the remainder of 2 seconds.
                            sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    }
                    Err(e) => {
                        if let Err(_log_err) =
                            log_error(&format!("Blockchain Info failed: {}", e))
                        {
                            // eprintln!("Failed to log error: {}", log_err);
                        }
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }

                // --- Step 2: Extract block height from cache ---
                let block_height = {
                    let blockchain_info = BLOCKCHAIN_INFO_CACHE.read().await;
                    blockchain_info.blocks
                };

                // --- Step 3: Fetch block data for *latest* block ---
                match fetch_block_data_by_height(&config_clone, block_height, 1).await {
                    Ok(new_data) => {
                        let mut cache = BLOCK_INFO_CACHE.write().await;

                        if cache.len() >= 1 {
                            cache.remove(0);
                        }
                        cache.push(new_data);
                    }
                    Err(e) => {
                        let _ = log_error(&format!(
                            "Block Data by Height failed at height {}: {}",
                            block_height, e
                        ));
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }

                // --- Step 4: Fetch the block from ~24 hours ago ---
                match fetch_block_data_by_height(&config_clone, block_height, 2).await {
                    Ok(block24_data) => {
                        let mut cache_24 = BLOCK24_INFO_CACHE.write().await;

                        if cache_24.len() >= 1 {
                            cache_24.remove(0);
                        }
                        cache_24.push(block24_data);
                    }
                    Err(e) => {
                        let _ = log_error(&format!(
                            "Block Data 24h failed at height {}: {}",
                            block_height, e
                        ));
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }

                // Maintain a strict ~2-second loop duration.
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(2) {
                    sleep(Duration::from_secs(2) - elapsed).await;
                }
            }
        }
    });


    // =============================================================================================
    // RPC WORKER TASK: MEMPOOL INFO
    // =============================================================================================
    //
    // Updates general mempool statistics. Runs every 3 seconds.
    //
    tokio::spawn({
        let config_clone = config.clone();

        async move {
            loop {
                let start = Instant::now();

                match fetch_mempool_info(&config_clone).await {
                    Ok(new_data) => {
                        let mut cache = MEMPOOL_INFO_CACHE.write().await;

                        if *cache != new_data {
                            *cache = new_data;
                        }
                    }
                    Err(e) => {
                        let _ = log_error(&format!("Mempool Info failed: {}", e));
                    }
                }

                // Maintain ~3-second pacing.
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(3) {
                    sleep(Duration::from_secs(3) - elapsed).await;
                }
            }
        }
    });


    // =============================================================================================
    // RPC WORKER TASK: NETWORK INFO
    // =============================================================================================
    //
    // Updates peer count, local services, version info, and related fields.
    // Runs every 7 seconds.
    //
    tokio::spawn({
        let config_clone = config.clone();

        async move {
            loop {
                let start = Instant::now();

                match fetch_network_info(&config_clone).await {
                    Ok(new_data) => {
                        let mut cache = NETWORK_INFO_CACHE.write().await;

                        if *cache != new_data {
                            *cache = new_data;
                        }
                    }
                    Err(e) => {
                        let _ = log_error(&format!("Network Info failed: {}", e));
                    }
                }

                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(7) {
                    sleep(Duration::from_secs(7) - elapsed).await;
                }
            }
        }
    });

// =============================================================================================
// RPC WORKER TASK: PEER INFO
// =============================================================================================
// Polls the node's peers list. Provides the raw data used to compute:
//   • Version distribution
//   • Client distribution
//   • Block propagation time estimates
//
// Runs every ~7 seconds. Peer sets rarely change faster than this.
//
tokio::spawn({
    let config_clone = config.clone();

    async move {
        loop {
            let start = Instant::now();

            match fetch_peer_info(&config_clone).await {
                Ok(new_data) => {
                    let mut cache = PEER_INFO_CACHE.write().await;

                    // Replace wholesale to avoid stale entries from removed peers.
                    if *cache != new_data {
                        cache.clear();
                        cache.extend(new_data);
                    }
                }
                Err(e) => {
                    let _ = log_error(&format!("Peer Info failed: {}", e));
                }
            }

            // Maintain ~7 second pacing.
            let elapsed = start.elapsed();
            if elapsed < Duration::from_secs(7) {
                sleep(Duration::from_secs(7) - elapsed).await;
            }
        }
    }
});


// =============================================================================================
// RPC WORKER TASK: CHAIN TIPS
// =============================================================================================
// Retrieves alternative chain tips (stale forks, valid forks, headers-only tips).
// This data drives the Consensus Warning popup.
// Runs every ~10 seconds.
//
tokio::spawn({
    let config_clone = config.clone();

    async move {
        loop {
            let start = Instant::now();

            match fetch_chain_tips(&config_clone).await {
                Ok(new_data) => {
                    let mut cache = CHAIN_TIP_CACHE.write().await;

                    // Wrap tips in the full RPC-style response struct.
                    let new_response = ChainTipsJsonWrap {
                        error: None,
                        id: None,
                        result: new_data,
                    };

                    if *cache != new_response {
                        *cache = new_response;
                    }
                }
                Err(e) => {
                    let _ = log_error(&format!("Chain Tips failed: {}", e));
                }
            }

            let elapsed = start.elapsed();
            if elapsed < Duration::from_secs(10) {
                sleep(Duration::from_secs(10) - elapsed).await;
            }
        }
    }
});


// =============================================================================================
// RPC WORKER TASK: NET TOTALS
// =============================================================================================
// Retrieves running totals of bytes sent/received from the node.
// Useful for diagnosing traffic flow or seeing relay throttling.
//
tokio::spawn({
    let config_clone = config.clone();

    async move {
        loop {
            let start = Instant::now();

            match fetch_net_totals(&config_clone).await {
                Ok(new_data) => {
                    let mut cache = NET_TOTALS_CACHE.write().await;

                    if *cache != new_data {
                        *cache = new_data;
                    }
                }
                Err(e) => {
                    // Log but never break the loop.
                    let _ = log_error(&format!("Net Totals failed: {}", e));
                }
            }

            // Maintain ~7 second pacing.
            let elapsed = start.elapsed();
            if elapsed < Duration::from_secs(7) {
                sleep(Duration::from_secs(7) - elapsed).await;
            }
        }
    }
});


// =============================================================================================
// RPC WORKER TASK: MEMPOOL DISTRIBUTION
// =============================================================================================
// This worker computes a miner-fee-tier & size-bucket distribution chart
// used in the mempool section of the UI.
//
// Important:
//   • Uses the dust_free toggle to filter out tiny transactions.
//   • Runs every 2 seconds for responsive charts.
//
// The previously complex TxID regex dedupe system has been removed —
// distribution errors no longer require granular logging.
//
let dust_flag = app.dust_free.clone();
let last_block_clone = app.last_block.clone();

tokio::spawn({
    let config_clone = config.clone();

    async move {
        loop {
            let start = Instant::now();
            let dust_free = dust_flag.load(Ordering::Relaxed);
            let last_block = last_block_clone.load(Ordering::Relaxed); 

            if let Err(e) = fetch_mempool_distribution(&config_clone, dust_free, last_block).await {
                // Distribution failures are usually transient due to mempool churn.
                let _ = &e; // intentionally unused now
            }

            let elapsed = start.elapsed();
            if elapsed < Duration::from_secs(2) {
                sleep(Duration::from_secs(2) - elapsed).await;
            }
        }
    }
});


// =================================================================================================
// SMALL SYNC BEFORE MAIN UI LOOP STARTS
// =================================================================================================

sleep(Duration::from_secs(1)).await;


// =================================================================================================
// MAIN DRAW LOOP — THE HEART OF THE DASHBOARD
// =================================================================================================
// Each iteration:
//   1. Pulls *all* cached RPC data via tokio::join!
//   2. Computes epoch progress, propagation time windows, miner info.
//   3. Handles all user input & popup state changes.
//   4. Renders every section of the UI.
//
// This loop never blocks on network I/O — all fetches happen inside background tasks.
//
loop {
    // ---------------------------------------------------------------------------------------------
    // Step 1: Retrieve all data from caches simultaneously.
    // ---------------------------------------------------------------------------------------------
    let (
        blockchain_info,
        mempool_info,
        network_info,
        peer_info,
        block_info,
        block24_info,
        net_totals,
        distribution,
        chaintips_info,
    ) = tokio::join!(
        BLOCKCHAIN_INFO_CACHE.read(),
        MEMPOOL_INFO_CACHE.read(),
        NETWORK_INFO_CACHE.read(),
        PEER_INFO_CACHE.read(),
        BLOCK_INFO_CACHE.read(),
        BLOCK24_INFO_CACHE.read(),
        NET_TOTALS_CACHE.read(),
        MEMPOOL_DISTRIBUTION_CACHE.read(),
        CHAIN_TIP_CACHE.read(),
    );

    app.last_block.store(blockchain_info.blocks, Ordering::Relaxed);

    // ---------------------------------------------------------------------------------------------
    // Epoch progress indicator — drives the animated header ("Flip Dot" logic).
    // ---------------------------------------------------------------------------------------------
    let into_epoch = blockchain_info.blocks % 2016;
    let percent = (into_epoch as f64 / 2016.0) * 100.0;

    // ---------------------------------------------------------------------------------------------
    // Consensus Warning Trigger
    // If any chaintip is a "valid-fork" of length >= 2, show warning.
    // Dedup using last_fork_alert_height so the popup appears only once per fork height.
    // ---------------------------------------------------------------------------------------------
    let chaintips_result = &chaintips_info.result;

    for tip in chaintips_result {
        if tip.status == "valid-fork" && tip.branchlen >= 2 {
            if app.last_fork_alert_height != Some(tip.height) {
                app.last_fork_alert_height = Some(tip.height);
                app.popup = PopupType::ConsensusWarning;
            }
            break;
        }
    }

    // ---------------------------------------------------------------------------------------------
    // Peer Aggregations: Versions & Clients
    // Used by the Network section based on toggle mode.
    // ---------------------------------------------------------------------------------------------
    let version_counts = PeerInfo::aggregate_and_sort_versions(&peer_info);
    let client_counts = PeerInfo::aggregate_and_sort_clients(&peer_info);

    // ---------------------------------------------------------------------------------------------
    // Block Propagation Time Estimation
    // Computes per-peer propagation delay, then averages it.
    // ---------------------------------------------------------------------------------------------
    let avg_block_propagate_time = PeerInfo::calculate_block_propagation_time(
        &peer_info,
        blockchain_info.time,
        blockchain_info.blocks,
    );

    // ---------------------------------------------------------------------------------------------
    // Rolling Propagation-Time Tracking (20 sample window)
    // Deduped by remembering the last block number seen.
    // ---------------------------------------------------------------------------------------------
    if !LAST_BLOCK_NUMBER.contains(&blockchain_info.blocks) {
        // New block — push a fresh propagation sample.
        if propagation_times.len() == 20 {
            propagation_times.pop_front();
        }
        propagation_times.push_back(avg_block_propagate_time);
        network_state.last_propagation_index = Some(propagation_times.len() - 1);
        network_state.last_block_seen = blockchain_info.blocks;

        LAST_BLOCK_NUMBER.clear();
        LAST_BLOCK_NUMBER.insert(network_state.last_block_seen);

        // Also fetch miner attribution for the new block.
        let block = network_state.last_block_seen;

        let _ = fetch_miner(&config, &miners_data, &block).await;

    } else {
        // Same block — but propagation estimate changed.
        if network_state.last_block_seen == blockchain_info.blocks {
            if let Some(idx) = network_state.last_propagation_index {
                if let Some(val) = propagation_times.get_mut(idx) {
                    if *val != avg_block_propagate_time {
                        *val = avg_block_propagate_time;
                    }
                }
            }
        }
    }
    // =============================================================================================
    // MINER DISTRIBUTION + LAST MINER RESOLUTION
    // =============================================================================================

    // Safely read the last-mined block's miner attribution.
    let last_miner = BLOCK_HISTORY.read().await.last_miner();

    // Convert default miner string → Arc<str> (so we can store by reference without ownership issues)
    let default_miner_arc = Arc::from(default_miner.as_str());

    // Resolve to actual miner if known, otherwise fallback
    let last_miner_ref = last_miner.as_ref().unwrap_or(&default_miner_arc);

    // Construct Hashrate Distribution vector for the Blockchain section toggle.
    //
    // NOTE:
    //  We intentionally convert miners into Arc<str> to cheaply clone & pass them.
    //
    let hash_distribution: Vec<(Arc<str>, u64)> = BLOCK_HISTORY
        .read()
        .await
        .get_miner_distribution()
        .iter()
        .map(|(miner, hashrate)| (Arc::from(miner.to_string()), *hashrate))
        .collect();

    // Construct last 20 miners and heights vector for the Blockchain section toggle.
    let last20_miners = {
        let h = BLOCK_HISTORY.read().await;
        h.last_n_with_heights(network_state.last_block_seen, 20)
    };
    app.last20_miners = last20_miners;

    // =============================================================================================
    // INPUT POLLING — Adaptive Polling Rate
    // =============================================================================================
    //
    // When the Tx Lookup popup is open, we poll keyboard input faster (50ms)
    // for responsive typing and paste detection.
    //
    // During normal dashboard view, relax to 250ms to reduce CPU noise.
    //
    let poll_time = if app.popup == PopupType::TxLookup {
        Duration::from_millis(50)
    } else {
        Duration::from_millis(250)
    };

    // =============================================================================================
    // INPUT HANDLING — Key Press Logic
    // =============================================================================================
    //
    // This section handles:
    //   • App shutdown (q)
    //   • Popup opening/closing (t, ?, Esc)
    //   • Hashrate & mempool toggles (h, d)
    //   • Version <-> Client toggle (c)
    //   • TxID text input (typing/paste)
    //
    if event::poll(poll_time)? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                // Close popup panels
                KeyCode::Esc if app.popup != PopupType::None => {
                    app.popup = PopupType::None;
                    app.is_pasting = false;
                }

                // Begin Shutdown
                KeyCode::Char('q') if !app.is_pasting => {
                    app.is_exiting = true;

                    // Manual layout for one last clean exit frame
                    let size = terminal.size()?;
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Length(14),
                                Constraint::Length(25),
                                Constraint::Max(18),
                                Constraint::Length(7),
                                Constraint::Length(1),
                            ]
                            .as_ref(),
                        )
                        .split(size);

                    terminal.draw(|frame| {
                        render_footer(frame, chunks[5], "Shutting Down Cleanly...");
                    })?;

                    std::thread::sleep(std::time::Duration::from_millis(500));
                    break;
                }

                // Open Tx Lookup popup
                KeyCode::Char('t') if app.popup == PopupType::None => {
                    app.popup = PopupType::TxLookup;
                    app.tx_input.clear();
                    app.tx_result = None;
                    app.is_pasting = false;
                }

                // Open Help popup
                KeyCode::Char('?') if app.popup == PopupType::None => {
                    app.popup = PopupType::Help;
                }

                // Hashrate Distribution toggle
                KeyCode::Char('h') if app.popup == PopupType::None && !app.show_last20_miners => {
                    app.show_hash_distribution = !app.show_hash_distribution;
                }

                // Last 20 miners and heights toggle
                KeyCode::Char('l') if app.popup == PopupType::None && !app.show_hash_distribution => {
                    app.show_last20_miners = !app.show_last20_miners;
                }

                // CHARACTER INPUT inside Tx Lookup popup
                KeyCode::Char(c) if app.popup == PopupType::TxLookup => {
                    if app.is_pasting {
                        // Ignore weird control characters during paste
                        if c != 'q' && c != '\n' {
                            app.tx_input.push(c);
                        }
                    } else {
                        app.tx_input.push(c);
                    }

                    // Heuristic: if input suddenly becomes long, treat it as a paste event
                    if !app.is_pasting && app.tx_input.len() > 10 {
                        app.is_pasting = true;
                    }
                }

                // Backspace logic inside Tx Lookup popup
                KeyCode::Backspace if app.popup == PopupType::TxLookup => {
                    app.tx_input.pop();
                    app.is_pasting = false;
                }

                // Press Enter inside Tx Lookup popup → run validation + RPC
                KeyCode::Enter if app.popup == PopupType::TxLookup => {
                    let trimmed = app.tx_input.trim();

                    if !trimmed.is_empty() {
                        if is_valid_txid(trimmed) {
                            let tx_result = fetch_transaction(&config, trimmed).await;

                            app.tx_result = tx_result.map_or_else(
                                |e| Some(format!("{}", e)),
                                Some,
                            );
                        } else {
                            app.tx_result = Some(
                                "Invalid TxID. Please enter a 64-character hex string."
                                    .to_string()
                            );
                        }
                        app.is_pasting = false;
                    }
                }

                // DUST-FREE toggle for mempool distribution
                KeyCode::Char('d') => {
                    let old = app.dust_free.load(Ordering::Relaxed);
                    app.dust_free.store(!old, Ordering::Relaxed);
                }

                // Version <-> Client distribution toggle
                KeyCode::Char('c') => {
                    app.show_client_distribution = !app.show_client_distribution;
                }

                 // Propagation sparkline <-> average toggle
                KeyCode::Char('p') => {
                    app.show_propagation_avg = !app.show_propagation_avg;
                }
                // If a non-character key is pressed during paste, end paste mode.
                _ => {
                    if app.is_pasting {
                        app.is_pasting = false;
                    }
                }
            }
        }
    }


    // =============================================================================================
    // MAIN RENDERING PASS — Draw All Dashboard Sections
    // =============================================================================================

    terminal.draw(|frame| {
        // Layout of the entire dashboard (vertical stacking)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),   // Header
                    Constraint::Length(14),  // Blockchain
                    Constraint::Length(25),  // Mempool
                    Constraint::Max(18),     // Network
                    Constraint::Length(7),   // Consensus Security
                    Constraint::Length(1),   // Footer
                ]
                .as_ref(),
            )
            .split(frame.size());

        // -----------------------------------------------------------------------------------------
        // HEADER SECTION
        // -----------------------------------------------------------------------------------------
        {
            let header_block = Block::default().borders(Borders::NONE);
            frame.render_widget(header_block, chunks[0]);

            let header_widget = render_header(percent);
            frame.render_widget(header_widget, chunks[0]);
        }

        // -----------------------------------------------------------------------------------------
        // BLOCKCHAIN SECTION
        // -----------------------------------------------------------------------------------------

        // Build HRD toggle label
        let hrd_label = if app.show_hash_distribution {
            Span::styled(
                "[H] HRD",
                Style::default().fg(C_KEYTOGGLE_HIGHLIGHT).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("[H] HRD", Style::default().fg(C_KEYTOGGLE_DIM))
        };

        // Build Last20 toggle label
        let last20_label = if app.show_last20_miners {
            Span::styled(
                "[L] 20",
                Style::default().fg(C_KEYTOGGLE_HIGHLIGHT).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("[L] 20", Style::default().fg(C_KEYTOGGLE_DIM))
        };

        // Full title for Blockchain block
        let blockchain_title = Spans::from(vec![
            Span::styled(
                "[Blockchain] ",
                Style::default()
                    .fg(C_SECTION_LABELS)
                    .add_modifier(Modifier::BOLD),
            ),
            hrd_label,
            Span::raw(" "), // spacing
            last20_label,
        ]);

        let block_blockchain = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_BLOCKCHAIN_BORDER))
            .border_type(BorderType::Rounded)
            .title(blockchain_title);

        frame.render_widget(block_blockchain, chunks[1]);

        // Choose between HRD chart OR normal blockchain info
        if app.show_hash_distribution {
            render_hashrate_distribution_chart(&hash_distribution, frame, chunks[1]);
        
        } else if app.show_last20_miners {
            // assuming you already computed rows in runapp and have them available here
            // e.g., `last20_rows: &[(u64, Option<Arc<str>>)]`
            draw_last20_miners(frame, chunks[1], &app.last20_miners);
        
        } else {
            if !block_info.is_empty() && !block24_info.is_empty() {
                let latest_block = &block_info[block_info.len() - 1];
                let block24 = &block24_info[block24_info.len() - 1];
        
                display_blockchain_info(
                    &blockchain_info,
                    latest_block,
                    block24,
                    last_miner_ref,
                    frame,
                    chunks[1],
                );
            }
        }
        

        // -----------------------------------------------------------------------------------------
        // MEMPOOL SECTION
        // -----------------------------------------------------------------------------------------

        // Dust-free toggle label
        let dust_label = if app.dust_free.load(Ordering::Relaxed) {
            Span::styled(
                " [D] DUST-FREE",
                Style::default()
                    .fg(C_KEYTOGGLE_HIGHLIGHT)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(" [D] ALL TX", Style::default().fg(C_KEYTOGGLE_DIM))
        };

        let mempool_title = Spans(vec![
            Span::styled(
                "[Mempool]",
                Style::default()
                    .fg(C_SECTION_LABELS)
                    .add_modifier(Modifier::BOLD),
            ),
            dust_label,
        ]);

        let block_mempool = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_MEMPOOL_BORDER))
            .border_type(BorderType::Rounded)
            .title(mempool_title);

        frame.render_widget(block_mempool, chunks[2]);

        display_mempool_info(
            &mempool_info,
            &distribution,
            app.dust_free.load(Ordering::Relaxed),
            frame,
            chunks[2],
        );

        // -----------------------------------------------------------------------------------------
        // NETWORK SECTION
        // -----------------------------------------------------------------------------------------

        // Label describing what pressing 'c' will toggle TO
        let cv_label = if app.show_client_distribution {
            "(c→Version)"
        } else {
            "(c→Client)"
        };
        
        // Label describing what pressing 'p' will toggle TO
        let prop_label = if app.show_propagation_avg {
            "(p→Spark)"
        } else {
            "(p→Avg)"
        };

        // If node is absent populate with micro-glyph for Network title header.
        let network_absence = if network_info.connections_out == 0 &&
            network_info.connections_in == 0 {
                Some("∅")
        } else {
                None
        };

        let title = match network_absence {
            Some(glyph) => format!("[Network] {} {}  {}", cv_label, prop_label, glyph),
            None => format!("[Network] {} {}", cv_label, prop_label),
        };

        let block_network = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_NETWORK_BORDER))
            .border_type(BorderType::Rounded)
            .title(
                Span::styled(
                    title,
                    Style::default()
                        .fg(C_SECTION_LABELS)
                        .add_modifier(Modifier::BOLD),
                )
            );

        frame.render_widget(block_network, chunks[3]);

        // Pass both version and client arrays. UI chooses based on toggle.
        display_network_info(
            &network_info,
            &net_totals,
            frame,
            &version_counts,
            &client_counts,
            &avg_block_propagate_time,
            &propagation_times,
            app.show_client_distribution,
            app.show_propagation_avg,
            chunks[3],
        );
        // -----------------------------------------------------------------------------------------
        // CONSENSUS SECURITY SECTION
        // -----------------------------------------------------------------------------------------
        {
            let consensus_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(C_CONSENSUS_BORDER))
                .border_type(BorderType::Rounded)
                .title(
                    Span::styled(
                        "[Consensus Security]",
                        Style::default()
                            .fg(C_SECTION_LABELS)
                            .add_modifier(Modifier::BOLD),
                    )
                );

            frame.render_widget(consensus_block, chunks[4]);

            // Displays fork info, stale tips, etc.
            display_consensus_security_info(&chaintips_result, frame, chunks[4]);
        }

        // -----------------------------------------------------------------------------------------
        // FOOTER SECTION
        // -----------------------------------------------------------------------------------------
        {
            let footer_msg = if app.is_exiting {
                "Shutting Down Cleanly..."
            } else {
                "Press 'q' to quit | 't' for Tx Lookup | '?' for Help"
            };

            let footer_block = Block::default().borders(Borders::NONE);
            frame.render_widget(footer_block, chunks[5]);

            render_footer(frame, chunks[5], footer_msg);
        }

        // =========================================================================================
        // POPUPS — Conditionals Rendered OVER the Main UI
        // =========================================================================================

        match app.popup {
            PopupType::None => {}, // No overlay needed

            PopupType::TxLookup => {
                render_tx_lookup_popup(frame, &mut app);
            }

            PopupType::Help => {
                render_help_popup(frame, &app);
            }

            PopupType::ConsensusWarning => {
                render_consensus_warning_popup(frame, &app);
            }
        }

    })?; // END terminal.draw()

} // END main loop

// Exit gracefully
Ok(())
} // END run_app



// =================================================================================================
// HELPER: CENTERED POPUP GEOMETRY
// =================================================================================================
/// Computes a centered rectangle sized by percent_x × percent_y of the terminal.
/// Useful for popups, alerts, and overlays.
fn centered_rect(percent_x: u16, percent_y: u16, size: Rect) -> Rect {
    // Vertical split into top, popup, bottom (center region = popup)
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(size);

    // Horizontal split of the popup to center it horizontally
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}



// =================================================================================================
// HELPER: TXID VALIDATION
// =================================================================================================
/// Verifies a string is a valid 64-character hex TxID.
fn is_valid_txid(tx_id: &str) -> bool {
    tx_id.len() == 64 && tx_id.chars().all(|c| c.is_ascii_hexdigit())
}



// =================================================================================================
// POPUP: TX LOOKUP
// =================================================================================================
/// Renders the Transaction Lookup popup overlay.
/// Allows typed or pasted TxID, validates it, and displays RPC result.
fn render_tx_lookup_popup<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let popup_area = centered_rect(80, 28, frame.size());

    // Clear under-popup area so text doesn't bleed through
    frame.render_widget(Clear, popup_area);

    // Outer popup block
    let popup = Block::default()
        .title("Transaction Lookup (Press Esc to go back)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    // User input line
    let input = Paragraph::new(app.tx_input.clone())
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });

    // RPC result rendering
    let result = match &app.tx_result {
        Some(tx) => Paragraph::new(tx.clone())
            .style(Style::default().fg(Color::Green))
            .wrap(Wrap { trim: true }),

        None => {
            if app.tx_input.trim().is_empty() {
                Paragraph::new("Enter a TxID and press Enter")
            } else {
                Paragraph::new("Press Enter to validate TxID")
                    .style(Style::default().fg(Color::Yellow))
            }
        }
    };

    // Draw popup and contents
    frame.render_widget(popup, popup_area);
    frame.render_widget(
        input,
        popup_area.inner(&Margin { vertical: 2, horizontal: 2 }),
    );
    frame.render_widget(
        result,
        popup_area.inner(&Margin { vertical: 5, horizontal: 2 }),
    );
}



// =================================================================================================
// POPUP: HELP PANEL
// =================================================================================================
/// Draws the Help popup showing global shortcuts and section descriptions.
fn render_help_popup<B: Backend>(frame: &mut Frame<B>, _app: &App) {
    let popup_area = centered_rect(80, 35, frame.size());
    frame.render_widget(Clear, popup_area);

    // Multi-line help text
    let help_text = vec![
        "",
        " GLOBAL CONTROLS",
        " ─────────────────────────",
        "  Q     Quit application",
        "  T     Transaction lookup",
        "  ESC   Close panels",
        "",
        " DASHBOARD SECTIONS",
        " ─────────────────────────",
        "  Blockchain   Hashrate Distribution",
        "  Mempool      Mempool Visuals",
        "  Network      Node Versions & Clients",
        "  Consensus    Fork Monitoring",
        "",
        " Toggles are displayed directly inside",
        " each section for clarity.",
        "",
        " Built for the community",
    ];

    let paragraph = Paragraph::new(help_text.join("\n"))
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false });

    let block = Block::default()
        .title("Help (Press Esc to go back)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let container = block.inner(popup_area);

    frame.render_widget(block, popup_area);
    frame.render_widget(paragraph, container);
}



// =================================================================================================
// POPUP: CONSENSUS WARNING
// =================================================================================================
/// Alerts the user when a stale fork of length ≥ 2 is detected.
fn render_consensus_warning_popup<B: Backend>(frame: &mut Frame<B>, _app: &App) {
    let popup_area = centered_rect(70, 20, frame.size());
    frame.render_widget(Clear, popup_area);

    let warning_text = vec![
        "",
        " CONSENSUS WARNING",
        " ─────────────────────────",
        " A stale fork has reached length 2.",
        "",
        " This is unusual and may indicate:",
        "  • network propagation delay",
        "  • isolated miner region",
        "  • temporary chain split",
        "  • delayed block relay",
        "",
        " Press ESC to continue.",
    ];

    let paragraph = Paragraph::new(warning_text.join("\n"))
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false });

    let block = Block::default()
        .title("Consensus Warning (Press Esc to go back)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let container = block.inner(popup_area);

    frame.render_widget(block, popup_area);
    frame.render_widget(paragraph, container);
}
