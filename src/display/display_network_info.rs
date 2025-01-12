
// display/display_network_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::models::network_info::NetworkInfo;
use crate::models::network_totals::NetTotals;
use crate::models::errors::MyError;

// Displays the network information in a `tui` terminal.
pub fn display_network_info<B: Backend>(
    frame: &mut Frame<B>,
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    version_counts: &Vec<(String, usize)>, 
    avg_block_propagate_time: u64,
    area: Rect,
) -> Result<(), MyError> {
    
    let color = if avg_block_propagate_time < 2 {
        Color::Green // Propagation time < 2 minute = Green (ideal).
    } else if avg_block_propagate_time < 5 {
        Color::Yellow // Propagation time between 2-5 minutes = Yellow (caution).
    } else {
        Color::Red // Propagation time > 5 minutes = Red (critical).
    };

    // Define layout for the network info, using the passed area.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section (only title).
                Constraint::Min(6),     // Content section.
            ]
            .as_ref(),
        )
        .split(area);

    // Header
    let header = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Network information content
    let mut network_content = vec![
        Spans::from(vec![
            Span::styled("Connections in: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network_info.connections_in.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled("Connections out: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network_info.connections_out.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Spans::from(vec![
            Span::styled("Total Bytes Received: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.2} MB", net_totals.totalbytesrecv as f64 / 1_048_576.0),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("Total Bytes Sent: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.2} MB", net_totals.totalbytessent as f64 / 1_048_576.0),
                Style::default().fg(Color::Gray),
            ),
        ]),
        // Determine the color based on the thresholds
        Spans::from(vec![
            Span::styled("Average Block Propagation Time: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0} minutes", avg_block_propagate_time),
                Style::default().fg(color),
            ),
        ]),
        // Spans::from(vec![]), // Blank line for separation.
    ];

    // Node Version Distribution.
    if !version_counts.is_empty() {
        network_content.push(
            Spans::from(vec![
            Span::styled("Node Version Distribution (Top 9):", Style::default().fg(Color::Gray)),
        ]));

        for (version, count) in version_counts {
            network_content.push(Spans::from(vec![
                Span::styled(format!("  - {}: ", version), Style::default().fg(Color::Gray)),
                Span::styled(format!("{} peers", count), Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    // Render the content
    let network_paragraph = Paragraph::new(network_content)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(network_paragraph, chunks[1]);

    Ok(())
}