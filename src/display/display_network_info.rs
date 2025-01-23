
// display/display_network_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph},
    Frame,
};
use crate::models::network_info::NetworkInfo;
use crate::models::network_totals::NetTotals;
use crate::models::errors::MyError;
use crate::utils::format_size;

// Displays the network information in a `tui` terminal.
pub fn display_network_info<B: Backend>(
    frame: &mut Frame<B>,
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    version_counts: &Vec<(String, usize)>,
    avg_block_propagate_time: i64,
    area: Rect,
) -> Result<(), MyError> {
    // Determine color based on average block propagation time.
    let color = if avg_block_propagate_time.abs() < 3 {
        Color::Green // Ideal.
    } else if avg_block_propagate_time.abs() < 60 {
        Color::Yellow // Caution.
    } else {
        Color::Red // Critical.
    };
    let abpt_text = "seconds";

    // Define layout with space for Node Version Distribution bar chart.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section.
                Constraint::Length(6), // Network info section.
                Constraint::Min(8),    // Bar chart section.
            ]
            .as_ref(),
        )
        .split(area);

    // Header block.
    let header = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // Network information content.
    let network_content = vec![
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
                format!("{}", format_size(net_totals.totalbytesrecv)),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("Total Bytes Sent: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", format_size(net_totals.totalbytessent)),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("Average Block Propagation Time: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.0} {}", avg_block_propagate_time, abpt_text),
                Style::default().fg(color),
            ),
        ]),
    ];

    // Render network info paragraph.
    let network_paragraph = Paragraph::new(network_content)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(network_paragraph, chunks[1]);

    // Node Version Distribution Bar Chart.
    if !version_counts.is_empty() {
        // Take only the top 7 versions.
        let limited_version_counts = version_counts.iter().take(7);
    
        // Convert limited version counts into BarChart format.
        let data: Vec<(&str, u64)> = limited_version_counts
            .map(|(version, count)| (version.as_str(), *count as u64))
            .collect();
    
        // BarChart for node version distribution.
        let barchart = BarChart::default()
            .block(
                Block::default()
                    .title("Node Version Distribution (Top 7)")
                    .borders(Borders::ALL),
            )
            .data(&data)
            .bar_width(8)
            .bar_gap(1)
            .bar_style(Style::default().fg(Color::DarkGray))
            .value_style(Style::default().fg(Color::White));
    
        // Render the BarChart in the appropriate chunk.
        frame.render_widget(barchart, chunks[2]);
    }

    Ok(())
}