// display/display_network_info.rs
//
// Network dashboard renderer.
//
// This module draws the Network section of the BlockchainInfo TUI.
// It includes:
//   - Incoming/outgoing connection counts (with flashing IN counter)
//   - Total bytes received/sent (formatted human-readable)
//   - Average block propagation time (color-coded severity)
//   - Toggle-view section: Version Distribution (BarChart) OR Client Distribution (ASCII)
//   - Sparkline showing recent block propagation times
//
// Like all display modules, it is pure rendering logic,
// receiving preprocessed data from `models` and plotting it visually.

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph, Sparkline},
    Frame,
};
use crate::models::{errors::MyError, network_info::NetworkInfo, network_totals::NetTotals};
use crate::utils::{format_size, normalize_percentages, BAR_ACTIVE};
use std::collections::VecDeque;
use crate::models::flashing_text::CONNECTIONS_IN_TEXT;

/// Renders the Network Information section of the dashboard.
///
/// This function displays:
///   - Incoming/outgoing peer counts
///   - Total bytes received/sent over the network
///   - Average block propagation time (with dynamic color coding)
///   - Either: version distribution (BarChart) OR client distribution (ASCII)
///   - A sparkline of recent propagation times
///
/// The caller controls whether to show client distribution via `show_client_distribution`.
pub fn display_network_info<B: Backend>(
    network_info: &NetworkInfo,
    net_totals: &NetTotals,
    frame: &mut Frame<B>,
    version_counts: &[(String, usize)],
    client_counts: &[(String, usize)],
    avg_block_propagate_time: &i64,
    propagation_times: &VecDeque<i64>,
    show_client_distribution: bool,
    show_propagation_avg: bool,
    area: Rect,
) -> Result<(), MyError> {
    
    // -----------------------------------------------------------------------
    // 1. BLOCK PROPAGATION TIME COLORING
    // -----------------------------------------------------------------------
    // Color thresholds:
    //   < 3 seconds      → Ideal (Green)
    //   < 60 seconds     → Caution (Yellow)
    //   >= 60 seconds    → Critical (Red)
    let color = if avg_block_propagate_time.abs() < 3 {
        Color::Green
    } else if avg_block_propagate_time.abs() < 60 {
        Color::Yellow
    } else {
        Color::Red
    };
    let abpt_text = "seconds";

    // -----------------------------------------------------------------------
    // 2. FLASHING CONNECTION-IN COUNTER
    // -----------------------------------------------------------------------
    // Each render, update the FlashingText handler so incoming connections
    // animate visually when the number changes.
    CONNECTIONS_IN_TEXT
        .lock()
        .unwrap()
        .update(network_info.connections_in as u64);

    let connections_in_style = CONNECTIONS_IN_TEXT.lock().unwrap().style();

    let connections_in_spans = Spans::from(vec![
        Span::styled("🔌 Connections in: ", Style::default().fg(Color::Gray)),
        Span::styled(network_info.connections_in.to_string(), connections_in_style),
    ]);

    // -----------------------------------------------------------------------
    // 3. TOP-LEVEL NETWORK LAYOUT
    // -----------------------------------------------------------------------
    // Layout for:
    //   chunks[0] → header (visual spacing)
    //   chunks[1] → network core stats
    //   chunks[2] → version/client distribution + sparkline
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header line.
                Constraint::Length(6),  // Network stats block.
                Constraint::Min(8),     // Distribution + Sparkline.
            ]
            .as_ref(),
        )
        .split(area);

    // Header placeholder (keeps consistency across display modules).
    let header = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // -----------------------------------------------------------------------
    // 4. CORE NETWORK STATS
    // -----------------------------------------------------------------------
    // These are presented as vertically stacked Span rows.
    let network_content = vec![
        connections_in_spans,

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
            Span::styled(
                "⏱️ Average Block Propagation Time: ",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                format!("{:.0} {}", avg_block_propagate_time, abpt_text),
                Style::default().fg(color),
            ),
        ]),
    ];

    // Render the network stats paragraph.
    let network_paragraph = Paragraph::new(network_content)
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(network_paragraph, chunks[1]);

    // -----------------------------------------------------------------------
    // 5. DISTRIBUTION + SPARKLINE LAYOUT
    // -----------------------------------------------------------------------
    // Left  68% → Version Distribution BarChart OR ASCII Client Distribution
    // Right 32% → Sparkline of propagation times
    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(chunks[2]);

    // -----------------------------------------------------------------------
    // 6. LEFT SIDE: CLIENT OR VERSION DISTRIBUTION
    // -----------------------------------------------------------------------
    if show_client_distribution {
        // ASCII client distribution (new feature)
        draw_client_distribution(frame, sub_chunks[0], client_counts);

    } else {
        // Traditional Version Distribution BarChart (Top 5 entries)
        if !version_counts.is_empty() {
            let limited_version_counts = version_counts.iter().take(5);

            // Convert input tuple format → BarChart data array
            let data: Vec<(&str, u64)> = limited_version_counts
                .map(|(version, count)| (version.as_str(), *count as u64))
                .collect();

            let total_versions = version_counts.len();

            let barchart = BarChart::default()
                .block(
                    Block::default()
                        .title(format!(
                            "Version Distribution (Top 5 of {})",
                            total_versions
                        ))
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

    // -----------------------------------------------------------------------
    // 7. RIGHT SIDE: AVERAGE OR SPARKLINE OF BLOCK PROPAGATION TIMES
    // -----------------------------------------------------------------------
    if show_propagation_avg {
        let total_len = propagation_times.len();

        let overall_avg = if total_len > 0 {
            propagation_times.iter().sum::<i64>() / total_len as i64
        } else {
            0
        };

        // Oldest 5 (only after 10+)
        let oldest_5_avg = if total_len >= 10 {
            Some(
                propagation_times.iter().take(5).sum::<i64>() / 5
            )
        } else {
            None
        };

        // Newest 5 (only when buffer is full)
        let newest_5_avg = if total_len == 20 {
        let sum: i64 = propagation_times
            .iter()
            .skip(15)
            .take(5)
            .sum();

        Some(sum / 5)

        } else {
            None
        };

        draw_propagation_avg(
            frame,
            sub_chunks[1],
            overall_avg,
            total_len as i64,
            oldest_5_avg,
            newest_5_avg,
        );

    } 
    else if !propagation_times.is_empty() {
        // Convert from VecDeque<i64> → Vec<u64> (unsigned)
        let propagation_data: Vec<u64> = propagation_times
            .iter()
            .map(|&t| t.unsigned_abs())
            .collect();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title("Propagation Times")
                    .borders(Borders::ALL),
            )
            .data(&propagation_data)
            .style(Style::default().fg(Color::DarkGray));

        frame.render_widget(sparkline, sub_chunks[1]);
    }

    Ok(())
}

/// Draws the ASCII Client Distribution panel.
///
/// This is used when `[Network] (c→Client)` toggle is active.
/// Displays up to 6 client names, with count, percent, and ASCII progress bar.
///
/// Example row:
///   BitcoinKnots     134  -  18% [====      ]
fn draw_client_distribution<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    client_counts: &[(String, usize)],
) {
    if client_counts.is_empty() {
        return;
    }

    // -----------------------------------------------------------------------
    // 1. Compute raw counts + normalized percentages
    // -----------------------------------------------------------------------
    let raw_counts: Vec<u64> = client_counts.iter().map(|(_, c)| *c as u64).collect();

    let pcts: Vec<u64> = normalize_percentages(&raw_counts);

    let mut lines: Vec<Spans> = Vec::new();

    // -----------------------------------------------------------------------
    // 2. Build up to 6 ASCII rows
    // -----------------------------------------------------------------------
    for ((name, count), pct) in client_counts.iter().zip(pcts.iter()).take(6) {
        // Fixed width bar = 10 chars
        let bar_width = 10;
        let filled = (*pct as usize * bar_width) / 100;
        let empty = bar_width - filled;

        let bar = format!("[{}{}]", "=".repeat(filled), " ".repeat(empty));

        let count_span = Span::styled(format!("{:>5} ", count), Style::default().fg(Color::Gray));

        let dash_span = Span::styled("- ", Style::default().fg(Color::DarkGray));

        let pct_span =
            Span::styled(format!("{:>3}% ", pct), Style::default().fg(Color::Gray));

        // Construct final row
        lines.push(Spans::from(vec![
            Span::styled(format!("{:<10}", name), Style::default().fg(Color::Cyan)),
            count_span,
            dash_span,
            pct_span,
            Span::styled(bar, Style::default().fg(BAR_ACTIVE)),
        ]));
    }

    // Place a blank spacer row at top for visual centering.
    lines.insert(0, Spans::from(" "));

    // Build the containing block + paragraph
    let block = Block::default()
        .title("Client Distribution")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(lines).block(block);

    frame.render_widget(paragraph, area);
}

/// Draws the average block propagation time panel.
///
/// Displays the signed average propagation delay (in seconds) computed
/// over the last 20 blocks. This view provides a quick, numerical anchor
/// for network synchronization health, complementing the sparkline view
/// which emphasizes variance and shape rather than direction.
///
/// The value is intentionally rendered as whole seconds to keep the signal
/// calm, readable, and free of visual noise.
pub fn draw_propagation_avg<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    overall_avg: i64,
    propagation_len: i64,
    oldest_5_avg: Option<i64>,
    newest_5_avg: Option<i64>,
) {

    let mut lines = Vec::new();

    lines.push(format!(
        "Avg ({} blks): {}s",
        propagation_len,
        overall_avg
    ));

    if let Some(avg) = oldest_5_avg {
        lines.push(format!(
            "Oldest 5: {}s",
            avg
        ));
    }

    if let Some(avg) = newest_5_avg {
        lines.push(format!(
            "Latest 5: {}s",
            avg
        ));
    }

    let content = format!("\n{}\n", lines.join("\n"));


    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Propagation Avg")
                .borders(Borders::ALL),
        );

    frame.render_widget(paragraph, area);

}
