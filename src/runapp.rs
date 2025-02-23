
// runapp.rs

use crate::config::RpcConfig;
use crate::rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info, fetch_block_data_by_height
    , fetch_chain_tips, fetch_net_totals, fetch_peer_info, fetch_mempool_distribution, fetch_transaction,
    initial_mempool_load};
use crate::models::errors::MyError;
use crate::display::{display_blockchain_info, display_mempool_info, display_network_info
    , display_consensus_security_info};
use crate::utils::{render_header, render_footer};
use crate::models::peer_info::PeerInfo;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction, Margin, Rect};
use tui::widgets::{Block, Borders, Paragraph, Clear, Wrap, BorderType};
use tui::style::{Color, Style, Modifier};
use tui::text::Span;
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
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use crate::utils::{BLOCKCHAIN_INFO_CACHE, BLOCK_INFO_CACHE, MEMPOOL_INFO_CACHE, CHAIN_TIP_CACHE, BLOCK24_INFO_CACHE,
PEER_INFO_CACHE, NETWORK_INFO_CACHE, NET_TOTALS_CACHE, MEMPOOL_DISTRIBUTION_CACHE, LOGGED_TXS};

struct App {
    show_popup: bool,
    tx_input: String,
    tx_result: Option<String>,
    is_exiting: bool,
}

impl App {
    fn new() -> Self {
        Self {
            show_popup: false,
            tx_input: String::new(),
            tx_result: None,
            is_exiting: false,
        }
    }
}

static LAST_BLOCK_NUMBER: Lazy<DashSet<u64>> = Lazy::new(|| DashSet::new());

// Sets up the terminal in TUI mode.
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

// Cleans up the terminal on exit.
pub fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

