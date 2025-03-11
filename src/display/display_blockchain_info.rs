
// display/display_blockchain_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph},
    Frame,
};
use num_format::{Locale, ToFormattedString};
use crate::{models::{block_info::BlockInfo, blockchain_info::BlockchainInfo}, 
    utils::{estimate_difficulty_change, estimate_24h_difficulty_change, format_size}};
use crate::models::errors::MyError;  
use crate::models::flashing_text::{BEST_BLOCK_TEXT, MINER_TEXT};

// Render the blockchain info into a `tui` terminal UI.
pub fn display_blockchain_info<B: Backend>(
    blockchain_info: &BlockchainInfo,
    block_info: &BlockInfo,
    block24_info: &BlockInfo,
    last_miner: &String,
    frame: &mut Frame<B>,
    area: Rect
) -> Result<(), MyError> {
        
    let mediantime = blockchain_info.parse_mediantime()?;
    let time = blockchain_info.parse_time()?;
    let formatted_size_on_disk = format_size(blockchain_info.size_on_disk);
    let time_since_block = blockchain_info.calculate_time_diff()?;
    let formatted_difficulty = blockchain_info.formatted_difficulty()?;
    let formatted_chainwork_bits = blockchain_info.formatted_chainwork_bits()?;

    let estimate_difficulty_chng = estimate_difficulty_change(
        blockchain_info.blocks,
        blockchain_info.time,
        block_info.time,  
    );

    let difficulty_change_display = if block_info.confirmations < 6 {
        // Display "N/A" for the first 6 blocks
        Span::styled(
            " N/A ",
            Style::default().fg(Color::Gray),
        )
    } else {
        // Display the formatted percentage
        Span::styled(
            format!(" {:.2}% ", estimate_difficulty_chng.abs()),
            Style::default().fg(Color::Gray),
        )
    };
    
    // New difficulty change estimate (24-hour window)
    let estimate_24h_difficulty_chng = estimate_24h_difficulty_change(
        blockchain_info.time,  // Latest block timestamp
        block24_info.time,     // Timestamp from 24-hour block
    );
    
    // Difficulty arrow.
    let difficulty_arrow = if block_info.confirmations < 6 {
        // No arrow in N/A mode
        " ".to_string()
    } else if estimate_difficulty_chng > 0.0 {
        // Up arrow for positive change
        "↑".to_string()
    } else {
        // Down arrow for negative change
        "↓".to_string()
    };


    // Difficulty arrow for 24-hour estimate
    let difficulty_arrow_24h = if estimate_24h_difficulty_chng > 0.0 {
        "↑".to_string()
    } else {
        "↓".to_string()
    };

    // Update the FlashingText variable
    BEST_BLOCK_TEXT.lock().unwrap().update(blockchain_info.blocks);
    MINER_TEXT.lock().unwrap().update(last_miner.to_string());

    // Get the style for the FlashingText
    let best_block_style = BEST_BLOCK_TEXT.lock().unwrap().style();
    let last_miner_style = MINER_TEXT.lock().unwrap().style();

    // Create the Spans with the updated style
    let best_block_spans = Spans::from(vec![
        Span::styled(
            "🏆 Best Block: ",
            Style::default().fg(Color::Gray), // Static style for the label
        ),
        Span::styled(
            blockchain_info.blocks.to_formatted_string(&Locale::en),
            best_block_style, // Dynamic style for the value
        ),
        Span::styled(
            " | ",
            Style::default().fg(Color::DarkGray), 
        ),
        Span::styled(
            "⛏️ Miner: ",
            Style::default().fg(Color::Gray), 
        ),
        Span::styled(
            format!("{}", last_miner), 
            last_miner_style, 
        ),
    ]);

    // Build the blockchain info text before using it.
    let blockchain_info_text = vec![
        Spans::from(vec![
            Span::styled("🔗 Chain: ", Style::default().fg(Color::Gray)),
            Span::styled(blockchain_info.chain.clone(), Style::default().fg(Color::Yellow)),
        ]),
        best_block_spans, 
        
        Spans::from(vec![
            Span::styled("  ⏳ Time since block: ", Style::default().fg(Color::Gray)),
            Span::styled(time_since_block, Style::default().fg(Color::Red)),
        ]),

        Spans::from(vec![
            Span::styled("🎯 Difficulty: ", Style::default().fg(Color::Gray)),
            Span::styled(formatted_difficulty, Style::default().fg(Color::LightRed)),
        ]),

        Spans::from(vec![
            Span::styled("     Blocks until adjustment: ", Style::default().fg(Color::Gray)),
            match blockchain_info.display_blocks_until_difficulty_adjustment() {
                Ok((block_text, block_color)) => Span::styled(block_text, Style::default().fg(block_color)),
                Err(e) => Span::styled(format!("Error: {}", e), Style::default().fg(Color::Red)),
            },
        ]),
    
        Spans::from(vec![
            Span::styled("  📉 Estimated change: ", Style::default().fg(Color::Gray)),
        
            // Epoch-based difficulty change
            Span::styled(
                difficulty_arrow,
                Style::default().fg(if estimate_difficulty_chng > 0.0 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            difficulty_change_display,
            
            Span::styled("(epoch)", Style::default().fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)),
        
            // Separator
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        
            // 24-hour difficulty change
            Span::styled(
                difficulty_arrow_24h,
                Style::default().fg(if estimate_24h_difficulty_chng > 0.0 {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::styled(
                format!(" {:.2}% ", estimate_24h_difficulty_chng.abs()),
                Style::default().fg(Color::Gray),
            ),
            Span::styled("(24hrs)", Style::default().fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)),
        ]),        

        Spans::from(vec![
            Span::styled("   Chainwork: ", Style::default().fg(Color::Gray)),
            Span::styled(formatted_chainwork_bits, Style::default().fg(Color::LightYellow)),
        ]),

        Spans::from(vec![
            Span::styled("📡 Verification progress: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.4}%", blockchain_info.verificationprogress * 100.0),
                Style::default().fg(Color::Yellow),
            ),
        ]),

        Spans::from(vec![
            Span::styled("💾 Size on Disk: ", Style::default().fg(Color::Gray)),
            Span::styled(formatted_size_on_disk, Style::default().fg(Color::Gray)),
        ]),

        Spans::from(vec![
            Span::styled("   Median Time: ", Style::default().fg(Color::Gray)),
            Span::styled(mediantime, Style::default().fg(Color::Gray)),
        ]),

        Spans::from(vec![
            Span::styled("⏰ Block Time : ", Style::default().fg(Color::Gray)),
            Span::styled(time, Style::default().fg(Color::Gray)),
        ]),
    ];

    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section (only title)
                Constraint::Min(7),     // Content section
            ]
            .as_ref(),
        )
        .split(area); // Use the passed area

    // Header (use blockchain_info_text here after it's been defined).
    let header = if !blockchain_info_text.is_empty() {
        // Show the header with borders, but without displaying content.
        Block::default()
            .borders(Borders::NONE) // Add borders to header.
            .style(Style::default().fg(Color::Cyan))
    } else {
        // Render an empty block with no borders if no content exists.
        Block::default()
            .borders(Borders::NONE)
    };

    frame.render_widget(header, chunks[0]); // Render the header in the first chunk.

     // Render the blockchain info content in the third chunk.
    let blockchain_info_paragraph = Paragraph::new(blockchain_info_text)
       .block(Block::default().borders(Borders::NONE));
    frame.render_widget(blockchain_info_paragraph, chunks[1]);

    Ok(())
}


