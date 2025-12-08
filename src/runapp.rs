
// runapp.rs

use crate::config::RpcConfig;
use crate::rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info, fetch_block_data_by_height
    , fetch_chain_tips, fetch_net_totals, fetch_peer_info, fetch_mempool_distribution, fetch_transaction,
    fetch_miner};
use crate::models::errors::MyError;
use crate::display::{display_blockchain_info, display_mempool_info, display_network_info
    , display_consensus_security_info, render_hashrate_distribution_chart};
use crate::utils::{render_header, render_footer, load_miners_data, BLOCK_HISTORY};
use crate::models::peer_info::PeerInfo;
use tui::backend::{CrosstermBackend, Backend};
use tui::layout::{Layout, Constraint, Direction, Margin, Rect, Alignment};
use tui::widgets::{Block, Borders, Paragraph, Clear, Wrap, BorderType};
use tui::style::{Color, Style, Modifier};
use tui::Frame;
use tui::text::{Span, Spans};
use tui::Terminal;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::sleep;
use blockchaininfo::utils::log_error;
use crate::models::chaintips_info::ChainTipsResponse;
// use regex::Regex;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use crate::utils::{BLOCKCHAIN_INFO_CACHE, BLOCK_INFO_CACHE, MEMPOOL_INFO_CACHE, CHAIN_TIP_CACHE, BLOCK24_INFO_CACHE,
PEER_INFO_CACHE, NETWORK_INFO_CACHE, NET_TOTALS_CACHE, MEMPOOL_DISTRIBUTION_CACHE}; // LOGGED_TXS};
use std::sync::Arc;
use tokio::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(PartialEq)]
pub enum PopupType {
    None,
    TxLookup,
    Help,
}
struct App {
    popup: PopupType,
    tx_input: String,
    tx_result: Option<String>,
    is_exiting: bool,
    is_pasting: bool, 
    show_hash_distribution: bool,
    dust_free: Arc<AtomicBool>,
    show_client_distribution: bool, //NEW
}

impl App {
    fn new() -> Self {
        Self {
            popup: PopupType::None,
            tx_input: String::new(),
            tx_result: None,
            is_exiting: false,
            is_pasting: false,
            show_hash_distribution: false,
            dust_free: Arc::new(AtomicBool::new(true)),
            show_client_distribution: false, // default: show version first
        }
    }
}

static LAST_BLOCK_NUMBER: Lazy<DashSet<u64>> = Lazy::new(|| DashSet::new());

/// Sets up the terminal in TUI mode.
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Cleans up the terminal on exit.
pub fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

