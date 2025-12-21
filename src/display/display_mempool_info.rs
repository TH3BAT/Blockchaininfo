// display/display_mempool_info.rs
//
// Mempool dashboard renderer.
//
// This module is responsible for drawing the entire "Mempool" section
// of the BlockchainInfo TUI. It shows:
// - Loading spinner while initial data is collected
// - Mempool memory usage gauge
// - Flashing transaction count (global mempool size)
// - Optional "dust-free" decoration on the transaction line
// - Size / Age / RBF distributions (with percent + ASCII progress bars)
// - Fee metrics (average, median, fee rate)
//
// This file is *display only* — it does not perform any mempool
// sampling or filtering logic, it simply renders what models provide.

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use num_format::{Locale, ToFormattedString};
use crate::{
    models::mempool_info::{MempoolDistribution, MempoolInfo},
    utils::{format_size, normalize_percentages},
    ui::colors::*,
};
use crate::models::errors::MyError;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::models::flashing_text::TRANSACTION_TEXT;

// Global spinner state for the "Searching through the Dust..." loading view.
// SPINNER_INDEX tracks the current frame index, SPINNER_FRAMES is the ASCII loop.
static SPINNER_INDEX: AtomicUsize = AtomicUsize::new(0);
const SPINNER_FRAMES: [&str; 4] = ["|", "/", "-", "\\"];

const SATS_PER_BTC: f64 = 100_000_000.0;


