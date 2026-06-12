//! display/display_network_info.rs
//!
//! Network dashboard renderer.
//!
//! This module draws the Network section of the BlockchainInfo TUI.
//! It includes:
//!   - Incoming/outgoing connection counts (with flashing IN counter)
//!   - Total bytes received/sent (formatted human-readable)
//!   - Average block propagation time (color-coded severity)
//!   - Toggle-view section: Version Distribution (BarChart) OR Client Distribution (ASCII)
//!   - Sparkline showing recent block propagation times
//!
//! Like all display modules, it is pure rendering logic,
//! receiving preprocessed data from `models` and plotting it visually.

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph, Sparkline},
    Frame,
};
use crate::models::{errors::MyError, network_info::NetworkInfo, network_totals::NetTotals};
use crate::utils::{format_size, normalize_percentages, create_progress_bar};
use crate::ui::colors::*;
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
    uasf_counts: &[((String, String), usize)],
    propagation_times: &VecDeque<i64>,
    show_client_distribution: bool,
    show_propagation_avg: bool,
    show_uasf_distribution: bool,
    area: Rect,
) -> Result<(), MyError> {
    
    // Extract current average propagation time.
    let latest_propagation_time = propagation_times
        .back()
        .copied()
        .unwrap_or(0);

    // Passed to draw_uasf_distribution() to calculate pct of total peers.    
    let total_version_peers: usize =
        version_counts.iter().map(|(_, count)| *count).sum();

    // -----------------------------------------------------------------------
    // 1. BLOCK PROPAGATION TIME COLORING
    // -----------------------------------------------------------------------
    // Color thresholds:
    //   < 3 seconds      → Ideal (Green)
    //   < 60 seconds     → Caution (Yellow)
    //   >= 60 seconds    → Critical (Red)
    let color = if latest_propagation_time.abs() < 3 {
        C_STATUS_LOW
    } else if latest_propagation_time.abs() < 60 {
        C_STATUS_MED
    } else {
        C_STATUS_HIGH
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
        Span::styled("🔌 In: ", Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::DIM)),
        Span::styled(network_info.connections_in.to_string(), connections_in_style),
        Span::raw("   "),
        Span::styled("Out: ", Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::DIM)),
        Span::styled(
            network_info.connections_out.to_string(),
            Style::default().fg(C_CONNECTIONS_OUT),
        ),
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
                Constraint::Length(4),  // Network stats block.
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
    // -----------------------------------------------------------------------
    let network_content = vec![
        connections_in_spans,

        Spans::from(vec![
            Span::styled("⬇️ Recv: ", Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::DIM)),
            Span::styled(
                format_size(net_totals.totalbytesrecv),
                Style::default().fg(C_MAIN_LABELS),
            ),
            Span::raw("   "),
            Span::styled("⬆️ Sent: ", Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::DIM)),
            Span::styled(
                format_size(net_totals.totalbytessent),
                Style::default().fg(C_MAIN_LABELS),
            ),
        ]),

        Spans::from(vec![
            Span::styled(
                "⏱️ Average Block Propagation Time: ",
                Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::DIM),
            ),
            Span::styled( 
                format!("{:.0} {}", latest_propagation_time, abpt_text),
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
    // 6. LEFT SIDE: CLIENT / VERSION / UASF DISTRIBUTION
    // -----------------------------------------------------------------------
    if show_uasf_distribution {
        draw_uasf_distribution(frame, sub_chunks[0], uasf_counts, total_version_peers as usize);

    } else if show_client_distribution {
        draw_client_distribution(frame, sub_chunks[0], client_counts);

    } else {
        // Traditional Version Distribution BarChart (Top 5 entries)
        if !version_counts.is_empty() {
            let limited_version_counts = version_counts.iter().take(5);

            let data: Vec<(&str, u64)> = limited_version_counts
                .map(|(version, count)| (version.as_str(), *count as u64))
                .collect();

            let total_versions = version_counts.len();
            let top5orless = total_versions.min(5);

            let barchart = BarChart::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(
                            Style::default()
                                .fg(Color::DarkGray)
                                .add_modifier(Modifier::DIM),
                        )
                        .title(Span::styled(
                            format!("Version Distribution (Top {} of {})", top5orless, total_versions),
                            Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::BOLD),
                        )),
                )
                .data(&data)
                .bar_width(7)
                .bar_gap(1)
                .bar_style(Style::default().fg(C_VERSION_CHART_BARS))
                .value_style(Style::default().fg(C_VERSION_CHART_VALUES));

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
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    )
                    .title(Span::styled(
                        format!("Propagation Times"),
                        Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::BOLD),
                        )),
            )
            .data(&propagation_data)
            .style(Style::default().fg(C_SPARKLINE));

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
    // 0. Build a display list: top 5 + "Other" (rest)
    // -----------------------------------------------------------------------
    let mut rows: Vec<(String, usize)> = client_counts.to_vec();

    rows.sort_by(|a, b| b.1.cmp(&a.1));

    let rows: Vec<(String, usize)> = if rows.len() <= 6 {
        rows
    } else {
        let mut top = rows.into_iter().take(5).collect::<Vec<_>>();
        let other_sum: usize = client_counts.iter().skip(5).map(|(_, c)| *c).sum();
        top.push(("Other".to_string(), other_sum));
        top
    };

    // -----------------------------------------------------------------------
    // 1. Compute raw counts + normalized percentages
    // -----------------------------------------------------------------------
    let raw_counts: Vec<u64> = rows.iter().map(|(_, c)| *c as u64).collect();

    let pcts: Vec<u64> = normalize_percentages(&raw_counts);

    let mut lines: Vec<Spans> = Vec::new();

    // -----------------------------------------------------------------------
    // 2. Build up to 6 ASCII rows
    // -----------------------------------------------------------------------
    for ((name, count), pct) in rows.iter().zip(pcts.iter()) {
        // Fixed width bar = 10 chars
        //let bar_width = 10;
        //let filled = (*pct as usize * bar_width) / 100;
        //let empty = bar_width - filled;

        let bar = create_progress_bar(*pct, 10);

        let count_span = Span::styled(format!("{:>5} ", count), Style::default().fg(C_CLIENT_DIST_MINER_COUNT));

        let dash_span = Span::styled("- ", Style::default().fg(C_SEPARATORS));

        let pct_span =
            Span::styled(format!("{:>3}% ", pct), Style::default().fg(C_CLIENT_DIST_MINER_PCT));

        // Construct final row
        lines.push(Spans::from(vec![
            Span::styled(format!("{:<10}", name), Style::default().fg(C_CLIENT_DIST_MINER_LABEL).add_modifier(Modifier::DIM)),
            // .add_modifier(Modifier::BOLD)),
            count_span,
            dash_span,
            pct_span,
            Span::styled(bar, Style::default().fg(C_HORIZONTAL_ASCII_BAR)
            .add_modifier(Modifier::DIM)),
        ]));
    }

    // Place a blank spacer row at top for visual centering.
    lines.insert(0, Spans::from(" "));

    // Build the containing block + paragraph
    let block = Block::default()
    .title(Span::styled(
        "Client Distribution",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    ))
    .borders(Borders::ALL)
    .border_style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM),
    );

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

   let mut lines: Vec<Spans> = Vec::new();

   lines.push(Spans::from(Span::raw("")));

    lines.push(Spans::from(vec![
        Span::styled(
            format!("Avg ({} blks): ", propagation_len),
            Style::default()
                .fg(C_MAIN_LABELS)
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(
            format!("{}s", overall_avg),
            Style::default().fg(C_MAIN_LABELS),
        ),
    ]));

    if let Some(avg) = oldest_5_avg {
        lines.push(Spans::from(vec![
            Span::styled(
                "Oldest 5: ",
                Style::default()
                    .fg(C_MAIN_LABELS)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                format!("{}s", avg),
                Style::default().fg(C_MAIN_LABELS),
            ),
        ]));
    }

    if let Some(avg) = newest_5_avg {
        lines.push(Spans::from(vec![
            Span::styled(
                "Latest 5: ",
                Style::default()
                    .fg(C_MAIN_LABELS)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                format!("{}s", avg),
                Style::default().fg(C_MAIN_LABELS),
            ),
        ]));
    }

    lines.push(Spans::from(Span::raw("")));

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(
                    "Propagation Avg",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
        );

    frame.render_widget(paragraph, area);

}

