
// runapp.rs

use crate::config::BitcoinRpcConfig;
use crate::rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info, fetch_block_data_by_height
    , fetch_chain_tips, fetch_net_totals, fetch_peer_info, fetch_mempool_distribution, fetch_transaction};
use crate::models::errors::MyError;
use crate::display::{display_blockchain_info, display_mempool_info, display_network_info
    , display_consensus_security_info};
use crate::utils::{DIFFICULTY_ADJUSTMENT_INTERVAL, render_header, render_footer};
use crate::models::peer_info::PeerInfo;
use tokio::try_join;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction, Margin, Rect};
use tui::widgets::{Block, Borders, BorderType, Paragraph, Clear, Wrap};
use tui::Terminal;
use tui::text::Span;
use tui::style::{Color, Style, Modifier};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use crate::models::mempool_info::MempoolDistribution;
use std::collections::VecDeque;

struct App {
    show_popup: bool,
    tx_input: String,
    tx_result: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            show_popup: false,
            tx_input: String::new(),
            tx_result: None,
        }
    }
}

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
    config: &BitcoinRpcConfig,
) -> Result<(), MyError> {
    let distribution = Arc::new(AsyncMutex::new(MempoolDistribution::default()));
    let mut last_known_block_number: u64 = 0;
    let mut propagation_times: VecDeque<i64> = VecDeque::with_capacity(20);
    let mut app = App::new();  

    terminal.draw(|frame| {
        let area = frame.size();
        let block = Block::default().title("Initializing...").borders(Borders::ALL);
        frame.render_widget(block, area);
    })?;

    loop {
        // Fetch blockchain info first since `blocks` is needed for the next call.
        let blockchain_info = fetch_blockchain_info(&config.bitcoin_rpc).await?;
            
        let distribution_clone = Arc::clone(&distribution);
        let epoc_start_block = (
            (blockchain_info.blocks - 1) / DIFFICULTY_ADJUSTMENT_INTERVAL
        ) * DIFFICULTY_ADJUSTMENT_INTERVAL;
    
        // Fetch everything else only if the popup is NOT open
        let (mempool_info, network_info, block_info, chaintips_info,
            net_totals, peer_info) = try_join!(
            fetch_mempool_info(&config.bitcoin_rpc),
            fetch_network_info(&config.bitcoin_rpc),
            fetch_block_data_by_height(&config.bitcoin_rpc, epoc_start_block),
            fetch_chain_tips(&config.bitcoin_rpc),
            fetch_net_totals(&config.bitcoin_rpc),
            fetch_peer_info(&config.bitcoin_rpc) 
        )?;
    
        let version_counts = PeerInfo::aggregate_and_sort_versions(&peer_info);
        let avg_block_propagate_time = PeerInfo::calculate_block_propagation_time(&peer_info, blockchain_info.time,
            blockchain_info.blocks);
    
        if blockchain_info.blocks != last_known_block_number {
            if propagation_times.len() == 20 {
                propagation_times.pop_front();
            }
            propagation_times.push_back(avg_block_propagate_time);
            last_known_block_number = blockchain_info.blocks;
        }
    
        tokio::spawn({
            let config_clone = config.bitcoin_rpc.clone();
            async move {
                if let Ok(((small, medium, large), (young, moderate, old), (rbf, non_rbf), 
                    average_fee, median_fee, average_fee_rate)) =
                    fetch_mempool_distribution(&config_clone, last_known_block_number).await
                {
                    let mut dist = distribution_clone.lock().await;
                    dist.small = small;
                    dist.medium = medium;
                    dist.large = large;
                    dist.young = young;
                    dist.moderate = moderate;
                    dist.old = old;
                    dist.rbf_count = rbf;
                    dist.non_rbf_count = non_rbf;
                    dist.average_fee = average_fee;
                    dist.median_fee = median_fee;
                    dist.average_fee_rate = average_fee_rate;
                }
            }
        });       
        
        // Lock the Mutex to access MempoolDistribution.
        let dist = distribution.lock().await;
        
        // Reduce `poll()` time when the popup is open for real-time input
        let poll_time = if app.show_popup {
            std::time::Duration::from_millis(100)  // Faster polling when typing
        } else {
            std::time::Duration::from_secs(5)  // Slow refresh when idle
        };

        if event::poll(poll_time)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Close Popup (if open)
                    KeyCode::Esc if app.show_popup => {
                        app.show_popup = false;
                    }

                    // Quit application (only if popup is NOT open)
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }

                    // Open Transaction Lookup Popup
                    KeyCode::Char('t') if !app.show_popup => {
                        app.show_popup = true;
                        app.tx_input.clear();  // Clear input field
                        app.tx_result = None;  // Clear previous transaction result
                    }

                    // Capture user input inside the popup
                    KeyCode::Char(c) if app.show_popup => {
                        app.tx_input.push(c);
                    }

                    // Backspace inside Popup
                    KeyCode::Backspace if app.show_popup => {
                        app.tx_input.pop();
                    }

                    // Fetch Transaction when Enter is pressed inside Popup
                    KeyCode::Enter if app.show_popup => {
                        if !app.tx_input.is_empty() {
                            let tx_result = fetch_transaction(&config.bitcoin_rpc, &app.tx_input).await;
                    
                            app.tx_result = match tx_result {
                                Ok(tx) => Some(tx),
                                Err(e) => {
                                    println!("❌ Error fetching transaction: {:?}", e); // Print the actual error
                                    Some(format!("Error: {:?}", e)) // Show the error message in the popup
                                }
                            };
                    
                            // Force UI refresh after fetching the transaction
                            terminal.draw(|frame| {
                                let popup_area = centered_rect(65, 27, frame.size());  // ✅ Ensure enough height
                                let popup = Block::default()
                                    .title("Transaction Lookup (Press Esc to go back)")
                                    .borders(Borders::ALL)
                                    .style(Style::default().fg(Color::Yellow));
                    
                                let input = Paragraph::new(app.tx_input.clone())
                                    .style(Style::default().fg(Color::Cyan))
                                    .wrap(Wrap { trim: true });  
                    
                                let result = match &app.tx_result {
                                    Some(tx) => Paragraph::new(tx.clone()).style(Style::default().fg(Color::Green)).wrap(Wrap { trim: true }),
                                    None => Paragraph::new("Fetching transaction...").style(Style::default()),
                                };
                    
                                frame.render_widget(Clear, popup_area);
                                frame.render_widget(popup, popup_area);
                                frame.render_widget(input, popup_area.inner(&Margin { vertical: 2, horizontal: 2 }));
                                frame.render_widget(result, popup_area.inner(&Margin { vertical: 5, horizontal: 2 }));
                            })?;
                        }
                    }                                                                                
                    _ => {}
                }
            }
            // Additional check to detect pasting behavior
            if event::poll(std::time::Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char(c) = key.code {
                        app.tx_input.push(c);
                        // needs_redraw = true;
                    }
                }
            }
        }

        // Only ONE `terminal.draw()` call per loop iteration
        terminal.draw(|frame| {
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
                .split(frame.size());

            // Render the main dashboard first
            let block_1 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_1, chunks[0]);
            let header_widget = render_header();
            frame.render_widget(header_widget, chunks[0]);

            let block_2 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[₿lockChain]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_2, chunks[1]);
            display_blockchain_info(frame, &blockchain_info, &block_info, chunks[1]).unwrap();

            let block_3 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[Mempool]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_3, chunks[2]);
            display_mempool_info(frame, &mempool_info, &dist, chunks[2]).unwrap();

            let block_4 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[Network]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_4, chunks[3]);
            display_network_info(frame, &network_info, &net_totals, &version_counts, &avg_block_propagate_time, 
                &propagation_times, chunks[3]).unwrap();

            let block_5 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled("[Consensus Security]", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)));
            frame.render_widget(block_5, chunks[4]);
            display_consensus_security_info(frame, &chaintips_info, chunks[4]).unwrap();

            let block_6 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_6, chunks[5]);
            render_footer(frame, chunks[5]);

            // Render the popup OVER the main dashboard
            if app.show_popup {
                let popup_area = centered_rect(65, 27, frame.size());  // Increased height from 15 → 20
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
                            .wrap(Wrap { trim: true }),  // Ensures multi-line text fits within the popup
                        None => Paragraph::new("Enter a TxID and press Enter")
                            .style(Style::default()),
                    };
            
                frame.render_widget(Clear, popup_area);
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