/// Runs the application logic and keeps the Dashboard alive.
pub async fn run_app<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: &RpcConfig,
) -> Result<(), MyError> {
    let mut propagation_times: VecDeque<i64> = VecDeque::with_capacity(20);
    let mut app = App::new();  
    // Load our new miners json file with wallet addresses.
    let miners_data = load_miners_data()?;
    // Create a longer-lived default value
    let default_miner = "Unknown".to_string();

    terminal.draw(|frame| {
        let area = frame.size();
        let block = Block::default().title("Initializing...").borders(Borders::ALL);
        frame.render_widget(block, area);
    })?;

    // Blockchain info
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                let start = Instant::now();
    
                // Step 1: Fetch Latest Blockchain Info From RPC
                match fetch_blockchain_info(&config_clone).await {
                    Ok(new_blockchain_info) => {
                        let mut cache = BLOCKCHAIN_INFO_CACHE.write().await;
                        // Only update the cache if the data has changed
                        if *cache != new_blockchain_info {
                            *cache = new_blockchain_info; // Update cache
                        } else {
                            // If the data hasn't changed, skip the rest of the loop
                            sleep(Duration::from_secs(2)).await;
                            continue;
                        }
                    }
                    Err(e) => {
                        if let Err(log_err) = log_error(&format!(
                            "Blockchain Info failed: {}", 
                            e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }
    
                // Step 2: Read Blockchain Info from Cache (Now It Exists)
                let block_height = {
                    let blockchain_info = BLOCKCHAIN_INFO_CACHE.read().await;
                    blockchain_info.blocks // Extract the latest block height
                };
    
                // Step 3: Fetch Block Data Using the Latest Block Height (Epoch Start Block)
                match fetch_block_data_by_height(&config_clone, block_height, 1).await {
                    Ok(new_data) => {
                        let mut cache = BLOCK_INFO_CACHE.write().await;
                        if cache.len() >= 1 {
                            cache.remove(0); // Remove the oldest block
                        }
                        cache.push(new_data); // Move the data into the cache
                    }
                    Err(e) => {
                        if let Err(log_err) = log_error(&format!(
                            "Block Data by Height failed at block height {}: {}", 
                            block_height, e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }
    
                // Step 4: Fetch Block Data for 24 Hours Ago
                match fetch_block_data_by_height(&config_clone, block_height, 2).await {
                    Ok(block24_data) => {
                        let mut cache_24 = BLOCK24_INFO_CACHE.write().await;
                        if cache_24.len() >= 1 { 
                            cache_24.remove(0); // Remove the oldest block
                        }
                        cache_24.push(block24_data); // Move the data into the cache
                    }
                    Err(e) => {
                        if let Err(log_err) = log_error(&format!(
                            "Block Data by Height failed at block height {}: {}", 
                            block_height, e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                }
    
                // Dynamic sleep to maintain a 2-second interval
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(2) {
                    sleep(Duration::from_secs(2) - elapsed).await;
                }
            }
        }
    });
        
    // Mempool info
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                let start = Instant::now();
    
                match fetch_mempool_info(&config_clone).await {
                    Ok(new_data) => {
                        let mut cache = MEMPOOL_INFO_CACHE.write().await;
                        if *cache != new_data { // Only update if the data has changed
                            *cache = new_data;
                        }
                    }
                    Err(e) => {
                        if let Err(log_err) = log_error(&format!(
                            "Mempool Info failed: {}", 
                            e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                    }
                }
    
                // Dynamic sleep to maintain a 3-second interval
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(3) {
                    sleep(Duration::from_secs(3) - elapsed).await;
                }
            }
        }
    });

    // Network info
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                let start = Instant::now();
    
                match fetch_network_info(&config_clone).await {
                    Ok(new_data) => {
                        let mut cache = NETWORK_INFO_CACHE.write().await;
                        if *cache != new_data { // Only update if the data has changed
                            *cache = new_data;
                        }
                    }
                    Err(e) => {
                        if let Err(log_err) = log_error(&format!(
                            "Network Info failed: {}",  
                            e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                    }
                }
    
                // Dynamic sleep to maintain a 7-second interval
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(7) {
                    sleep(Duration::from_secs(7) - elapsed).await;
                }
            }
        }
    });

    // Peer info
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                let start = Instant::now();
    
                match fetch_peer_info(&config_clone).await {
                    Ok(new_data) => {
                        let mut cache = PEER_INFO_CACHE.write().await;
                        if *cache != new_data {
                            cache.clear();
                            cache.extend(new_data);
                        }
                    }
                    Err(e) => {
                        if let Err(log_err) = log_error(&format!(
                            "Peer Info failed: {}",  
                            e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                    }
                }
    
                // Dynamic sleep to maintain a 7-second interval
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(7) {
                    sleep(Duration::from_secs(7) - elapsed).await;
                }
            }
        }
    });

    // Chain Tips
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                let start = Instant::now();
    
                match fetch_chain_tips(&config_clone).await {
                    Ok(new_data) => {
                        let mut cache = CHAIN_TIP_CACHE.write().await;
                        let new_response = ChainTipsResponse {
                            error: None,
                            id: None,
                            result: new_data,
                        };
                        if *cache != new_response { // Only update if the data has changed
                            *cache = new_response;
                        }
                    }
                    Err(e) => {
                        if let Err(log_err) = log_error(&format!(
                            "Chain Tips failed: {}",  
                            e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                    }
                }
    
                // Dynamic sleep to maintain a 10-second interval
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(10) {
                    sleep(Duration::from_secs(10) - elapsed).await;
                }
            }
        }
    });

    // Net Totals
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                let start = Instant::now();

                match fetch_net_totals(&config_clone).await {
                    Ok(new_data) => {
                        let mut cache = NET_TOTALS_CACHE.write().await;
                        if *cache != new_data { // Only update if the data has changed
                            *cache = new_data;
                        }
                    }
                    Err(e) => {
                        // Log the error to error_log.txt
                        if let Err(log_err) = log_error(&format!(
                            "Net Totals failed: {}", 
                            e
                        )) {
                            eprintln!("Failed to log error: {}", log_err);
                        }
                    }
                }
                
                 // Dynamic sleep to maintain a 7-second interval
                 let elapsed = start.elapsed();
                 if elapsed < Duration::from_secs(7) {
                     sleep(Duration::from_secs(7) - elapsed).await;
                 }
            }
        }
    });

    // Mempool Distribution
    let dust_flag = app.dust_free.clone();
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            // let txid_regex = Regex::new(r#""([a-fA-F0-9]{64})""#).unwrap(); // Matches 64-char TxID
    
            loop {
                let start = Instant::now();
                let dust_free = dust_flag.load(Ordering::Relaxed);
                if let Err(e) = fetch_mempool_distribution(&config_clone, dust_free).await {
                    let _ = &e;
                    // let error_str = e.to_string();
                   /*
                    // Extract TxID using regex
                    if let Some(captures) = txid_regex.captures(&error_str) {
                        if let Some(txid) = captures.get(1) {
                            let txid_str = txid.as_str().to_string();
    
                            // Check if we've logged this TxID already
                            let logged_txs_read = LOGGED_TXS.read().await;
                            if !logged_txs_read.0.contains(&txid_str) {
                                if let Err(log_err) = log_error(&format!(
                                    "Mempool Distribution failed: {}",  
                                    e
                                )) {
                                    eprintln!("Failed to log error: {}", log_err);
                                }
                                drop(logged_txs_read);
                                let mut logged_txs_write = LOGGED_TXS.write().await;
                                let (set, queue) = &mut *logged_txs_write;
                                if set.len() >= 500 {
                                    if let Some(oldest_tx) = queue.pop_front() {
                                        set.remove(&oldest_tx);
                                    }
                                }
                                // Wrap `tx_id` in an `Arc` for shared ownership
                                let tx_id_arc = Arc::new(txid_str.clone());
                                set.insert(tx_id_arc.clone());
                                queue.push_back(tx_id_arc);
                            }
                        }
                    } */
                }
    
                // Dynamic sleep to maintain a 2-second interval
                let elapsed = start.elapsed();
                if elapsed < Duration::from_secs(2) {
                    sleep(Duration::from_secs(2) - elapsed).await;
                }
            }
        }
    });

    sleep(Duration::from_secs(1)).await;

    loop {
        // Step 1: Fetch all UI data first
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
        
        /*  Adding a progressive Epoch dot which indicates where we are in ths current difficulty cycle.
            The percent variable is passed to render_header() to indicate which circle phase is visible. */
        let into_epoch = blockchain_info.blocks % 2016;
        let percent = (into_epoch as f64 / 2016.0) * 100.0;

        let chaintips_result = &chaintips_info.result;
        let version_counts = PeerInfo::aggregate_and_sort_versions(&peer_info);
        // let version_counts_ref: &[(String, usize)] = &version_counts;
        let client_counts  = PeerInfo::aggregate_and_sort_clients(&peer_info);
        // let client_counts_ref: &[(String, usize)] = &client_counts;
    
        let avg_block_propagate_time = PeerInfo::calculate_block_propagation_time(
            &peer_info,
            blockchain_info.time,
            blockchain_info.blocks,
        );
    
        // Track Propagation Time
        if !LAST_BLOCK_NUMBER.contains(&blockchain_info.blocks) {
            if propagation_times.len() == 20 {
                propagation_times.pop_front();
            }
            propagation_times.push_back(avg_block_propagate_time);
    
            if let Err(e) = fetch_miner(&config, &miners_data, &blockchain_info.blocks).await {
                eprintln!("Error in fetch_miner: {}", e);
            }
    
            LAST_BLOCK_NUMBER.clear(); // Clear the set
            LAST_BLOCK_NUMBER.insert(blockchain_info.blocks); // Insert the new block number
        } else {
            // If the block number hasn't changed but propagation time differs, update the last entry
            if let Some(last_value) = propagation_times.back_mut() {
                if *last_value != avg_block_propagate_time {
                    *last_value = avg_block_propagate_time;
                }
            }
        }
    
        // Read the last miner inserted
        let last_miner = BLOCK_HISTORY.read().await.last_miner();
        let default_miner_arc = Arc::from(default_miner.as_str()); // Convert &str to Arc<str>
        let last_miner_ref = last_miner.as_ref().unwrap_or(&default_miner_arc);
        
        // Convert Vec<(String, u64)> to &[(&str, u64)]
        let hash_distribution: Vec<(Arc<str>, u64)> = BLOCK_HISTORY
            .read()
            .await
            .get_miner_distribution()
            .iter()
            .map(|(miner, hashrate)| (Arc::from(miner.to_string()), *hashrate))
            .collect::<Vec<_>>();

    
        // Dynamic Polling for Smooth Input & CPU Optimization
        let poll_time = if app.popup == PopupType::TxLookup {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(250) // More relaxed updates when idle
        };
    
        // Handle User Input
        if event::poll(poll_time)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc if app.popup != PopupType::None => {
                        app.popup = PopupType::None;
                        app.is_pasting = false;
                    }
                    KeyCode::Char('q') if !app.is_pasting => {
                        app.is_exiting = true; // 🚀 Flag shutdown mode
        
                        // Get terminal size to recompute layout manually
                        let size = terminal.size()?;
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
                            .split(size);
        
                        // Force one last UI update before quitting
                        terminal.draw(|frame| {
                            render_footer(frame, chunks[5], "Shutting Down Cleanly...");
                        })?;
        
                        std::thread::sleep(std::time::Duration::from_millis(500)); // Short delay for visibility
        
                        break; // Quit App
                    }
                    KeyCode::Char('t') if app.popup == PopupType::None => {
                        app.popup = PopupType::TxLookup;
                        app.tx_input.clear();
                        app.tx_result = None;
                        app.is_pasting = false;
                    }
                    KeyCode::Char('?') if app.popup == PopupType::None => {
                        app.popup = PopupType::Help;
                    }
                    KeyCode::Char('h') if app.popup == PopupType::None => {
                        // Toggle the hash distribution flag
                        app.show_hash_distribution = !app.show_hash_distribution;
                    }
                    // Character input (typing or paste) only when Tx Lookup popup is open
                    KeyCode::Char(c) if app.popup == PopupType::TxLookup => {
                        if app.is_pasting {
                            // Ignore special characters during paste
                            if c != 'q' && c != '\n' {
                                app.tx_input.push(c);
                            }
                        } else {
                            // Normal character input
                            app.tx_input.push(c);
                        }

                        // Detect paste start
                        if !app.is_pasting && app.tx_input.len() > 10 {
                            app.is_pasting = true;
                        }
                    }

                    // Backspace inside Tx Lookup
                    KeyCode::Backspace if app.popup == PopupType::TxLookup => {
                        app.tx_input.pop();
                        app.is_pasting = false; // Reset paste flag on backspace
                    }

                    // Enter key inside Tx Lookup
                    KeyCode::Enter if app.popup == PopupType::TxLookup => {
                        if !app.tx_input.is_empty() {
                            if is_valid_txid(&app.tx_input.trim()) {
                                let tx_result = fetch_transaction(&config, &app.tx_input).await;
                                app.tx_result = tx_result.map_or_else(
                                    |e| Some(format!("{}", e)),
                                    Some
                                );
                            } else {
                                app.tx_result = Some("Invalid TxID. Please enter a 64-character hex string.".to_string());
                            }
                            app.is_pasting = false; // Reset paste
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        app.dust_free.store(
                        !app.dust_free.load(Ordering::Relaxed),
                        Ordering::Relaxed
                        );
                     }
                    KeyCode::Char('c') => {
                        app.show_client_distribution = !app.show_client_distribution;
                    }

                    _ => {
                        // Assume paste is complete when a non-char key is pressed
                        if app.is_pasting {
                            app.is_pasting = false;
                        }
                    }
                }
            }
        }
        // Step 2: Draw UI Layout First
        terminal.draw(|frame| {
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
            
            // Header Block
            let block_1 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_1, chunks[0]);
            let header_widget = render_header(percent);
            frame.render_widget(header_widget, chunks[0]);

            // Blockchain Info Block
            let hrd_label = if app.show_hash_distribution {
                // ON — match Mempool's yellow highlight
                Span::styled(
                    "[H] HRD",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                )
            } else {
                // OFF — dark gray, same as inactive Mempool toggle
                Span::styled(
                    "[H] HRD",
                    Style::default().fg(Color::DarkGray)
                )
            };

            // Build the full title as two spans for clean separation
            let blockchain_title = Spans::from(vec![
                Span::styled(
                    "[Blockchain] ",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                ),
                hrd_label,
            ]);

            let block_2 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(blockchain_title);

            frame.render_widget(block_2, chunks[1]);
            
            // println!("DEBUG: block_info length = {}", block_info.len());
            if app.show_hash_distribution {
                render_hashrate_distribution_chart(&hash_distribution, frame, chunks[1]);
            } else {
                if !block_info.is_empty() && !block24_info.is_empty() {
                    let latest_block = &block_info[block_info.len() - 1];  // Safe indexing
                    let block24 = &block24_info[block24_info.len() - 1];  // Safe indexing
                
                    display_blockchain_info(&blockchain_info, latest_block, block24, last_miner_ref, frame, chunks[1]);
                } else {
                    // println!("⚠️ No block info available!");
                }                       
            }
            // Mempool Info Block
            let dust_label = if app.dust_free.load(Ordering::Relaxed) {
                Span::styled(" [D] DUST-FREE", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(" [D] ALL TX", Style::default().fg(Color::DarkGray))
            };

            let mempool_title = Spans(vec![
                Span::styled("[Mempool]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
                dust_label,
            ]);

            let block_3 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(mempool_title);
            frame.render_widget(block_3, chunks[2]);
            display_mempool_info(&mempool_info, &distribution, app.dust_free.load(Ordering::Relaxed), frame, chunks[2]);

           // Network Info Block
            let toggle_label = if app.show_client_distribution {
                "(c→Version)"
            } else {
                "(c→Client)"
            };

            let block_4 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    format!("[Network] {}", toggle_label),
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
                ));

            frame.render_widget(block_4, chunks[3]);
            display_network_info(
                &network_info,
                &net_totals,
                frame,
                &version_counts,
                &client_counts,
                &avg_block_propagate_time,
                &propagation_times,
                app.show_client_distribution,
                chunks[3]
            );

            // Consensus Security Info Block
            let block_5 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[Consensus Security]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_5, chunks[4]);
            display_consensus_security_info(&chaintips_result, frame,  chunks[4]);

            // Footer Block
            let footer_msg = if app.is_exiting {
                "Shutting Down Cleanly..."
            } else {
                "Press 'q' to quit | 't' for Tx Lookup | '?' for Help"
            };

            let block_6 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_6, chunks[5]);
            render_footer(frame, chunks[5], footer_msg);

            match app.popup {
                PopupType::None => {}
                PopupType::TxLookup => {
                    render_tx_lookup_popup(frame, &mut app);
                }
                PopupType::Help => {
                    render_help_popup(frame, &mut app);
                }
            }
        })?;

    }           
    Ok(())
}

/// Helper function to center the popup.
fn centered_rect(percent_x: u16, percent_y: u16, size: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(size);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}


/// Function to validate TxID.
fn is_valid_txid(tx_id: &str) -> bool {
    tx_id.len() == 64 && tx_id.chars().all(|c| c.is_ascii_hexdigit())
}

fn render_tx_lookup_popup<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let popup_area = centered_rect(80, 28, frame.size());
    frame.render_widget(Clear, popup_area);

    let popup = Block::default()
        .title("Transaction Lookup (Press Esc to go back)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let input = Paragraph::new(app.tx_input.clone())
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true });

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

    frame.render_widget(popup, popup_area);
    frame.render_widget(input, popup_area.inner(&Margin { vertical: 2, horizontal: 2 }));
    frame.render_widget(result, popup_area.inner(&Margin { vertical: 5, horizontal: 2 }));
}

fn render_help_popup<B: Backend>(frame: &mut Frame<B>, _app: &App) {
    let popup_area = centered_rect(80, 35, frame.size());
    frame.render_widget(Clear, popup_area);

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
