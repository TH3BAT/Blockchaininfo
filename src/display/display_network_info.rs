
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
use crate::utils::{format_size, normalize_percentages};
use std::collections::VecDeque;
use crate::models::flashing_text::CONNECTIONS_IN_TEXT;

// Displays the network information in a `tui` terminal.
pub fn display_network_info<B: Backend>(
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    frame: &mut Frame<B>,
    version_counts: &[(String, usize)],
    client_counts: &[(String, usize)],
    avg_block_propagate_time: &i64,
    propagation_times: &VecDeque<i64>,
    show_client_distribution: bool,
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

    // Update the FlashingText variable
    CONNECTIONS_IN_TEXT.lock().unwrap().update(network_info.connections_in as u64);

    // Get the style for the FlashingText
    let connections_in_style = CONNECTIONS_IN_TEXT.lock().unwrap().style();

    let connections_in_spans =Spans::from(vec![
        Span::styled("🔌 Connections in: ", Style::default().fg(Color::Gray)),
        Span::styled(
            network_info.connections_in.to_string(),
            connections_in_style,
        ),
    ]);

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
        connections_in_spans,
        /* Spans::from(vec![
            Span::styled("🔌 Connections in: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network_info.connections_in.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]), */
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

    if show_client_distribution {
        // 🔹 ASCII client distribution in sub_chunks[0]
        draw_client_distribution(frame, sub_chunks[0], client_counts);
    } else {
        // 🔹 Your existing Version Distribution BarChart
        if !version_counts.is_empty() {
            let limited_version_counts = version_counts.iter().take(5);

            let data: Vec<(&str, u64)> = limited_version_counts
                .map(|(version, count)| (version.as_str(), *count as u64))
                .collect();

            let total_versions = version_counts.len();

            let barchart = BarChart::default()
                .block(
                    Block::default()
                        .title(format!("Version Distribution (Top 5 of {})", total_versions))
                        .borders(Borders::ALL),
                )
                .data(&data)
                .bar_width(7)
                .bar_gap(1)
                .bar_style(Style::default().fg(Color::DarkGray))
                .value_style(Style::default().fg(Color::White));

            frame.render_widget(barchart, sub_chunks[0]);
        }
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

fn draw_client_distribution<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    client_counts: &[(String, usize)],
) {
    if client_counts.is_empty() {
        return;
    }

    // NEW: extract raw counts and compute normalized percentages
    let raw_counts: Vec<u64> = client_counts
        .iter()
        .map(|(_, c)| *c as u64)
        .collect();

    let pcts: Vec<u64> = normalize_percentages(&raw_counts);

    let mut lines: Vec<Spans> = Vec::new();

    // UPDATED: zip client rows with normalized percentages
    for ((name, count), pct) in client_counts.iter().zip(pcts.iter()).take(6) {

        // ASCII bar width
        let bar_width = 10;
        let filled = (*pct as usize * bar_width) / 100;
        let empty = bar_width - filled;

        let bar = format!(
            "[{}{}]",
            "=".repeat(filled),
            " ".repeat(empty)
        );

        lines.push(Spans::from(vec![
            Span::styled(
                format!("{:<10}", name),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(format!("{:>5} - {:>3}% ", count, pct)),   // <-- modernized
            Span::styled(bar, Style::default().fg(Color::DarkGray)),
        ]));
    }
    // Insert a blank row at the top for vertical centering
    lines.insert(0, Spans::from(" "));
    
    let block = Block::default()
        .title("Client Distribution")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(paragraph, area);
}
