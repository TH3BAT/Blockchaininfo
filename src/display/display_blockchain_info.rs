
// display/display_blockchain_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use num_format::{Locale, ToFormattedString};
use crate::{models::{block_info::BlockInfo, blockchain_info::BlockchainInfo}, 
    utils::{estimate_difficulty_change, estimate_24h_difficulty_change, format_size}};
use crate::models::errors::MyError;  
use crate::models::flashing_text::BEST_BLOCK_TEXT;

// Render the blockchain info into a `tui` terminal UI.
pub fn display_blockchain_info<B: Backend>(
    blockchain_info: &BlockchainInfo,
    block_info: &BlockInfo,
    block24_info: &BlockInfo,
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

    // Get the style for the FlashingText
    let best_block_style = BEST_BLOCK_TEXT.lock().unwrap().style();

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
    ]);

    // Build the blockchain info text before using it.
    let blockchain_info_text = vec![
        Spans::from(vec![
            Span::styled("🔗 Chain: ", Style::default().fg(Color::Gray)),
            Span::styled(blockchain_info.chain.clone(), Style::default().fg(Color::Yellow)),
        ]),

        best_block_spans, // Flash yellow when best block changes.
        
        /* Keeping previous TUI code   
        Spans::from(vec![
            Span::styled("🏆 Best Block: ", Style::default().fg(Color::Gray)),
            Span::styled(
                blockchain_info.blocks.to_formatted_string(&Locale::en),
                Style::default().fg(Color::Green),
            ),
        ]),
        */

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