pub fn render_hashrate_distribution_chart<B: Backend>(
    distribution: &[(&str, u64)], 
    frame: &mut Frame<B>,
    area: Rect,
) -> Result<(), MyError> {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section (only title)
                Constraint::Min(7),     // Content section
            ]
            .as_ref(),
        )
        .split(area); // Use the passed area

    // Sort the distribution:
    // 1. Primary sort: Descending order by hashrate (second element of the tuple)
    // 2. Secondary sort: Ascending order by miner name (first element of the tuple)
    let mut sorted_distribution = distribution.to_vec();
    sorted_distribution.sort_by(|a, b| {
        let hashrate_cmp = b.1.cmp(&a.1); // Sort by hashrate (descending)
        if hashrate_cmp == std::cmp::Ordering::Equal {
            a.0.cmp(&b.0) // If hashrate is equal, sort by miner name (ascending)
        } else {
            hashrate_cmp
        }
    });

    // Take only the top 8 miners
    let top_8_distribution: Vec<(&str, u64)> = sorted_distribution
        .into_iter()
        .take(8) // Limit to top 8
        .collect();

    let total_miners = distribution.len();
    let top8_dist = top_8_distribution.len();

    let barchart = BarChart::default()
        .block(Block::default().title(format!("Hash Rate Distribution Top {} of {} 🌐 (24 hrs)", top8_dist, total_miners))
        .borders(Borders::ALL))
        .data(&top_8_distribution) // Use the sorted and filtered data
        .bar_width(7)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::DarkGray))
        .value_style(Style::default().fg(Color::White));
    
    frame.render_widget(barchart, chunks[1]);
    
    Ok(())
}