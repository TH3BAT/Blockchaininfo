
// main.rs

mod config;
mod rpc;
mod models;
mod utils;
mod display;

use config::{load_config, BitcoinRpcConfig};
use rpc::{fetch_blockchain_info, fetch_mempool_info, fetch_network_info, fetch_block_data_by_height
    , fetch_chain_tips, fetch_net_totals, fetch_peer_info};
use models::errors::MyError;
use display::{display_blockchain_info, display_mempool_info, display_network_info
    , display_consensus_security_info};
use crate::utils::{DIFFICULTY_ADJUSTMENT_INTERVAL, display_header_widget};
use tokio::try_join;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction};
use tui::widgets::{Block, Borders};
use tui::Terminal;
use tui::text::Span;
use tui::style::{Color, Style, Modifier};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use utils::{render_footer, aggregate_and_sort_versions};

#[tokio::main]
async fn main() -> Result<(), MyError> {
    // Parse and load RPC configuration or environment variables to connect to node.
    let config_file = "config.toml";
    let config = load_config(config_file)?;

    if config.bitcoin_rpc.username.is_empty()
        || config.bitcoin_rpc.password.is_empty()
        || config.bitcoin_rpc.address.is_empty()
    {
        return Err(MyError::Config("Invalid config data".to_string()));
    }

    // Setup terminal in TUI mode.
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, &config).await;

    // Clean up terminal.
    cleanup_terminal(&mut terminal)?;

    result
}

// Sets up the terminal in TUI mode.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

// Cleans up the terminal on exit.
fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

// Runs the application logic and keeps the TUI alive.
async fn run_app<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    config: &BitcoinRpcConfig,
) -> Result<(), MyError> {
    loop {
        // Fetch blockchain info first since `blocks` is needed for the next call.
        let blockchain_info = fetch_blockchain_info(&config.bitcoin_rpc).await?;

        // Extract the block height from BlockchainInfo.
        let epoc_start_block = (
            (blockchain_info.blocks - 1) / DIFFICULTY_ADJUSTMENT_INTERVAL
        ) * DIFFICULTY_ADJUSTMENT_INTERVAL;

        // Concurrently fetch mempool info, network info, block info, and chain tips.
        let (mempool_info, network_info, block_info, chaintips_info, net_totals, peer_info) = try_join!(
            fetch_mempool_info(&config.bitcoin_rpc),
            fetch_network_info(&config.bitcoin_rpc),
            fetch_block_data_by_height(&config.bitcoin_rpc, epoc_start_block),
            fetch_chain_tips(&config.bitcoin_rpc),
            fetch_net_totals(&config.bitcoin_rpc),
            fetch_peer_info(&config.bitcoin_rpc) 
        )?;

        let version_counts = aggregate_and_sort_versions(&peer_info);
        // let sorted_peers = sort_peers(&peer_info);

        // Draw the TUI.
        terminal.draw(|frame| {
            // Define the layout with updated sections.
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(8),   // Application title.
                        Constraint::Length(14),  // Blockchain section.
                        Constraint::Length(7),   // Mempool section.
                        Constraint::Max(17),     // Network section.
                        Constraint::Length(7),   // Consensus Security section.
                        Constraint::Length(1),   // Footer section.
                    ]
                    .as_ref(),
                )
                .split(frame.size());
            
            // Block 1: App title.
            let block_1 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_1, chunks[0]);
            let header_widget = display_header_widget(); // Create header widget.
            frame.render_widget(header_widget, chunks[0]); // Render the header widget.

            // Block 2: Blockchain Info.
            let block_2 = Block::default().borders(Borders::NONE).title(Span::styled(
                "[Blockchain]",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED), 
            ));
            frame.render_widget(block_2, chunks[1]);
            display_blockchain_info(frame, &blockchain_info, &block_info, chunks[1]).unwrap();
        
            // Block 3: Mempool Info.
            let block_3 = Block::default().borders(Borders::NONE).title(Span::styled(
                "[Mempool]",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED), 
            ));
            frame.render_widget(block_3, chunks[2]);
            display_mempool_info(frame, &mempool_info, chunks[2]).unwrap();
        
            // Block 4: Network Info.
            let block_4 = Block::default().borders(Borders::NONE).title(Span::styled(
                "[Network]",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED), 
            ));
            frame.render_widget(block_4, chunks[3]);
            display_network_info(frame, &network_info, &net_totals, &version_counts, chunks[3]).unwrap();

            // Block 5: Consensus Security.
            display_consensus_security_info(frame, &chaintips_info, chunks[4]).unwrap();

            // Block 6: Footer.
            let block_6 = Block::default().borders(Borders::NONE);
            frame.render_widget(block_6, chunks[5]);
            render_footer(frame, chunks[5]);
        })?;

        // Exit the loop if 'q' or 'Esc' is pressed, or Ctrl+C is detected.
        if event::poll(std::time::Duration::from_millis(3000))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break; // Allow quitting with 'q' or Escape key.
                    }
                    KeyCode::Char('c') => {
                        break; // Allow quitting with Ctrl+C.
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
