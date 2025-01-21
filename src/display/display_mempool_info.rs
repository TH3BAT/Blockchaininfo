
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
use crate::models::mempool_info::{MempoolInfo, MempoolDistribution};
use crate::models::errors::MyError;

// Displays the mempool information in a `tui` terminal.
pub fn display_mempool_info<B: Backend>(
    frame: &mut Frame<B>,
    mempool_info: &MempoolInfo,
    distribution: &MempoolDistribution,
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
    let total_size = distribution.small + distribution.medium + distribution.large;
    let total_age = distribution.young + distribution.moderate + distribution.old;
    let total_rbf = distribution.rbf_count + distribution.non_rbf_count;

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
        // Spans::from(vec![]), // Blank line for separation.
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
         // Spans::from(vec![]), // Blank line for separation.
         // Size Distribution
        Spans::from(vec![Span::styled("Size Distribution (5%, excluding dust transactions):", Style::default().fg(Color::Gray)),]),
        Spans::from(vec![
            Span::styled("  Small (< 250 vBytes)    : ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.small * 20).to_formatted_string(&Locale::en),
                    if total_size > 0 { distribution.small * 100 / total_size } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  Medium (250-1000 vBytes): ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.medium * 20).to_formatted_string(&Locale::en),
                    if total_size > 0 { distribution.medium * 100 / total_size } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  Large (> 1000 vBytes)   : ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.large * 20).to_formatted_string(&Locale::en),
                    if total_size > 0 { distribution.large * 100 / total_size } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),

        // Age Distribution
        Spans::from(vec![Span::styled("Age Distribution (5%, excluding dust transactions):", Style::default().fg(Color::Gray)),]),
        Spans::from(vec![
            Span::styled("  Young (< 5 min)         : ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.young * 20).to_formatted_string(&Locale::en),
                    if total_age > 0 { distribution.young * 100 / total_age } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  Moderate (5 min - 1 hr) : ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.moderate * 20).to_formatted_string(&Locale::en),
                    if total_age > 0 { distribution.moderate * 100 / total_age } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  Old (> 1 hr)            : ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.old * 20).to_formatted_string(&Locale::en),
                    if total_age > 0 { distribution.old * 100 / total_age } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),

        // RBF Distribution
        Spans::from(vec![Span::styled("RBF Distribution (5%, excluding dust transactions):", Style::default().fg(Color::Gray)),]),
        Spans::from(vec![
            Span::styled("  RBF Transactions    : ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.rbf_count * 20).to_formatted_string(&Locale::en),
                    if total_rbf > 0 { distribution.rbf_count * 100 / total_rbf } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  Non-RBF Transactions: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{} ({}%)",
                    (distribution.non_rbf_count * 20).to_formatted_string(&Locale::en),
                    if total_rbf > 0 { distribution.non_rbf_count * 100 / total_rbf } else { 0 }
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![Span::styled("Fee Metrics (5%, excluding dust transactions):", Style::default().fg(Color::Gray))]),
        Spans::from(vec![
            Span::styled("  Average Fee (BTC): ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.8}", distribution.average_fee),
                Style::default().fg(Color::Gray),
            ),
        ]),

        Spans::from(vec![
            Span::styled("  Median Fee (BTC): ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.8}", distribution.median_fee),
                Style::default().fg(Color::Gray),
            ),
        ]),

        Spans::from(vec![
            Span::styled("  Average Fee Rate (sats/vByte): ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.2}", distribution.average_fee_rate), // Show two decimal places for precision.
                Style::default().fg(Color::Gray),
            ),
        ]),
    ];

    // No borders for empty sections.
    let mempool_paragraph = Paragraph::new(mempool_content)
        .block(Block::default().borders(Borders::NONE)); // No border.
    
    frame.render_widget(mempool_paragraph, chunks[1]);

    Ok(())
}
