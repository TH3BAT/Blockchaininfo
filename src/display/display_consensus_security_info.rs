use tui::{
    style::{Color, Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};
use crate::models::chaintips_info::ChainTip;
use crate::models::errors::MyError;  

pub fn display_consensus_security_info<B: tui::backend::Backend>(
    frame: &mut tui::Frame<B>,
    chaintips_info: &[ChainTip],
    area: tui::layout::Rect,
) -> Result<(), MyError> {
    let mut lines = Vec::new();

    // Add a blank line for separation after the title.
    lines.push(Spans::from(vec![]));

    // Add a "Fork Monitoring:" subheading
    lines.push(Spans::from(vec![
        Span::styled(
            "Fork Monitoring:",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
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
            "active" => "Active Chain",
            "valid-fork" => "Stale Fork",
            "valid-headers" => "Headers Only",
            "unknown" => "Unknown",
            _ => "Other",
        };

        let line = Spans::from(vec![
            Span::styled(format!("Height: {:>8}", tip.height), Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::styled(format!("Status: {:<15}", status), Style::default().fg(Color::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("Branch Length: {:>2}", tip.branchlen), Style::default().fg(Color::Gray)),
        ]);
        lines.push(line);
    }

    // Create a block with the title "[Consensus Security]"
    let block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            "[Consensus Security]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));

    // Render the block with the data inside.
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);

    Ok(())
}


