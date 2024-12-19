
// display/display_network_info.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::models::network_info::NetworkInfo;
use crate::models::errors::MyError;

// Displays the network information in a `tui` terminal.
pub fn display_network_info<B: Backend>(
    frame: &mut Frame<B>,
    network_info: &NetworkInfo,
    area: Rect, // Accept area parameter.
) -> Result<(), MyError> {
    // Define layout for the network info, using the passed area.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Header section (only title).
                Constraint::Min(5),     // Content section.
            ]
            .as_ref(),
        )
        .split(area);

    // Render header
    let header = Block::default()
        .borders(Borders::NONE) // Show borders for the header.
        .style(Style::default().fg(Color::Cyan)); // Style for the borders (Cyan color).
    frame.render_widget(header, chunks[0]);


    // Network information content (without repeating title).
    let network_content = vec![
        Spans::from(vec![
            Span::styled("Connections in: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network_info.connections_in.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::styled("Connections out: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network_info.connections_out.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];

    // No borders for empty sections.
    let network_paragraph = Paragraph::new(network_content)
        .block(Block::default().borders(Borders::NONE)); // No border.
    
    frame.render_widget(network_paragraph, chunks[1]);

    Ok(())
}
