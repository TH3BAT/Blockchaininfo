
// display/display_network_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph, Sparkline},
    Frame,
};
use crate::models::{errors::MyError, network_info::NetworkInfo, network_totals::NetTotals};
use crate::utils::format_size;
use std::collections::VecDeque;

// Displays the network information in a `tui` terminal.
pub fn display_network_info<B: Backend>(
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    frame: &mut Frame<B>,
    version_counts: &[(String, usize)],
    avg_block_propagate_time: &i64,
    propagation_times: &VecDeque<i64>,
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

    // Define layout with space for Node Version Distribution bar chart and sparkline.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section.
                Constraint::Length(6), // Network info section.
                Constraint::Min(8),    // Bar chart and sparkline section.
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
            Span::styled("🔌 Connections in: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network_info.connections_in.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled("🔗 Connections out: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network_info.connections_out.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Spans::from(vec![
            Span::styled("⬇️ Total Bytes Received: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format_size(net_totals.totalbytesrecv).to_string(),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("⬆️ Total Bytes Sent: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format_size(net_totals.totalbytessent).to_string(),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("⏱️ Average Block Propagation Time: ", Style::default().fg(Color::Gray)),
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

    // Define sub-chunks for bar chart and sparkline.
    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(chunks[2]);

    // Node Version Distribution Bar Chart.
    if !version_counts.is_empty() {
        // Take only the top 5 versions.
        let limited_version_counts = version_counts.iter().take(5);

        // Convert limited version counts into BarChart format.
        let data: Vec<(&str, u64)> = limited_version_counts
            .map(|(version, count)| (version.as_str(), *count as u64))
            .collect();

        // Get the total number of versions for the title.
        let total_versions = version_counts.len();

        // BarChart for node version distribution.
        let barchart = BarChart::default()
            .block(
                Block::default()
                    .title(format!("Version Distribution (Top 5 of {})", total_versions)) // Dynamic title.
                    .borders(Borders::ALL),
            )
            .data(&data)
            .bar_width(8)
            .bar_gap(1)
            .bar_style(Style::default().fg(Color::DarkGray))
            .value_style(Style::default().fg(Color::White));

        // Render the BarChart in the left sub-chunk.
        frame.render_widget(barchart, sub_chunks[0]);
    }

    // Sparkline for block propagation times.
    if !propagation_times.is_empty() {
        // Bind the temporary vector to a variable for longer lifetime.
        let propagation_data: Vec<u64> = propagation_times.iter().map(|&t| t.unsigned_abs()).collect();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title("Propagation Times")
                    .borders(Borders::ALL),
            )
            .data(&propagation_data) // Pass the reference to the bound variable.
            .style(Style::default().fg(Color::DarkGray));

        // Render the Sparkline in the right sub-chunk.
        frame.render_widget(sparkline, sub_chunks[1]);
    } else {
        // println!("Propagation times are empty. Sparkline won't render.");
    }

    Ok(())
}
