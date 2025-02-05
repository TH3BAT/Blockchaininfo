
// runapp.rs

use crate::config::BitcoinRpcConfig;
use crate::rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info, fetch_block_data_by_height
    , fetch_chain_tips, fetch_net_totals, fetch_peer_info, fetch_mempool_distribution};
use crate::models::errors::MyError;
use crate::display::{display_blockchain_info, display_mempool_info, display_network_info
    , display_consensus_security_info};
use crate::utils::{DIFFICULTY_ADJUSTMENT_INTERVAL, render_header, render_footer};
use crate::models::peer_info::PeerInfo;
use tokio::try_join;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction};
use tui::widgets::{Block, Borders, BorderType};
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

    loop {
        // Fetch blockchain info first since `blocks` is needed for the next call.
        let blockchain_info = fetch_blockchain_info(&config.bitcoin_rpc).await?;

        // Extract the block height from BlockchainInfo.
        let epoc_start_block = (
            (blockchain_info.blocks - 1) / DIFFICULTY_ADJUSTMENT_INTERVAL
        ) * DIFFICULTY_ADJUSTMENT_INTERVAL;

        // Concurrently fetch mempool info, network info, block info, and chain tips.
        let ((mempool_info, sample_ids), network_info, block_info, chaintips_info, net_totals, peer_info) = try_join!(
            fetch_mempool_info(&config.bitcoin_rpc, 5.0),
            fetch_network_info(&config.bitcoin_rpc),
            fetch_block_data_by_height(&config.bitcoin_rpc, epoc_start_block),
            fetch_chain_tips(&config.bitcoin_rpc),
            fetch_net_totals(&config.bitcoin_rpc),
            fetch_peer_info(&config.bitcoin_rpc) 
        )?;

        let version_counts = PeerInfo::aggregate_and_sort_versions(&peer_info);
        let avg_block_propagate_time = PeerInfo::calculate_block_propagation_time(&peer_info, blockchain_info.time, blockchain_info.blocks);
        if blockchain_info.blocks != last_known_block_number {
            if propagation_times.len() == 20 {
                propagation_times.pop_front();
            }
            propagation_times.push_back(avg_block_propagate_time);
            last_known_block_number = blockchain_info.blocks;
        }

        tokio::spawn({
            let config_clone = config.bitcoin_rpc.clone();
            let distribution_clone = distribution.clone();
            async move {
                if let Ok(((small, medium, large), (young, moderate, old), (rbf, non_rbf), 
                    average_fee, median_fee, average_fee_rate)) =
                    fetch_mempool_distribution(&config_clone, sample_ids).await
                {
                    let mut dist = distribution_clone.lock().await;
                    // Update size distribution.
                    dist.small = small;
                    dist.medium = medium;
                    dist.large = large;
                    // Update age distribution.
                    dist.young = young;
                    dist.moderate = moderate;
                    dist.old = old;
                    // Update RBF stats.
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
        
        // Draw the TUI.
        terminal.draw(|frame| {
            // Define the layout with updated sections.
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),   // Application title.
                        Constraint::Length(14),  // Blockchain section.
                        Constraint::Length(25),   // Mempool section.
                        Constraint::Max(18),     // Network section.
                        Constraint::Length(7),   // Consensus Security section.
                        Constraint::Length(1),   // Footer section.
                    ]
                    .as_ref(),
                )
                .split(frame.size());
            
            // Block 1: App title.
            let block_1 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_1, chunks[0]);
            let header_widget = render_header(); // Create header widget.
            frame.render_widget(header_widget, chunks[0]); // Render the header widget.

            // Block 2: Blockchain Info.
            let block_2 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                "[₿lockChain]",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD), 
            ));
            frame.render_widget(block_2, chunks[1]);
            display_blockchain_info(frame, &blockchain_info, &block_info, chunks[1]).unwrap();
        
            // Block 3: Mempool Info.
            let block_3 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                "[Mempool]",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD), 
            ));            
            frame.render_widget(block_3, chunks[2]);
            display_mempool_info(frame, &mempool_info, &dist, chunks[2]).unwrap();
        
            // Block 4: Network Info.
            let block_4 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                "[Network]",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD), 
            ));
            frame.render_widget(block_4, chunks[3]);
            display_network_info(frame, &network_info, &net_totals, &version_counts, &avg_block_propagate_time, 
                &propagation_times, chunks[3]).unwrap();

            // Block 5: Consensus Security.
            let block_5 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                "[Consensus Security]",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ));
            frame.render_widget(block_5, chunks[4]);
            display_consensus_security_info(frame, &chaintips_info, chunks[4]).unwrap();

            // Block 6: Footer.
            let block_6 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_6, chunks[5]);
            render_footer(frame, chunks[5]);
        })?;

        // Exit the loop if 'q' or 'Esc' is pressed.
        if event::poll(std::time::Duration::from_millis(10000))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break; // Allow quitting with 'q' or Escape key.
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
