
// display/display_consensus_security_info.rs

use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    layout::{Constraint, Direction, Layout},
};
use crate::models::chaintips_info::ChainTip;
use crate::models::errors::MyError;  

pub fn display_consensus_security_info<B: tui::backend::Backend>(
    chaintips_info: &Vec<ChainTip>,
    frame: &mut tui::Frame<B>,
    area: tui::layout::Rect,
) -> Result<(), MyError> {

    // Create the layout for this specific chunk (using passed 'area').
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1), // Header section (only title).
                Constraint::Min(5),    // Content section.
            ]
            .as_ref(),
        )
        .split(area);

    // Render header
    let header = Block::default()
        .borders(Borders::NONE) // Show borders for the header.
        .style(Style::default().fg(Color::Cyan)); // Style for borders (Cyan color).
    frame.render_widget(header, chunks[0]);

    // Prepare content for the TUI display.
    let mut lines = Vec::new();

    // Add a "Fork Monitoring:" subheading.
    lines.push(Spans::from(vec![
        Span::styled(
            "🌲 Fork Monitoring:",
            Style::default().fg(Color::Gray),
        ),
    ]));

    // Filter active chain and valid forks.
    let mut filtered_tips: Vec<&ChainTip> = chaintips_info
        .iter()
        .filter(|tip| tip.status == "active" || tip.status == "valid-fork")
        .collect();

    // Sort by height in descending order (highest block first).
    filtered_tips.sort_by(|a, b| b.height.cmp(&a.height));

    // Keep only the active chain and the last two forks.
    let limited_tips = filtered_tips.into_iter().take(3);

    // Generate lines for the TUI display.
    for tip in limited_tips {
        let status = match tip.status.as_str() {
            "active" => "⚡ Active Chain",
            "valid-fork" => "❌ Stale Fork",
            "valid-headers" => "Headers Only",
            "unknown" => "Unknown",
            _ => "Other",
        };

        let line = Spans::from(vec![
            Span::styled(format!("🌳 Height: {:>7}", tip.height), Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::styled(format!("Status: {:<14}", status), Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("📏 Length: {:>2}", tip.branchlen), Style::default().fg(Color::Gray)),
        ]);
        lines.push(line);
    }

    // Render the content in the second chunk.
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, chunks[1]);

    Ok(())
}
