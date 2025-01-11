
// display/display_mempool_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use num_format::{Locale, ToFormattedString};
use crate::utils::format_size;
use crate::models::mempool_info::MempoolInfo;
use crate::models::errors::MyError;

// Displays the mempool information in a `tui` terminal.
pub fn display_mempool_info<B: Backend>(
    frame: &mut Frame<B>,
    mempool_info: &MempoolInfo,
    area: Rect, // Added 'area' parameter
) -> Result<(), MyError> {
    // Calculate formatted and colored memory usage.
    let mempool_size_in_memory = format_size(mempool_info.usage);
    let max_mempool_size_in_memory = format_size(mempool_info.maxmempool);

    let mempool_size_in_memory_color = if mempool_info.usage < mempool_info.maxmempool / 3 {
        Style::default().fg(Color::Gray)
    } else if mempool_info.usage < 2 * mempool_info.maxmempool / 3 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Red)
    };

    let min_relay_fee_vsats = mempool_info.min_relay_tx_fee_vsats();

    // Create the layout for this specific chunk (using passed 'area').
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section (only title).
                Constraint::Min(5),     // Content section.
            ]
            .as_ref(),
        )
        .split(area);

    // Render header
    let header = Block::default()
        .borders(Borders::NONE) // Show borders for the header.
        .style(Style::default().fg(Color::Cyan)); // Style for borders (Cyan color).
    frame.render_widget(header, chunks[0]);


    // Mempool information content (without repeating title).
    let mempool_content = vec![
        Spans::from(vec![
            Span::styled("Transactions: ", Style::default().fg(Color::Gray)),
            Span::styled(
                mempool_info.size.to_formatted_string(&Locale::en),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled("Memory: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} ", mempool_size_in_memory),
                mempool_size_in_memory_color,
            ),
            Span::raw(format!("/ {}", max_mempool_size_in_memory)),
        ]),
        Spans::from(vec![
            Span::styled("Total Fees: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{:.8}", mempool_info.total_fee)),
        ]),
        Spans::from(vec![
            // The label "Min Transaction Fee: " in gray.
            Span::styled("Min Transaction Fee: ", Style::default().fg(Color::Gray)),
            
            // The value in yellow.
            Span::styled(
                format!("{}", min_relay_fee_vsats.to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Yellow),
            ),
            
            // The "vSats/vByte" text in gray.
            Span::styled(" vSats/vByte", Style::default().fg(Color::Gray)),
        ]),        
    ];

    // No borders for empty sections.
    let mempool_paragraph = Paragraph::new(mempool_content)
        .block(Block::default().borders(Borders::NONE)); // No border.
    
    frame.render_widget(mempool_paragraph, chunks[1]);

    Ok(())
}
