
// display/display_mempool_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use num_format::{Locale, ToFormattedString};
use crate::{models::mempool_info::{MempoolDistribution, MempoolInfo}, utils::{format_size,
    normalize_percentages}};
use crate::models::errors::MyError;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::models::flashing_text::TRANSACTION_TEXT;

static SPINNER_INDEX: AtomicUsize = AtomicUsize::new(0);
const SPINNER_FRAMES: [&str; 4] = ["|", "/", "-", "\\"];

// Displays the mempool information in a `tui` terminal.
pub fn display_mempool_info<B: Backend>(
    mempool_info: &MempoolInfo,
    distribution: &MempoolDistribution,
    dust_free: bool,
    frame: &mut Frame<B>,
    area: Rect, 
) -> Result<(), MyError> {

    // Check if data is still initializing
    let is_loading = distribution.small == 0 && distribution.medium == 0 && distribution.large == 0 && distribution.rbf_count == 0;

    if is_loading {
        let spinner = SPINNER_FRAMES[SPINNER_INDEX.load(Ordering::Relaxed) % SPINNER_FRAMES.len()];
        SPINNER_INDEX.fetch_add(1, Ordering::Relaxed);

        let loading_text = Paragraph::new(format!("{} Searching through the Dust...", spinner))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(loading_text, area);
        return Ok(());
    }

    // Calculate formatted and colored memory usage.
    let mempool_size_in_memory = format_size(mempool_info.usage);
    let max_mempool_size_in_memory = format_size(mempool_info.maxmempool);

    let mempool_usage_percent = (mempool_info.usage as f64 / mempool_info.maxmempool as f64) * 100.0;

    let mempool_size_in_memory_color = if mempool_info.usage < mempool_info.maxmempool / 3 {
        Style::default().fg(Color::Gray)
    } else if mempool_info.usage < 2 * mempool_info.maxmempool / 3 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Red)
    };

    let min_relay_fee_vsats = mempool_info.min_relay_tx_fee_vsats();
    let total_size = distribution.small + distribution.medium + distribution.large;
    let dust_free_percentage = (total_size as f64 / mempool_info.size as f64) * 100.0;
    let formatted_dust_free = format!("{:.1}%", dust_free_percentage);
    
    // Size Distribution (Small / Medium / Large)
    let size_counts = vec![
        distribution.small as u64,
        distribution.medium as u64,
        distribution.large as u64,
    ];

    let size_pcts = normalize_percentages(&size_counts);
    let small_pct  = size_pcts[0];
    let medium_pct = size_pcts[1];
    let large_pct  = size_pcts[2];

    // Age Distribution (Young / Moderate / Old)
    let age_counts = vec![
        distribution.young as u64,
        distribution.moderate as u64,
        distribution.old as u64,
    ];

    let age_pcts = normalize_percentages(&age_counts);
    let young_pct    = age_pcts[0];
    let moderate_pct = age_pcts[1];
    let old_pct      = age_pcts[2];

    // RBF Distribution (RBF / Non-RBF)
    let rbf_counts = vec![
        distribution.rbf_count as u64,
        distribution.non_rbf_count as u64,
    ];

    let rbf_pcts = normalize_percentages(&rbf_counts);
    let rbf_pct     = rbf_pcts[0];
    let non_rbf_pct = rbf_pcts[1];

    let small_prog_bar = create_progress_bar(small_pct, 10);
    let medium_prog_bar = create_progress_bar(medium_pct, 10);
    let large_prog_bar = create_progress_bar(large_pct, 10);
    let young_prog_bar = create_progress_bar(young_pct, 10);
    let moderate_prog_bar = create_progress_bar(moderate_pct, 10);
    let old_prog_bar = create_progress_bar(old_pct, 10);
    let rbf_prog_bar = create_progress_bar(rbf_pct, 10);
    let non_rbf_prog_bar = create_progress_bar(non_rbf_pct, 10);

     // Update the FlashingText variable
     TRANSACTION_TEXT.lock().unwrap().update(mempool_info.size);

     // Get the style for the FlashingText
     let transaction_style = TRANSACTION_TEXT.lock().unwrap().style();
    
    let mut spans: Vec<Span> = vec![
        Span::styled("📊 Transactions: ", Style::default().fg(Color::Gray)),
        Span::styled(
            mempool_info.size.to_formatted_string(&Locale::en),
            transaction_style,
        ),
    ];

    // Only show dust-free metrics if toggle is ON
    if dust_free {
        spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(
            format!("{} ", formatted_dust_free),
            Style::default().fg(Color::Gray),
        ));
        spans.push(
            Span::styled(
                "dᵤₛₜ₋fᵣₑₑ",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )
        );
    }

    let transaction_spans = Spans::from(spans);

    // Create the layout for this specific chunk (using passed 'area').
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section (only title).
                Constraint::Length(3),  // Gauge section.
                Constraint::Min(5),     // Content section.
            ]
            .as_ref(),
        )
        .split(area);

    // Render header
    let header = Block::default()
        .borders(Borders::NONE) 
        .style(Style::default().fg(Color::Cyan)); 
    frame.render_widget(header, chunks[0]);

    // Render the gauge for mempool memory usage.
    let mempool_gauge = Gauge::default()
        .block(Block::default().title("Mempool Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::DarkGray).bg(Color::Black))
        .percent(mempool_usage_percent as u16);
    frame.render_widget(mempool_gauge, chunks[1]);

    let mempool_content = vec![
        transaction_spans,
        Spans::from(vec![
            Span::styled("💾 Memory: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} ", mempool_size_in_memory),
                mempool_size_in_memory_color,
            ),
            Span::styled(format!("/ {}", max_mempool_size_in_memory),
            Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("💰 Total Fees: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.8}", mempool_info.total_fee),
            Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("⚖️ Min Transaction Fee: ", Style::default().fg(Color::Gray)),
            Span::styled(
                min_relay_fee_vsats.to_formatted_string(&Locale::en),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(" vSats/vByte", Style::default().fg(Color::Gray)),
        ]), 
         // Size Distribution.
        Spans::from(vec![
            Span::styled("📏 Size Distribution ", Style::default().fg(Color::Gray)),
            //Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC)),
            ]),
        Spans::from(vec![
            Span::styled("  🔹 Small (< 250 vBytes)     ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.small).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    small_pct, small_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  🔸 Medium (250-1000 vBytes) ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.medium).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    medium_pct, medium_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  🔳 Large (> 1000 vBytes)    ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.large).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    large_pct, large_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),

        // Age Distribution.
        Spans::from(vec![
            Span::styled("⏳ Age Distribution ", Style::default().fg(Color::Gray)),
            //Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC)),
            ]),
        Spans::from(vec![
            Span::styled("  🟢 Young (< 5 min)          ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.young).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    young_pct, young_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  🟡 Moderate (5 min - 1 hr)  ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.moderate).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    moderate_pct, moderate_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  🔴 Old (> 1 hr)             ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.old).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    old_pct, old_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),

        // RBF Distribution.
        Spans::from(vec![
            Span::styled("♻️ RBF Distribution ", Style::default().fg(Color::Gray)),
            //Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC)),
            ]),
        Spans::from(vec![
            Span::styled("  🔄 RBF Transactions         ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.rbf_count).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    rbf_pct, rbf_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("  ✅ Non-RBF Transactions     ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "{:>7}",
                    (distribution.non_rbf_count).to_formatted_string(&Locale::en)),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" - ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(
                    "{:>3}% {}",
                    non_rbf_pct, non_rbf_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("📉 Fee Metrics ", Style::default().fg(Color::Gray)),
            //Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC)),
            ]),
        Spans::from(vec![
            Span::styled("  📊 Average Fee (BTC): ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.8}", distribution.average_fee),
                Style::default().fg(Color::Gray),
            ),
        ]),

        Spans::from(vec![
            Span::styled("  📊 Median Fee (BTC) : ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.8}", distribution.median_fee),
                Style::default().fg(Color::Gray),
            ),
        ]),
        
        Spans::from(vec![
            Span::styled("  🎯 Average Fee Rate (sats/vByte): ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{:.2}", distribution.average_fee_rate),
                Style::default().fg(Color::Gray),
            ),
        ]),    
    ];

    let mempool_paragraph = Paragraph::new(mempool_content)
        .block(Block::default().borders(Borders::NONE)); 
    
    frame.render_widget(mempool_paragraph, chunks[2]);

    Ok(())
}

/// Helper function to create a visual progres bar used alongside percent metrics.
fn create_progress_bar(percent: u64, width: u16) -> String {
    let filled = (percent as f64 / 100.0 * width as f64).round() as usize;
    let empty = width as usize - filled;
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}