// Runs the application logic and keeps the TUI alive.
pub async fn run_app<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: &RpcConfig,
) -> Result<(), MyError> {
    
    // let distribution = Arc::new(AsyncMutex::new(MempoolDistribution::default()));
     // Lock block number tracking
    // let mut last_known_block_number = LAST_BLOCK_NUMBER.lock().await;
    let mut propagation_times: VecDeque<i64> = VecDeque::with_capacity(20);
    let mut app = App::new();  
    
    // Create a flag to track initial load completion
    let initial_load_complete = Arc::new(AtomicBool::new(false));

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
                // Step 1: Fetch Latest Blockchain Info From RPC
                if let Ok(blockchain_info) = fetch_blockchain_info(&config_clone).await {
                    *BLOCKCHAIN_INFO_CACHE.write().await = blockchain_info;  // Update cache
                } else {
                    // println!("❌ Failed to fetch blockchain info. Retrying...");
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }

                // Step 2: Read Blockchain Info from Cache (Now It Exists)
                let block_height = {
                    let blockchain_info = BLOCKCHAIN_INFO_CACHE.read().await;
                    blockchain_info.blocks // Extract the latest block height
                };

                // Step 3: Fetch Block Data Using the Latest Block Height (Epoch Start Block)
                if let Ok(new_data) = fetch_block_data_by_height(&config_clone, block_height, 1).await {
                    let mut cache = BLOCK_INFO_CACHE.write().await;
                
                    // Maintain last 10 blocks to prevent infinite growth
                    if cache.len() >= 10 {
                        cache.remove(0); // Remove the oldest block
                    }
                
                    let new_data_clone = new_data.clone();  // Clone it before pushing
                    cache.push(new_data_clone);
                
                    // println!("✅ BlockInfo Updated: Height = {}", new_data_clone.height);
                } else {
                    // println!("❌ Failed to fetch block data. Retrying...");
                }

                // **Step 4: Fetch Block Data for 24 Hours Ago**
                if let Ok(block24_data) = fetch_block_data_by_height(&config_clone, block_height, 2).await {
                    let mut cache_24 = BLOCK24_INFO_CACHE.write().await;
                    
                    // Maintain last 10 blocks to prevent infinite growth
                    if cache_24.len() >= 10 {
                        cache_24.remove(0); // Remove the oldest block
                    }
                    
                    let block24_clone = block24_data.clone();  // Clone before pushing
                    cache_24.push(block24_clone);
                    
                    // println!("✅ Block24Info Updated: Height = {}", block24_clone.height);
                } else {
                    // println!("❌ Failed to fetch 24-hour block data. Retrying...");
                }

                sleep(Duration::from_secs(2)).await;
            }
        }
    });
     
    
    // Mempool info
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                if let Ok(new_data) = fetch_mempool_info(&config_clone).await {
                    *MEMPOOL_INFO_CACHE.write().await = new_data;
                }
                sleep(Duration::from_secs(3)).await;
            }
        }
    });

    // Network info
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                if let Ok(new_data) = fetch_network_info(&config_clone).await {
                    *NETWORK_INFO_CACHE.write().await = new_data;
                }
                sleep(Duration::from_secs(7)).await;
            }
        }
    });

    // Peer info
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                if let Ok(new_data) = fetch_peer_info(&config_clone).await {
                    *PEER_INFO_CACHE.write().await = new_data;
                }
                sleep(Duration::from_secs(7)).await;
            }
        }
    });

    // Chain Tips
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                if let Ok(new_data) = fetch_chain_tips(&config_clone).await {
                    *CHAIN_TIP_CACHE.write().await = ChainTipsResponse {
                        error: None,  // Adjust if error handling is needed
                        id: None,     // Adjust if an ID is required
                        result: new_data,  // Wrap `Vec<ChainTip>` inside `ChainTipsResponse`
                    };
                    
                }
                sleep(Duration::from_secs(10)).await;
            }
        }
    });

    // Net Totals
    tokio::spawn({
        let config_clone = config.clone();
        async move {
            loop {
                if let Ok(new_data) = fetch_net_totals(&config_clone).await {
                    *NET_TOTALS_CACHE.write().await = new_data;
                }
                sleep(Duration::from_secs(7)).await;
            }
        }
    });

    // Mempool Distribution
    tokio::spawn({
        let config_clone = config.clone();
        let initial_load_complete = Arc::clone(&initial_load_complete);
        async move {
            let txid_regex = Regex::new(r#""([a-fA-F0-9]{64})""#).unwrap(); // Matches 64-char TxID

            // Step 1: Initial Load (Batch RPC)
            if let Err(e) = initial_mempool_load(&config_clone).await {
                let error_str = e.to_string();

                // Extract TxID using regex
                if let Some(captures) = txid_regex.captures(&error_str) {
                    if let Some(txid) = captures.get(1) {
                        let txid_str = txid.as_str().to_string();

                        // Check if we've logged this TxID already
                        let logged_txs_read = LOGGED_TXS.read().await;
                        if !logged_txs_read.contains(&txid_str) {
                            log_error(&format!("Initial Mempool Load failed for TxID: {}", txid_str));
                            drop(logged_txs_read);
                            let mut logged_txs_write = LOGGED_TXS.write().await;
                            logged_txs_write.insert(txid_str); // Mark as logged
                        }
                    }
                }
            }

            // Mark initial load as complete
            initial_load_complete.store(true, Ordering::Relaxed);

            // Step 2: Main Loop (Incremental Updates)
            loop {
                // Clone the Arc for each iteration
                let initial_load_complete_clone = Arc::clone(&initial_load_complete);
                if let Err(e) = fetch_mempool_distribution(&config_clone, initial_load_complete_clone).await {
                    let error_str = e.to_string();

                    // Extract TxID using regex
                    if let Some(captures) = txid_regex.captures(&error_str) {
                        if let Some(txid) = captures.get(1) {
                            let txid_str = txid.as_str().to_string();

                            // Check if we've logged this TxID already
                            let logged_txs_read = LOGGED_TXS.read().await;
                            if !logged_txs_read.contains(&txid_str) {
                                log_error(&format!("Mempool Distribution failed for TxID: {}", txid_str));
                                drop(logged_txs_read);
                                let mut logged_txs_write = LOGGED_TXS.write().await;
                                logged_txs_write.insert(txid_str); // Mark as logged
                            }
                        }
                    }
                }
                sleep(Duration::from_secs(2)).await;
            }
        }
    });

    sleep(Duration::from_secs(1)).await;

    loop {

        // Step 1: Fetch all UI data first
        let blockchain_info = BLOCKCHAIN_INFO_CACHE.read().await;
        let mempool_info = MEMPOOL_INFO_CACHE.read().await;
        let network_info = NETWORK_INFO_CACHE.read().await;
        let peer_info = PEER_INFO_CACHE.read().await;
        let block_info = BLOCK_INFO_CACHE.read().await;
        let block24_info = BLOCK24_INFO_CACHE.read().await;
        let net_totals = NET_TOTALS_CACHE.read().await;
        let distribution = MEMPOOL_DISTRIBUTION_CACHE.read().await;
        let chaintips_info = CHAIN_TIP_CACHE.read().await;
        let chaintips_result = &chaintips_info.result;
        
        let version_counts = PeerInfo::aggregate_and_sort_versions(&peer_info);
        let version_counts_ref: &[(String, usize)] = &version_counts;
       
        let avg_block_propagate_time = PeerInfo::calculate_block_propagation_time(
                &peer_info,
                blockchain_info.time,
                blockchain_info.blocks
        );

       // Track Propagation Time
       if !LAST_BLOCK_NUMBER.contains(&blockchain_info.blocks) {
            if propagation_times.len() == 20 {
                propagation_times.pop_front();
            }
            propagation_times.push_back(avg_block_propagate_time);
            LAST_BLOCK_NUMBER.insert(blockchain_info.blocks); // Insert the new block number
        } else {
            // If the block number hasn't changed but propagation time differs, update the last entry
            if let Some(last_value) = propagation_times.back_mut() {
                if *last_value != avg_block_propagate_time {
                    *last_value = avg_block_propagate_time;
                }
            }
        }
    
        // Dynamic Polling for Smooth Input & CPU Optimization
        let poll_time = if app.show_popup {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(250) // More relaxed updates when idle
        };
    
        // Handle User Input
        if event::poll(poll_time)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc if app.show_popup => app.show_popup = false, // Close Popup
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.is_exiting = true;  // 🚀 Flag shutdown mode
                    
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
                    
                        std::thread::sleep(std::time::Duration::from_millis(500));  // Short delay for visibility
                    
                        break;  // Quit App
                    },
                    KeyCode::Char('t') if !app.show_popup => {
                        app.show_popup = true;
                        app.tx_input.clear();
                        app.tx_result = None;
                    }
                    KeyCode::Char(c) if app.show_popup => app.tx_input.push(c),  // Typing input
                    KeyCode::Backspace if app.show_popup => { app.tx_input.pop(); } // Delete input
                    KeyCode::Enter if app.show_popup => {
                        if !app.tx_input.is_empty() {
                            let tx_result = fetch_transaction(&config, &app.tx_input).await;
                            app.tx_result = tx_result.map_or_else(
                                |e| Some(format!("Error: {:?}", e)),
                                Some
                            );
                        }
                    }
                    _ => {}
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
            let header_widget = render_header();
            frame.render_widget(header_widget, chunks[0]);

            // Blockchain Info Block
            let block_2 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[₿lockChain]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_2, chunks[1]);
            
            // println!("DEBUG: block_info length = {}", block_info.len());

            if !block_info.is_empty() && !block24_info.is_empty() {
                let latest_block = &block_info[block_info.len() - 1];  // Safe indexing
                let block24 = &block24_info[block24_info.len() - 1];  // Safe indexing
            
                display_blockchain_info(&blockchain_info, latest_block, block24, frame, chunks[1]);
            } else {
                // println!("⚠️ No block info available!");
            }                       

            // Mempool Info Block
            let block_3 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[Mempool]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_3, chunks[2]);
            display_mempool_info(&mempool_info, &distribution, frame, chunks[2]);

            // Network Info Block
            let block_4 = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[Network]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_4, chunks[3]);
            display_network_info(
                &network_info,
                &net_totals,
                frame,
                &version_counts_ref,
                &avg_block_propagate_time,
                &propagation_times,
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
                "Press 'q' to quit | 't' for Tx Lookup"
            };

            let block_6 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_6, chunks[5]);
            render_footer(frame, chunks[5], footer_msg);

            // Popup for Transaction Lookup
            if app.show_popup {
                let popup_area = centered_rect(75, 27, frame.size());  

                // Clear the area to avoid text overlapping
                frame.render_widget(Clear, popup_area);

                // Popup UI Block
                let popup = Block::default()
                    .title("Transaction Lookup (Press Esc to go back)")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Yellow));

                // User Input Field
                let input = Paragraph::new(app.tx_input.clone())
                    .style(Style::default().fg(Color::Cyan))
                    .wrap(Wrap { trim: true });

                // Transaction Result Display
                let result = match &app.tx_result {
                    Some(tx) => Paragraph::new(tx.clone())
                        .style(Style::default().fg(Color::Green))
                        .wrap(Wrap { trim: true }),  // Ensures multi-line text fits within the popup
                    None => Paragraph::new("Enter a TxID and press Enter")
                        .style(Style::default()),
                };

                frame.render_widget(popup, popup_area);
                frame.render_widget(input, popup_area.inner(&Margin { vertical: 2, horizontal: 2 }));
                frame.render_widget(result, popup_area.inner(&Margin { vertical: 5, horizontal: 2 }));
            }
        })?;

    }           
    Ok(())
}

// Helper function to center the popup
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

