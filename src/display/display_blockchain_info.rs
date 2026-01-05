
// This module renders all Blockchain-related metrics in the TUI.
// It draws Best Block, Miner, Difficulty, Time Since Block,
// difficulty projections, chainwork, verification progress,
// disk size, timestamps, and the Hash Rate Distribution chart.
//
// No RPC logic lives here — this is pure UI rendering.
//

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph, Wrap},
    Frame,
};
use num_format::{Locale, ToFormattedString};
use crate::{
    models::{block_info::BlockInfo, blockchain_info::BlockchainInfo},
    utils::{estimate_difficulty_change, estimate_24h_difficulty_change, format_size},
    ui::colors::*
};
use crate::models::errors::MyError;
use crate::models::flashing_text::{BEST_BLOCK_TEXT, MINER_TEXT};
use crate::consensus::satoshi_math::*;
use std::sync::Arc;


/// Renders the Blockchain section of the dashboard.
///
/// This includes:
/// • Chain name  
/// • Best block height (flashing)  
/// • Miner of best block (flashing)  
/// • Time since block  
/// • Difficulty + two projections (epoch + 24h)  
/// • Blocks until next retarget  
/// • Chainwork  
/// • Verification progress  
/// • On-disk size  
/// • Median and block timestamps  
///
/// All styling and layout is handled here.
pub fn display_blockchain_info<B: Backend>(
    blockchain_info: &BlockchainInfo,
    block_info: &BlockInfo,
    block24_info: &BlockInfo,
    last_miner: &Arc<str>,
    frame: &mut Frame<B>,
    area: Rect,
) -> Result<(), MyError> {
    
    // Convert blockchain timestamps + sizes into displayable formats.
    let mediantime = blockchain_info.parse_mediantime()?;
    let time = blockchain_info.parse_time()?;
    let formatted_size_on_disk = format_size(blockchain_info.size_on_disk);
    let time_since_block = blockchain_info.calculate_time_diff()?;
    let formatted_difficulty = blockchain_info.formatted_difficulty()?;
    let formatted_chainwork_bits = blockchain_info.formatted_chainwork_bits()?;

    // Epoch-based difficulty projection.
    // Uses timestamp of last block in epoch-start window.
    let estimate_difficulty_chng = estimate_difficulty_change(
        blockchain_info.blocks,
        blockchain_info.time,
        block_info.time,
    );

    // Determine how deep we are into the current difficulty epoch.
    // (epoch = 2016 blocks)
    let height = blockchain_info.blocks;
    let blocks_into_epoch = height % DIFFICULTY_ADJUSTMENT_INTERVAL;
    
    // Difficulty estimate shown only after block 5 of the epoch.
    let difficulty_change_display = if blocks_into_epoch < 5 {
        Span::styled(" N/A ", Style::default().fg(C_MAIN_LABELS))
    } else {
        Span::styled(
            format!(" {:.2}% ", estimate_difficulty_chng.abs()),
            Style::default().fg(C_MAIN_LABELS),
        )
    };

    // 24-hour difficulty projection uses timestamps of latest and 24h-ago block.
    let estimate_24h_difficulty_chng = estimate_24h_difficulty_change(
        blockchain_info.time,
        block24_info.time,
    );

    // Arrow for epoch diff projection.
    let difficulty_arrow = if block_info.confirmations < 6 {
        " ".to_string() // Hidden arrow during N/A period
    } else if estimate_difficulty_chng > 0.0 {
        "↑".to_string()
    } else {
        "↓".to_string()
    };

    // Arrow for 24-hour diff projection.
    let difficulty_arrow_24h = if estimate_24h_difficulty_chng > 0.0 {
        "↑".to_string()
    } else {
        "↓".to_string()
    };

    // FlashingText system: update Best Block & Miner flashing styles.
    BEST_BLOCK_TEXT.lock().unwrap().update(blockchain_info.blocks);
    MINER_TEXT.lock().unwrap().update(last_miner.to_string());

    let best_block_style = BEST_BLOCK_TEXT.lock().unwrap().style();
    let last_miner_style = MINER_TEXT.lock().unwrap().style();

    // Build the "Best Block | Miner" line with dynamic flashing styles.
    let best_block_spans = Spans::from(vec![
        Span::styled("🏆 Best Block: ", Style::default().fg(C_MAIN_LABELS)),
        Span::styled(
            blockchain_info.blocks.to_formatted_string(&Locale::en),
            best_block_style,
        ),
        Span::styled(" | ", Style::default().fg(C_SEPARATORS)),
        Span::styled("⛏️ Miner: ", Style::default().fg(C_MAIN_LABELS)),
        Span::styled(format!("{}", last_miner), last_miner_style),
    ]);

    // Build every display line in a Vec<Spans>.
    let blockchain_info_text = vec![
        Spans::from(vec![
            Span::styled("🔗 Chain: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(blockchain_info.chain.clone(), Style::default().fg(C_CHAIN)),
        ]),

        best_block_spans, // Flashing block + miner line

        Spans::from(vec![
            Span::styled("  ⏳ Time since block: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(time_since_block, Style::default().fg(C_TIME_SINCE_BLOCK)),
        ]),

        Spans::from(vec![
            Span::styled("🎯 Difficulty: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(formatted_difficulty, Style::default().fg(C_DIFFICULTY)),
        ]),

        // Remaining blocks in difficulty epoch.
        Spans::from(vec![
            Span::styled("     Blocks until adjustment: ", Style::default().fg(C_MAIN_LABELS)),
            match blockchain_info.display_blocks_until_difficulty_adjustment() {
                Ok((block_text, block_color)) =>
                    Span::styled(block_text, Style::default().fg(block_color)),
                Err(e) =>
                    Span::styled(format!("Error: {}", e), Style::default().fg(Color::Red)),
            },
        ]),

        // Difficulty projections block (epoch + 24hr).
        Spans::from(vec![
            Span::styled("  📉 Estimated change: ", Style::default().fg(C_MAIN_LABELS)),

            // Epoch arrow
            Span::styled(
                difficulty_arrow,
                Style::default().fg(if estimate_difficulty_chng > 0.0 { C_ESTIMATE_POS } else { C_ESTIMATE_NEG }),
            ),
            difficulty_change_display,

            Span::styled("(epoch)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            Span::styled(" | ", Style::default().fg(C_SEPARATORS)),

            // 24h arrow
            Span::styled(
                difficulty_arrow_24h,
                Style::default().fg(if estimate_24h_difficulty_chng > 0.0 { C_ESTIMATE_POS } else { C_ESTIMATE_NEG }),
            ),
            Span::styled(
                format!(" {:.2}% ", estimate_24h_difficulty_chng.abs()),
                Style::default().fg(C_MAIN_LABELS),
            ),
            Span::styled("(24hrs)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]),

        // Chainwork line
        Spans::from(vec![
            Span::styled("   Chainwork: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(formatted_chainwork_bits, Style::default().fg(C_CHAINWORK)),
        ]),

        // Verification progress
        Spans::from(vec![
            Span::styled("📡 Verification progress: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(
                format!("{:.4}%", blockchain_info.verificationprogress * 100.0),
                Style::default().fg(C_VERIFICATION),
            ),
        ]),

        // Disk size
        Spans::from(vec![
            Span::styled("💾 Size on Disk: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(formatted_size_on_disk, Style::default().fg(C_MAIN_LABELS)),
        ]),

        // Median time
        Spans::from(vec![
            Span::styled("   Median Time: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(mediantime, Style::default().fg(C_MAIN_LABELS)),
        ]),

        // Block time
        Spans::from(vec![
            Span::styled("⏰ Block Time : ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(time, Style::default().fg(C_MAIN_LABELS)),
        ]),
    ];

    // Layout:
    // [ Header (1 line) ]
    // [ Blockchain content ]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1), Constraint::Min(7)].as_ref())
        .split(area);

    // A decorative header block (currently empty of text).
    let header = if !blockchain_info_text.is_empty() {
        Block::default().borders(Borders::NONE).style(Style::default().fg(Color::Cyan))
    } else {
        Block::default().borders(Borders::NONE)
    };

    frame.render_widget(header, chunks[0]);

    // Main content paragraph.
    let blockchain_info_paragraph =
        Paragraph::new(blockchain_info_text).block(Block::default().borders(Borders::NONE));

    frame.render_widget(blockchain_info_paragraph, chunks[1]);

    Ok(())
}


/// Renders the Hash Rate Distribution chart (top 8 miners).
///
/// Sorting:
/// • Primary: descending by hashrate  
/// • Secondary: ascending by miner name  
///
/// Then converts Arc<str> → &str for the BarChart widget.
pub fn render_hashrate_distribution_chart<B: Backend>(
    distribution: &Vec<(Arc<str>, u64)>,
    frame: &mut Frame<B>,
    area: Rect,
) -> Result<(), MyError> {

    // Use to show block representation that replaces static '24 hrs' time.
    let window_blocks: u64 = distribution.iter().map(|entry| entry.1).sum();
    let window_display = if window_blocks < (BLOCKS_PER_HOUR * HOURS_PER_DAY) {
        format!("{}/{} blks", window_blocks, (BLOCKS_PER_HOUR * HOURS_PER_DAY))
    } else {
        format!("{} blks", (BLOCKS_PER_HOUR * HOURS_PER_DAY))
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1), Constraint::Min(7)].as_ref())
        .split(area);

    // Sort by hashrate descending, name ascending.
    let mut sorted_distribution = distribution.to_vec();
    sorted_distribution.sort_by(|a, b| {
        let hashrate_cmp = b.1.cmp(&a.1);
        if hashrate_cmp == std::cmp::Ordering::Equal {
            a.0.cmp(&b.0)
        } else {
            hashrate_cmp
        }
    });

    // Keep only top 8 miners.
    let top_8_distribution: Vec<(Arc<str>, u64)> = sorted_distribution.into_iter().take(8).collect();

    let total_miners = distribution.len();
    let top8_dist = top_8_distribution.len();

    // Convert for tui::widgets::BarChart.
    let top_8_distribution_ref: Vec<(&str, u64)> = top_8_distribution
        .iter()
        .map(|(miner, hashrate)| (miner.as_ref(), *hashrate))
        .collect::<Vec<_>>();

    let barchart = BarChart::default()
        .block(
            Block::default()
                .title(format!(
                    "Hash Rate Distribution Top {} of {} 🌐 ({})",
                    top8_dist, total_miners, window_display
                ))
                .borders(Borders::ALL),
        )
        .data(&top_8_distribution_ref)
        .bar_width(7)
        .bar_gap(1)
        .bar_style(Style::default().fg(C_HASHRATE_CHART_BARS))
        .value_style(Style::default().fg(C_HASHRATE_CHART_VALUES));

    frame.render_widget(barchart, chunks[1]);

    Ok(())
}

/// Renders the "Last 20 Blocks / Miner" panel.
///
/// Displays a rolling window of the most recent block heights
/// alongside their associated miners, using data derived from
/// `BLOCK_HISTORY`.
///
/// Layout:
/// • Two-column table (10 rows per column)
/// • Block height aligned right
/// • Miner label truncated to fit available width
///
/// Ordering:
/// • Oldest block at the top
/// • Newest block at the bottom
///
/// Behavior:
/// • If fewer than 20 blocks are available (startup / sync),
///   only the available rows are rendered.
/// • Unknown or missing miner labels are shown as "Unknown".
///
/// Notes:
/// • This function is render-only and performs no locking or async work.
/// • Data preparation (including height association) must be done
///   upstream in the async runtime before calling this function.
/// • Designed to pair with the Blockchain panel toggle `[L] 20`.
pub fn draw_last20_miners<B: Backend>(
    frame: &mut Frame<B>,
    area: Rect,
    rows: &[(u64, Option<Arc<str>>)],
) {
    // Inner area (match other panels: keep it simple)
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Split into header + body
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
    .split(inner);

    // Split body into 2 columns
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // Header line
    let header = Paragraph::new(Spans::from(vec![
        Span::styled("Last 20 Blocks / Miners", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::raw("(newest at top)"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE))
    .wrap(Wrap { trim: true });

    frame.render_widget(header, chunks[0]);

    // Helper: format a single row as "HEIGHT  MINER"
    fn fmt_line(
        width: u16,
        height: u64,
        miner: Option<&str>,
    ) -> Spans<'static> {
        let height_str = format!("{:>7} ", height);
        let spacer = "  ";
        let miner_str = miner.unwrap_or("Unknown");

        let available = width as usize;
        let mut miner_out = miner_str.to_string();

        let remaining = available.saturating_sub(height_str.len()).max(1);
        if miner_out.len() > remaining {
            if remaining >= 2 {
                miner_out.truncate(remaining.saturating_sub(1));
                miner_out.push('…');
            } else {
                miner_out.truncate(remaining);
            }
        }

        Spans::from(vec![
            Span::styled(
                height_str,
                Style::default().fg(C_LAST20_HEIGHT_LABEL)
                .add_modifier(Modifier::BOLD),
            ),
            Span::raw(spacer),
            Span::styled(
                miner_out,
                Style::default().fg(C_LAST20_MINER_LABEL)
                .add_modifier(Modifier::DIM),
            ),
        ])
    }

    // Split into up to 10 left + 10 right
    let left_rows = rows.iter().take(10);
    let right_rows = rows.iter().skip(10).take(10);

    let left_text: Vec<Spans> = left_rows
        .map(|(h, m)| fmt_line(cols[0].width, *h, m.as_deref()))
        .collect();

    let right_text: Vec<Spans> = right_rows
        .map(|(h, m)| fmt_line(cols[1].width, *h, m.as_deref()))
        .collect();

    let left_para = Paragraph::new(left_text)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    let right_para = Paragraph::new(right_text)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });

    frame.render_widget(left_para, cols[0]);
    frame.render_widget(right_para, cols[1]);
}


