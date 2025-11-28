
// utils.rs

#[cfg(target_os = "macos")]
use std::process::Command;
use crate::models::errors::MyError;
use tui::widgets::{Block, Borders, Paragraph};
use tui::text::{Span, Spans};
use tui::style::{Color, Style, Modifier};
use tui::layout::{Rect, Alignment};
use tui::Frame;
use tui::backend::Backend;
use std::fs::{OpenOptions, metadata, rename};
use std::fs;
use std::io::{self, Write};
use std::sync::Mutex;
use lazy_static::lazy_static;
use chrono::Local;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use std::collections::{HashSet, VecDeque};
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::BlockInfo;
use crate::models::chaintips_info::ChainTipsResponse;
use crate::models::mempool_info::{MempoolDistribution, MempoolInfo};
use crate::models::peer_info::PeerInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::network_totals::NetTotals;
use crate::models::block_info::{BlockHistory, MinersData};
use std::io::Read;

// Constants for bytes formatting.
const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;
const TB: u64 = GB * 1024;

// Constants for estimated difficulty adjustment change.
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2016;
pub const BLOCK_TIME_SECONDS: u64 = 600;
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static BLOCKCHAIN_INFO_CACHE: Lazy<Arc<RwLock<BlockchainInfo>>> = 
    Lazy::new(|| Arc::new(RwLock::new(BlockchainInfo::default())));

pub static MEMPOOL_INFO_CACHE: Lazy<Arc<RwLock<MempoolInfo>>> = 
    Lazy::new(|| Arc::new(RwLock::new(MempoolInfo::default())));

pub static NETWORK_INFO_CACHE: Lazy<Arc<RwLock<NetworkInfo>>> = 
    Lazy::new(|| Arc::new(RwLock::new(NetworkInfo::default())));

pub static PEER_INFO_CACHE: Lazy<Arc<RwLock<Vec<PeerInfo>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub static NET_TOTALS_CACHE: Lazy<Arc<RwLock<NetTotals>>> = 
    Lazy::new(|| Arc::new(RwLock::new(NetTotals::default())));

pub static BLOCK_INFO_CACHE: Lazy<Arc<RwLock<Vec<BlockInfo>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub static BLOCK24_INFO_CACHE: Lazy<Arc<RwLock<Vec<BlockInfo>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

pub static CHAIN_TIP_CACHE: Lazy<Arc<RwLock<ChainTipsResponse>>> =
    Lazy::new(|| Arc::new(RwLock::new(ChainTipsResponse::default())));

pub static MEMPOOL_DISTRIBUTION_CACHE: Lazy<Arc<RwLock<MempoolDistribution>>> =
    Lazy::new(|| Arc::new(RwLock::new(MempoolDistribution::default())));

lazy_static! {
    pub static ref LOGGED_TXS: Lazy<Arc<RwLock<(HashSet<Arc<String>>, VecDeque<Arc<String>>)>>> =
        Lazy::new(|| Arc::new(RwLock::new((HashSet::with_capacity(500), VecDeque::with_capacity(500)))));
}

// Use a Mutex to ensure thread-safe access to the log file
lazy_static! {
    static ref LOG_FILE: Mutex<()> = Mutex::new(()); // Global Mutex for thread-safe logging
}

// For hash distribution (past 24 hours or 144 blocks).
lazy_static! {
    pub static ref BLOCK_HISTORY: Arc<RwLock<BlockHistory>> = Arc::new(RwLock::new(BlockHistory::new()));
}

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

/// Retrieves the RPC password stored in macOS Keychain.
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
        .map_err(|e| MyError::Keychain(format!("Failed to retrieve password: {}", e)))?;

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

/// Estimate the current epoch's difficulty change and return as a percentage.
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

/// Estimate the current past 24 hrs difficulty change and return as a percentage.
pub fn estimate_24h_difficulty_change(
    current_block_time: u64,
    block24_time: u64,
) -> f64 {
    let expected_duration = 144 * BLOCK_TIME_SECONDS; // Fixed 144-block window
    let actual_duration = current_block_time - block24_time;

    let adjustment_factor = expected_duration as f64 / actual_duration as f64;
    (adjustment_factor - 1.0) * 100.0 // Return percentage change
}


/// Returns a `tui` widget for the header of Dashboard.
pub fn render_header(percent: f64) -> Paragraph<'static> {
    let dot: &'static str = if percent == 0.0 {
        "●" // new epoch signature (full moon)
    } else if percent < 25.0 {
        "○"
    } else if percent < 50.0 {
        "◔"
    } else if percent < 75.0 {
        "◑"
    } else {
        "◕"
    };

    
    let cycle_dot = Span::styled(dot, Style::default().fg(Color::Yellow));

    // Combine the footer message and app version.
    let lines = vec![
        Spans::from(vec![
            Span::styled(
                r"₿lockChainInfo ",
                Style::default().fg(Color::Cyan),
            ),
            cycle_dot,
        ]),
        Spans::from(Span::styled(
            format!("v{}", APP_VERSION),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
    ];

    Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().title("").borders(Borders::NONE))
}

/// Returns a `tui` widget for the footer of Dashboard.
pub fn render_footer<B: Backend>(f: &mut Frame<B>, area: Rect, message: &str) {
    let footer_text = vec![Spans::from(vec![Span::styled(
        message,
        Style::default().fg(Color::Gray),
    )])];

    let footer = Paragraph::new(footer_text)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(footer, area);
}

/// Logs RPC errors to errors_log.txt file.
pub fn log_error(message: &str) -> io::Result<()> {
    let log_path = "error_log.txt";

    // Check if the file exists and is in the old format
    if let Ok(meta) = metadata(log_path) {
        if meta.len() > 0 {
            let mut file = OpenOptions::new().read(true).open(log_path)?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;

            // Check if the file contains the old format
            if buffer.contains("JsonParsingError(") {
                // Rotate the log file to archive
                let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                let rotated_log_path = format!("error_log_{}.txt", timestamp);
                rename(log_path, rotated_log_path)?;
            }
        }
    }

    // Auto-truncate if the file exceeds 500KB
    if let Ok(meta) = metadata(log_path) {
        if meta.len() > 500_000 {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
            let rotated_log_path = format!("error_log_{}.txt", timestamp);
            rename(log_path, rotated_log_path)?;
        }
    }

    // Open the log file in append mode (create it if it doesn't exist)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    // Write the log message with a timestamp
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let log_entry = format!("[{}] {}\n", timestamp, message);

    // Lock the global Mutex to ensure thread-safe writes
    let _lock = LOG_FILE.lock().unwrap();
    file.write_all(log_entry.as_bytes())?;

    Ok(())
}

/// Loads the miners.json file into [MinersData].
pub fn load_miners_data() -> Result<MinersData, MyError> {
    let file_path = "miners.json";
    let data = fs::read_to_string(file_path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            MyError::FileNotFound(format!("The file '{}' was not found.", file_path))
        } else {
            MyError::Io(e)
        }
    })?;
    let miners_data: MinersData = serde_json::from_str(&data)?;
    Ok(miners_data)
}

