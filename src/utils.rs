
// utils.rs

use std::process::Command;
use crate::models::errors::MyError;
use tui::widgets::{Block, Borders, Paragraph};
use tui::text::{Span, Spans};
use tui::style::{Color, Style, Modifier};
use tui::layout::{Rect, Alignment};
use tui::Frame;
use tui::backend::Backend;
use crate::models::peer_info::PeerInfo;
use std::collections::HashMap;

// Constants for bytes formatting.
const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;
const TB: u64 = GB * 1024;

// Constants for estimated difficulty adjustment change.
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2016;
pub const BLOCK_TIME_SECONDS: u64 = 600;


// Formats a size in bytes into a more readable format (KB, MB, etc.).
pub fn format_size(bytes: u64) -> String {
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// Retrieves the RPC password stored in macOS Keychain.
#[cfg(target_os = "macos")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    let output = Command::new("security")
        .arg("find-generic-password")
        .arg("-s")
        .arg("rpc-password")
        .arg("-a")
        .arg("bitcoin")
        .arg("-w")
        .output()
        .map_err(|e| MyError::Keychain(format!("Keychain access error: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr).to_string();
        Err(MyError::Keychain(format!("Password not found in keychain: {}", error_message)))
    }
}

// Linux-specific logic (placeholder, implement accordingly).
#[cfg(target_os = "linux")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Linux keyring access not supported".to_string()))
}

// Windows-specific logic (placeholder, implement accordingly).
#[cfg(target_os = "windows")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Windows keychain access not supported".to_string()))
}

// Fallback for unsupported OS.
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Unsupported OS for keychain access".to_string()))
}

// Estimate the current epoch's difficulty change and return as a percentage.
pub fn estimate_difficulty_change(
    current_block_height: u64,
    current_block_time: u64,
    epoch_start_block_time: u64,
) -> f64 {
    let blocks_in_epoch = (current_block_height % DIFFICULTY_ADJUSTMENT_INTERVAL) - 1;
    let expected_duration = blocks_in_epoch * BLOCK_TIME_SECONDS;
    let actual_duration = current_block_time - epoch_start_block_time;

    let adjustment_factor = expected_duration as f64 / actual_duration as f64;
    (adjustment_factor - 1.0) * 100.0 // Return percentage change
} 
// Returns a `tui` widget for the blockchain header.
pub fn display_header_widget() -> Paragraph<'static> {
    // Create the header lines.
    let lines = vec![
        Spans::from(Span::styled(
            r"__________.__                 __          .__           .__       .__        _____       ",
            Style::default().fg(Color::Gray),
        )),
        Spans::from(Span::styled(
            r"\______   \  |   ____   ____ |  | __ ____ |  |__ _____  |__| ____ |__| _____/ ____\____   ",
            Style::default().fg(Color::Gray),
        )),
        Spans::from(Span::styled(
            r" |    |  _/  |  /  _ \_/ ___\|  |/ // ___\|  |  \\__  \ |  |/    \|  |/    \   __\/  _ \ ",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Spans::from(vec![
            Span::styled(
            r" |    |   \  |_(  <_> )  \___|    <\  \___|   Y  \/ __ \|  |   |  \  |   |  \  | (  <_> ) ",
            Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("v{}", env!("CARGO_PKG_VERSION").to_string()),
                Style::default().fg(Color::DarkGray),
            )
        ]),
        Spans::from(Span::styled(
            r" |______  /____/\____/ \___  >__|_ \\___  >___|  (____  /__|___|  /__|___|  /__|  \____/",
            Style::default().fg(Color::LightBlue),
        )),
        Spans::from(Span::styled(
            r"        \/                 \/     \/    \/     \/     \/        \/        \/             ",
            Style::default().fg(Color::Gray),
        )),
    ];

    // Create the paragraph widget.
    Paragraph::new(lines)
        .block(Block::default().title("").borders(Borders::NONE))
}

pub fn render_footer<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let footer = Paragraph::new("Press 'q' or ESC to quit.")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, area);
}

/*
// Helper function to extract the version number as a tuple
fn extract_version(subver: &str) -> (u32, u32, u32) {
    let version_pattern = regex::Regex::new(r"/Satoshi:(\d+)\.(\d+)\.(\d+)").unwrap();
    if let Some(captures) = version_pattern.captures(subver) {
        let major = captures.get(1).map_or(0, |m| m.as_str().parse::<u32>().unwrap_or(0));
        let minor = captures.get(2).map_or(0, |m| m.as_str().parse::<u32>().unwrap_or(0));
        let patch = captures.get(3).map_or(0, |m| m.as_str().parse::<u32>().unwrap_or(0));
        (major, minor, patch)
    } else {
        (0, 0, 0) // Default to 0.0.0 if version is not found
    }
}
*/

pub fn normalize_version(subver: &str) -> String {
    let version_pattern = regex::Regex::new(r"/Satoshi:(\d+\.\d+\.\d+)").unwrap();
    if let Some(captures) = version_pattern.captures(subver) {
        captures.get(1).map_or_else(|| "Unknown".to_string(), |m| m.as_str().to_string())
    } else {
        "Unknown".to_string()
    }
}

// Aggregate and sort Node Version Distribution.
pub fn aggregate_and_sort_versions(peer_info: &[PeerInfo]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    // Aggregate peer counts for normalized versions
    for peer in peer_info.iter().filter(|peer| peer.subver.contains("Satoshi")) {
        let normalized_version = normalize_version(&peer.subver); // Stripped-down version
        *counts.entry(normalized_version).or_insert(0) += 1;
    }

    // Convert HashMap to Vec
    let mut sorted_counts: Vec<(String, usize)> = counts.into_iter().collect();

    // Sort by peer count in descending order
    sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));

    sorted_counts
}

pub fn calculate_block_propagation_time(peer_info: &[PeerInfo], best_block_time: u64) -> u64 {
    let mut propagation_times: Vec<u64> = Vec::new(); // Specify u64 for the vector.

    // Iterate over peers and filter those who match the condition.
    for peer in peer_info.iter().filter(|peer| peer.subver.contains("Satoshi")) {
        let mut peer_last_block_timestamp = peer.last_block;
        
        // If the last_block is 0, set it to the best_block_time.
        if peer_last_block_timestamp == 0 {
            peer_last_block_timestamp = best_block_time; // Set to best block time.
        }

        // Calculate the propagation time in seconds (timestamps are assumed to be in seconds).
        let propagation_time_in_seconds = best_block_time - peer_last_block_timestamp;
        
        // Convert to milliseconds (multiply by 1000)
        let propagation_time_in_ms = propagation_time_in_seconds * 1000;

        // Store in the vector
        propagation_times.push(propagation_time_in_ms);
    }
    
    // Calculate the average propagation time in milliseconds.
    let total_peers = propagation_times.len();
    if total_peers == 0 {
        return 0; // Avoid division by zero if no valid peers were found.
    }

    let total_time: u64 = propagation_times.iter().sum(); // Sum of the vector
    let average_propagation_time_in_ms = total_time / total_peers as u64; // Division by number of peers.
    
    // Convert milliseconds to minutes (after calculating the average).
    // If greater than 60 minutes, then we have corrupt data, and will return 99.
    if average_propagation_time_in_ms / 60000 < 60 {
        average_propagation_time_in_ms / 60000 // Convert milliseconds to minutes.
    } else {
        99
    }
}