/// Draws the UASF signal distribution panel in the Network section.
///
/// Displays the top observed UASF signals parsed from peer subversion strings,
/// grouped by BIP identifier and signal version.
///
/// Example parsed signal:
/// `/Satoshi:29.2.0/Knots:20251110+bip110-v0.1/UASF-BIP110:0.1/`
///
/// renders as:
/// `BIP110  v0.1`
///
/// Percentages are calculated against the same filtered peer universe used by
/// Version Distribution, keeping UASF signal share aligned with observed
/// Bitcoin/Satoshi peers rather than raw connection count.
///
/// This panel is observational only:
/// it reports visible peer signals and does not infer activation, support,
/// enforcement, or network consensus.
fn draw_uasf_distribution<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    uasf_counts: &[((String, String), usize)],
    total_peers: usize,
) {
    let mut lines: Vec<Spans> = Vec::new();

    // Add spacing below border title
    lines.push(Spans::from(""));

    if uasf_counts.is_empty() || total_peers == 0 {
        lines.push(Spans::from(Span::styled(
            "No UASF signals observed",
            Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD),
        )));
    } else {
        for ((bip, version), count) in uasf_counts.iter().take(5) {
            let pct = (*count as f64 / total_peers as f64) * 100.0;

            lines.push(Spans::from(vec![
                Span::styled(
                    format!("{:<8}", bip),
                    Style::default().fg(C_UASF_SIGNAL_LABEL).add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    format!(" v{:<5}", version),
                    Style::default().fg(C_UASF_SIGNAL_VERSION),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:>3}", count),
                    Style::default().fg(C_UASF_SIGNAL_COUNT),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:>5.1}%", pct),
                    Style::default().fg(C_UASF_SIGNAL_PCT),
                ),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        )
        .title(Span::styled(
            format!("UASF Signals (Top {} of {})", uasf_counts.len().min(4), uasf_counts.len()),
            Style::default().fg(C_MAIN_LABELS).add_modifier(Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}