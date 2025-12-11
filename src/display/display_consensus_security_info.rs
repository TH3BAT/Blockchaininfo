// display/display_consensus_security_info.rs
//
// Renders the "Consensus Security" section of the dashboard.
// This section displays active chain tips and fork information,
// helping users visually monitor whether unexpected chains appear
// (e.g., stale forks, competing tips, potential re-org signals).
//
// All processing here is presentation-only; the underlying chaintips
// were already retrieved and deserialized inside the RPC subsystem.
//

use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    layout::{Constraint, Direction, Layout},
};
use crate::models::chaintips_info::ChainTip;
use crate::models::errors::MyError;

/// Draws the Consensus Security panel.
///
/// This panel uses Bitcoin Core's `getchaintips` data to highlight:
///   • The active chain tip  
///   • Any "valid-fork" tips (stale forks)  
///   • Their heights and branch lengths  
///
/// Only the active chain + top two forks are displayed to keep the UI compact.
/// The frame & area are passed by `runapp.rs`.
pub fn display_consensus_security_info<B: tui::backend::Backend>(
    chaintips_info: &Vec<ChainTip>,
    frame: &mut tui::Frame<B>,
    area: tui::layout::Rect,
) -> Result<(), MyError> {

    // Split the provided area into header and content.
    //
    // [ Header ]
    // [ Fork monitoring content ]
    //
    // Layout is consistent with the other dashboard sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1), // Space reserved for the header
                Constraint::Min(5),    // Main fork-monitoring content
            ]
            .as_ref(),
        )
        .split(area);

    // Header block (empty title, only style/border presence).
    // This maintains consistency with section formatting across the UI.
    let header = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(header, chunks[0]);

    // ===== Build the text lines for the panel =====
    let mut lines = Vec::new();

    // Section label
    lines.push(Spans::from(vec![
        Span::styled(
            "🌲 Fork Monitoring:",
            Style::default().fg(Color::Gray),
        ),
    ]));

    // Filter only the relevant tips:
    //
    //   "active"      → the main chain
    //   "valid-fork"  → recognized but stale forks
    //
    // Other statuses (valid-headers, invalid, etc.) generally clutter
    // the panel and rarely provide useful real-time signal for operators.
    let mut filtered_tips: Vec<&ChainTip> = chaintips_info
        .iter()
        .filter(|tip| tip.status == "active" || tip.status == "valid-fork")
        .collect();

    // Sort by block height descending so highest tips appear first.
    filtered_tips.sort_by(|a, b| b.height.cmp(&a.height));

    // Only show 3 entries max: the active chain + two highest forks.
    let limited_tips = filtered_tips.into_iter().take(3);

    // Convert each tip into a formatted TUI line.
    for tip in limited_tips {
        // Human-readable labels
        let status = match tip.status.as_str() {
            "active"        => "⚡ Active Chain",
            "valid-fork"    => "❌ Stale Fork",
            "valid-headers" => "Headers Only",
            "unknown"       => "Unknown",
            _               => "Other",
        };

        // Compose a structured row:
        //
        // 🌳 Height: ####### | Status: <label> | 📏 Length: ##
        //
        // Colors:
        //   - Gray  → Neutral / structural numbers
        //   - Yellow → Highlights the fork status
        let line = Spans::from(vec![
            Span::styled(
                format!("🌳 Height: {:>7}", tip.height),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("Status: {:<14}", status),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("📏 Length: {:>2}", tip.branchlen),
                Style::default().fg(Color::Gray),
            ),
        ]);

        lines.push(line);
    }

    // Render the text block into the lower layout chunk.
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, chunks[1]);

    Ok(())
}