/// Displays the mempool information in a `tui` terminal.
///
/// This function:
/// - Shows a loading spinner while mempool distribution is still initializing
/// - Renders a gauge for mempool memory usage
/// - Updates and displays a flashing transaction counter
/// - Optionally decorates transaction line with "dust-free" percentage
/// - Builds distribution panels for Size / Age / RBF
/// - Displays fee metrics (avg / median / fee rate)
///
/// `area` is the layout region this section should occupy.
pub fn display_mempool_info<B: Backend>(
    mempool_info: &MempoolInfo,
    distribution: &MempoolDistribution,
    dust_free: bool,
    frame: &mut Frame<B>,
    area: Rect,
) -> Result<(), MyError> {

    // -----------------------------------------------------------------------
    // 1. LOADING STATE
    // -----------------------------------------------------------------------
    // If all the key distribution buckets are zero, treat it as "still warming up".
    // This avoids drawing empty charts and instead shows an animated status line.
    let is_loading = distribution.small == 0
        && distribution.medium == 0
        && distribution.large == 0
        && distribution.rbf_count == 0;

    if is_loading {
        // Rotate through spinner frames using a global atomic index.
        let spinner =
            SPINNER_FRAMES[SPINNER_INDEX.load(Ordering::Relaxed) % SPINNER_FRAMES.len()];
        SPINNER_INDEX.fetch_add(1, Ordering::Relaxed);

        // Centered "Searching through the Dust..." message while mempool scanner runs.
        let loading_text = Paragraph::new(format!("{} Searching through the Dust...", spinner))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(loading_text, area);
        return Ok(());
    }

    // -----------------------------------------------------------------------
    // 2. MEMORY USAGE & COLOR CODING
    // -----------------------------------------------------------------------
    // Convert raw byte usage to human-readable string (e.g. "23.4 MiB").
    let mempool_size_in_memory = format_size(mempool_info.usage);
    let max_mempool_size_in_memory = format_size(mempool_info.maxmempool);

    // Compute mempool fullness percentage for the usage gauge.
    let mempool_usage_percent =
        (mempool_info.usage as f64 / mempool_info.maxmempool as f64) * 100.0;

    // Color ramp for current mempool usage:
    // - < 33%  → Gray (low)
    // - < 66%  → Yellow (medium)
    // - >= 66% → Red (high)
    let mempool_size_in_memory_color = if mempool_info.usage < mempool_info.maxmempool / 3 {
        Style::default().fg(Color::Gray)
    } else if mempool_info.usage < 2 * mempool_info.maxmempool / 3 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Red)
    };

    // Node's minimum relay fee in vSats/vByte (local policy baseline).
    let min_relay_fee_vsats = mempool_info.min_relay_tx_fee_vsats();

    // Dust-free percentage:
    // Ratio of [small + medium + large] bucket counts to total mempool tx count.
    let total_size = distribution.small + distribution.medium + distribution.large;
    let dust_free_percentage = (total_size as f64 / mempool_info.size as f64) * 100.0;
    let formatted_dust_free = format!("{:.1}%", dust_free_percentage);

    // -----------------------------------------------------------------------
    // 3. NORMALIZED DISTRIBUTIONS (SIZE / AGE / RBF)
    // -----------------------------------------------------------------------
    // We normalize each set of counts into percentages so that:
    // - Each category row has a 0–100% value
    // - We can reuse the same 10-character ASCII progress bar helper.

    // Size Distribution (Small / Medium / Large).
    let size_counts = vec![
        distribution.small as u64,
        distribution.medium as u64,
        distribution.large as u64,
    ];

    let size_pcts = normalize_percentages(&size_counts);
    let small_pct = size_pcts[0];
    let medium_pct = size_pcts[1];
    let large_pct = size_pcts[2];

    // Age Distribution (Young / Moderate / Old).
    let age_counts = vec![
        distribution.young as u64,
        distribution.moderate as u64,
        distribution.old as u64,
    ];

    let age_pcts = normalize_percentages(&age_counts);
    let young_pct = age_pcts[0];
    let moderate_pct = age_pcts[1];
    let old_pct = age_pcts[2];

    // RBF Distribution (RBF / Non-RBF).
    let rbf_counts = vec![
        distribution.rbf_count as u64,
        distribution.non_rbf_count as u64,
    ];

    let rbf_pcts = normalize_percentages(&rbf_counts);
    let rbf_pct = rbf_pcts[0];
    let non_rbf_pct = rbf_pcts[1];

    // -----------------------------------------------------------------------
    // 4. PROGRESS BAR STRINGS FOR DISTRIBUTIONS
    // -----------------------------------------------------------------------
    // Using a unified width=10 ASCII bar so the panel remains compact and aligned.
    let small_prog_bar = create_progress_bar(small_pct, 10);
    let medium_prog_bar = create_progress_bar(medium_pct, 10);
    let large_prog_bar = create_progress_bar(large_pct, 10);
    let young_prog_bar = create_progress_bar(young_pct, 10);
    let moderate_prog_bar = create_progress_bar(moderate_pct, 10);
    let old_prog_bar = create_progress_bar(old_pct, 10);
    let rbf_prog_bar = create_progress_bar(rbf_pct, 10);
    let non_rbf_prog_bar = create_progress_bar(non_rbf_pct, 10);

    // -----------------------------------------------------------------------
    // 5. FLASHING TRANSACTION COUNT (GLOBAL MEMPOOL SIZE)
    // -----------------------------------------------------------------------
    // Update the FlashingText state with latest mempool size.
    // This controls how the transactions number pulses on the dashboard.
    TRANSACTION_TEXT.lock().unwrap().update(mempool_info.size);

    // Retrieve the style for current FlashingText frame (e.g. color/weight).
    let transaction_style = TRANSACTION_TEXT.lock().unwrap().style();

    // Build the "📊 Transactions: N" line.
    // Optional dust-free decoration is appended if the toggle is ON.
    let mut spans: Vec<Span> = vec![
        Span::styled("📊 Transactions: ", Style::default().fg(C_MAIN_LABELS)),
        Span::styled(
            mempool_info.size.to_formatted_string(&Locale::en),
            transaction_style,
        ),
    ];

    // Only show dust-free metrics if toggle is ON.
    if dust_free {
        spans.push(Span::styled(" | ", Style::default().fg(C_SEPARATORS)));
        spans.push(Span::styled(
            format!("{} ", formatted_dust_free),
            Style::default().fg(C_DUST_FREE_PCT),
        ));
        spans.push(
            Span::styled(
                "dᵤₛₜ₋fᵣₑₑ",
                Style::default()
                    .fg(C_DUST_FREE_LABEL)
                    .add_modifier(Modifier::ITALIC),
            ),
        );
    }

    let transaction_spans = Spans::from(spans);

    // -----------------------------------------------------------------------
    // 6. LAYOUT (HEADER / GAUGE / CONTENT)
    // -----------------------------------------------------------------------
    // This section is given a single Rect `area` by the parent.
    // Internally, we split it into:
    // - Row 0: header (currently just spacing/styling stub)
    // - Row 1: mempool usage gauge
    // - Row 2+: full content block (all spans stacked vertically)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section (only title or spacing).
                Constraint::Length(3),  // Gauge section.
                Constraint::Min(5),     // Content section.
            ]
            .as_ref(),
        )
        .split(area);

    // Render header (no text yet, but keeps layout consistent with other sections).
    let header = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // -----------------------------------------------------------------------
    // 7. MEMPOOL MEMORY USAGE GAUGE
    // -----------------------------------------------------------------------
    // Shows mempool usage as a percentage of maxmempool, with a labeled border.
    let mempool_gauge = Gauge::default()
        .block(Block::default().title("Mempool Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(C_MEMPOOL_USAGE_GAUGE_FG).bg(C_MEMPOOL_USAGE_GAUGE_BG))
        .percent(mempool_usage_percent as u16);
    frame.render_widget(mempool_gauge, chunks[1]);

    // -----------------------------------------------------------------------
    // 8. MAIN CONTENT: COUNTS, DISTRIBUTIONS, FEE METRICS
    // -----------------------------------------------------------------------
    // All the remaining lines are stacked inside a Paragraph.
    let mempool_content = vec![
        // Flashing transaction count (with optional dust-free tag).
        transaction_spans,

        // Memory usage breakdown: current vs max.
        Spans::from(vec![
            Span::styled("💾 Memory: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(
                format!("{} ", mempool_size_in_memory),
                mempool_size_in_memory_color,
            ),
            Span::styled(
                format!("/ {}", max_mempool_size_in_memory),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
        ]),

        // Total fees currently sitting in the mempool (BTC).
        Spans::from(vec![
            Span::styled("💰 Total Fees: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(
                format!("{:.8}", mempool_info.total_fee),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
        ]),

        // Local node minimum relay fee (vsats/vByte).
        Spans::from(vec![
            Span::styled("⚖️ Min Transaction Fee: ", Style::default().fg(C_MAIN_LABELS)),
            Span::styled(
                min_relay_fee_vsats.to_formatted_string(&Locale::en),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(" vSats/vByte", Style::default().fg(C_MAIN_LABELS)),
        ]),

        // -------------------------------------------------------------------
        // SIZE DISTRIBUTION
        // -------------------------------------------------------------------
        Spans::from(vec![
            Span::styled("📏 Size Distribution ", Style::default().fg(C_MAIN_LABELS)),
            // Optional "dust-free" tag is commented out here; preserved for future use.
            // Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC)),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🔹 Small (< 250 vBytes)     ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:>7}", (distribution.small).to_formatted_string(&Locale::en)),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(C_SEPARATORS)),
            Span::styled(
                format!("{:>3}% {}", small_pct, small_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🔸 Medium (250-1000 vBytes) ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:>7}", (distribution.medium).to_formatted_string(&Locale::en)),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>3}% {}", medium_pct, medium_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🔳 Large (> 1000 vBytes)    ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:>7}", (distribution.large).to_formatted_string(&Locale::en)),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(C_SEPARATORS)),
            Span::styled(
                format!("{:>3}% {}", large_pct, large_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),

        // -------------------------------------------------------------------
        // AGE DISTRIBUTION
        // -------------------------------------------------------------------
        Spans::from(vec![
            Span::styled("⏳ Age Distribution ", Style::default().fg(C_MAIN_LABELS)),
            //Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC)),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🟢 Young (< 5 min)          ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:>7}", (distribution.young).to_formatted_string(&Locale::en)),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(C_SEPARATORS)),
            Span::styled(
                format!("{:>3}% {}", young_pct, young_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🟡 Moderate (5 min - 1 hr)  ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.moderate).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(C_SEPARATORS)),
            Span::styled(
                format!("{:>3}% {}", moderate_pct, moderate_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🔴 Old (> 1 hr)             ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:>7}", (distribution.old).to_formatted_string(&Locale::en)),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(C_SEPARATORS)),
            Span::styled(
                format!("{:>3}% {}", old_pct, old_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),

        // -------------------------------------------------------------------
        // RBF DISTRIBUTION
        // -------------------------------------------------------------------
        Spans::from(vec![
            Span::styled("♻️ RBF Distribution ", Style::default().fg(C_MAIN_LABELS)),
            //Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC)),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🔄 RBF Transactions         ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.rbf_count).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(C_SEPARATORS)),
            Span::styled(
                format!("{:>3}% {}", rbf_pct, rbf_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "  ✅ Non-RBF Transactions     ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.non_rbf_count).to_formatted_string(&Locale::en)
                ),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
            Span::styled(" - ", Style::default().fg(C_SEPARATORS)),
            Span::styled(
                format!("{:>3}% {}", non_rbf_pct, non_rbf_prog_bar),
                Style::default().fg(C_HORIZONTAL_ASCII_BAR),
            ),
        ]),

        // -------------------------------------------------------------------
        // FEE METRICS
        // -------------------------------------------------------------------
        Spans::from(vec![
            Span::styled("📉 Fee Metrics ", Style::default().fg(C_MAIN_LABELS)),
        ]),
        Spans::from(vec![
            Span::styled(
                "  📊 Average Fee (BTC): ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:.8}", distribution.average_fee as f64 / SATS_PER_BTC),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "  📊 Median Fee (BTC) : ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:.8}", distribution.median_fee as f64 / SATS_PER_BTC),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
        ]),
        Spans::from(vec![
            Span::styled(
                "  🎯 Average Fee Rate (sats/vByte): ",
                Style::default().fg(C_MEMPOOL_DIST_LABELS),
            ),
            Span::styled(
                format!("{:.2}", distribution.average_fee_rate),
                Style::default().fg(C_MEMPOOL_VALUES),
            ),
        ]),
    ];

    // Wrap all content lines inside a Paragraph and render into the content chunk.
    let mempool_paragraph = Paragraph::new(mempool_content)
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(mempool_paragraph, chunks[2]);

    Ok(())
}

/// Helper function to create a visual progress bar used alongside percent metrics.
///
/// Example:
///   percent = 40, width = 10  →  "[====      ]"
///
/// The bar length is fixed by `width`; the number of '=' characters is
/// proportional to `percent` (rounded to nearest).
fn create_progress_bar(percent: u64, width: u16) -> String {
    let filled = (percent as f64 / 100.0 * width as f64).round() as usize;
    let empty = width as usize - filled;
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}
