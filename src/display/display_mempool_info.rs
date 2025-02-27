
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
use crate::{models::mempool_info::{MempoolDistribution, MempoolInfo}, utils::format_size};
use crate::models::errors::MyError;
use std::sync::atomic::{AtomicUsize, Ordering};

static SPINNER_INDEX: AtomicUsize = AtomicUsize::new(0);
const SPINNER_FRAMES: [&str; 4] = ["|", "/", "-", "\\"];

// Displays the mempool information in a `tui` terminal.
pub fn display_mempool_info<B: Backend>(
    mempool_info: &MempoolInfo,
    distribution: &MempoolDistribution,
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

    let (small_pct, medium_pct, large_pct) = calculate_rounded_percentages(distribution.small.try_into().unwrap(), distribution.medium.try_into().unwrap(), Some(distribution.large.try_into().unwrap()), total_size.try_into().unwrap());
    let (young_pct, moderate_pct, old_pct) = calculate_rounded_percentages(distribution.young.try_into().unwrap(), distribution.moderate.try_into().unwrap(), Some(distribution.old.try_into().unwrap()), total_size.try_into().unwrap());
    let (rbf_pct, non_rbf_pct, _) = calculate_rounded_percentages(distribution.rbf_count.try_into().unwrap(), distribution.non_rbf_count.try_into().unwrap(), None, total_size.try_into().unwrap());

    let small_prog_bar = create_progress_bar(small_pct, 10);
    let medium_prog_bar = create_progress_bar(medium_pct, 10);
    let large_prog_bar = create_progress_bar(large_pct.unwrap_or(0), 10);
    let young_prog_bar = create_progress_bar(young_pct, 10);
    let moderate_prog_bar = create_progress_bar(moderate_pct, 10);
    let old_prog_bar = create_progress_bar(old_pct.unwrap_or(0), 10);
    let rbf_prog_bar = create_progress_bar(rbf_pct, 10);
    let non_rbf_prog_bar = create_progress_bar(non_rbf_pct, 10);

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
        Spans::from(vec![
            Span::styled("📊 Transactions: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!(
                    "{} ",
                    mempool_info.size.to_formatted_string(&Locale::en),
                ),
                Style::default().fg(Color::Green),
            ),
            Span::styled("| ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} ", formatted_dust_free),
                Style::default().fg(Color::Gray), // Dust-free percentage in Gray
            ),
            Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)),
        ]),
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
            Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)),
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
                    large_pct.unwrap_or(0), large_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),

        // Age Distribution.
        Spans::from(vec![
            Span::styled("⏳ Age Distribution ", Style::default().fg(Color::Gray)),
            Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)),
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
                    old_pct.unwrap_or(0), old_prog_bar
                ),
                Style::default().fg(Color::Gray),
            ),
        ]),

        // RBF Distribution.
        Spans::from(vec![
            Span::styled("♻️ RBF Distribution ", Style::default().fg(Color::Gray)),
            Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)),
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
            Span::styled("dᵤₛₜ₋fᵣₑₑ", Style::default().fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)),
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


fn calculate_rounded_percentages(first: u64, second: u64, third: Option<u64>, total_size: u64) -> (u64, u64, Option<u64>) {
    if total_size == 0 {
        return (0, 0, third.map(|_| 0)); // Avoid division by zero
    }

    // Calculate raw percentages
    let first_pct = (first * 100) as f64 / total_size as f64;
    let second_pct = (second * 100) as f64 / total_size as f64;
    let third_pct = third.map(|t| (t * 100) as f64 / total_size as f64);

    // Floor the percentages
    let mut first_floor = first_pct.floor() as u64;
    let mut second_floor = second_pct.floor() as u64;
    let mut third_floor = third_pct.map(|p| p.floor() as u64);

    // Calculate remainders
    let mut remainders = vec![
        (first_pct - first_floor as f64, "first"),
        (second_pct - second_floor as f64, "second"),
    ];

    if let Some(pct) = third_pct {
        remainders.push((pct - third_floor.unwrap() as f64, "third"));
    }

    // Sort by remainder in descending order
    remainders.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Calculate the total of floored percentages
    let mut total = first_floor + second_floor + third_floor.unwrap_or(0);

    // Distribute the remainder
    for (_remainder, category) in remainders {
        if total >= 100 {
            break;
        }
        match category {
            "first" => first_floor += 1,
            "second" => second_floor += 1,
            "third" => {
                if let Some(t) = third_floor.as_mut() {
                    *t += 1;
                }
            }
            _ => unreachable!(),
        }
        total += 1;
    }

    (first_floor, second_floor, third_floor)
}


fn create_progress_bar(percent: u64, width: u16) -> String {
    let filled = (percent as f64 / 100.0 * width as f64).round() as usize;
    let empty = width as usize - filled;
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}