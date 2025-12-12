//! Shared utilities for BlockchainInfo.
//!
//! This module provides:
//! - Global caches (Arc<RwLock<...>>) for cross-module data sharing
//! - Formatting helpers (sizes, percentages)
//! - Difficulty adjustment estimators
//! - Dashboard header/footer render utilities
//! - Error logging with automatic rotation
//! - Keychain/RPC password retrieval (macOS only)
//! - File loading helpers (e.g., miners.json)
//!
//! All logic is intentionally lightweight—optimized for clarity,
//! thread-safety, and serving the TUI layer cleanly.

use crate::models::errors::MyError;
use tui::widgets::{Block, Borders, Paragraph};
use tui::text::{Span, Spans};
use tui::style::{Color, Style, Modifier};
use tui::layout::{Rect, Alignment};
use tui::Frame;
use tui::backend::Backend;
use tui::style::Color::{DarkGray, Yellow};

use std::fs::{OpenOptions, metadata, rename};
use std::fs;
use std::io::{self, Write};
use std::sync::Mutex;
use std::io::Read;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use lazy_static::lazy_static;
use chrono::Local;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::BlockInfo;
use crate::models::chaintips_info::ChainTipsResponse;
use crate::models::mempool_info::{MempoolDistribution, MempoolInfo};
use crate::models::peer_info::PeerInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::network_totals::NetTotals;
use crate::models::block_info::{BlockHistory, MinersData};

//
// ────────────────────────────────────────────────────────────────────────────────
//   SIZE CONSTANTS & FORMATTERS
// ────────────────────────────────────────────────────────────────────────────────
//

const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;
const TB: u64 = GB * 1024;

// For ASCII bar charts.
pub const BAR_ACTIVE: Color = Color::Gray;

/// Convert raw bytes into human-readable units.
///
/// Examples:
/// - `1536 → "1.50 KB"`
/// - `1048576 → "1.00 MB"`
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

//
// ────────────────────────────────────────────────────────────────────────────────
//   APP VERSION (from Cargo.toml)
// ────────────────────────────────────────────────────────────────────────────────
//

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

//
// ────────────────────────────────────────────────────────────────────────────────
//   GLOBAL CACHES (TUI-accessible)
// ────────────────────────────────────────────────────────────────────────────────
//
// These provide read/write access across all async tasks & drawing logic.
// Each cache is an Arc<RwLock<T>>.

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

// Tracks logged TXIDs to avoid duplication in logs.
// (500 item rolling window)
lazy_static! {
    pub static ref LOGGED_TXS: Lazy<Arc<RwLock<(HashSet<Arc<String>>, VecDeque<Arc<String>>)>>> =
        Lazy::new(|| Arc::new(RwLock::new((HashSet::with_capacity(500), VecDeque::with_capacity(500)))));
}

// Hash distribution history over (typically) the past 144 blocks.
lazy_static! {
    pub static ref BLOCK_HISTORY: Arc<RwLock<BlockHistory>> =
        Arc::new(RwLock::new(BlockHistory::new()));
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   KEYCHAIN / RPC PASSWORD RETRIEVAL
// ────────────────────────────────────────────────────────────────────────────────
//

#[cfg(target_os = "macos")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    use std::process::Command;

    let output = Command::new("security")
        .arg("find-generic-password")
        .arg("-s").arg("rpc-password")
        .arg("-a").arg("bitcoin")
        .arg("-w")
        .output()
        .map_err(|e| MyError::Keychain(format!("Keychain retrieval failed: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(MyError::Keychain(
            format!("Password not found: {}", String::from_utf8_lossy(&output.stderr)),
        ))
    }
}

#[cfg(target_os = "linux")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Linux keyring access not supported".into()))
}

#[cfg(target_os = "windows")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Windows keychain access not supported".into()))
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Unsupported OS for keychain access".into()))
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   DIFFICULTY ADJUSTMENT ESTIMATORS
// ────────────────────────────────────────────────────────────────────────────────
//

pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2016;
pub const BLOCK_TIME_SECONDS: u64 = 600;

/// Estimate % difficulty change for the *current epoch*.
pub fn estimate_difficulty_change(
    current_block_height: u64,
    current_block_time: u64,
    epoch_start_block_time: u64,
) -> f64 {
    let blocks_in_epoch = (current_block_height % DIFFICULTY_ADJUSTMENT_INTERVAL) - 1;
    let expected = blocks_in_epoch * BLOCK_TIME_SECONDS;
    let actual = current_block_time.saturating_sub(epoch_start_block_time);

    let factor = expected as f64 / actual as f64;
    (factor - 1.0) * 100.0
}

/// Estimate % difficulty change over the past 24 hours (144 blocks).
pub fn estimate_24h_difficulty_change(
    current_block_time: u64,
    block24_time: u64,
) -> f64 {
    let expected = 144 * BLOCK_TIME_SECONDS;
    let actual = current_block_time.saturating_sub(block24_time);

    let factor = expected as f64 / actual as f64;
    (factor - 1.0) * 100.0
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   DASHBOARD HEADER & FOOTER
// ────────────────────────────────────────────────────────────────────────────────
//

/// Render the header block, including the epoch-cycle dot and version.
pub fn render_header(percent: f64) -> Paragraph<'static> {
    // Phase glyph (visual epoch indicator)
    let dot = if percent == 0.0 {
        "●" // New epoch (solid circle)
    } else if percent < 25.0 {
        "○"
    } else if percent < 50.0 {
        "◔"
    } else if percent < 75.0 {
        "◑"
    } else {
        "◕"
    };

    // We want the first phase change to be at 10%, and the percent is passed already converted.
    let color = if percent < 10.0 { DarkGray } else { Yellow };

    Paragraph::new(vec![
        Spans::from(vec![
            Span::styled("₿lockChainInfo ", Style::default().fg(Color::Cyan)),
            Span::styled(dot, Style::default().fg(color)),
        ]),
        Spans::from(Span::styled(
            format!("v{}", APP_VERSION),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE))
}

/// Render footer message centered across the dashboard.
pub fn render_footer<B: Backend>(f: &mut Frame<B>, area: Rect, message: &str) {
    let footer = Paragraph::new(vec![Spans::from(Span::styled(
        message,
        Style::default().fg(Color::Gray),
    ))])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    f.render_widget(footer, area);
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   ERROR LOG ROTATION & APPEND
// ────────────────────────────────────────────────────────────────────────────────
//

lazy_static! {
    /// Global mutex ensuring only one writer accesses the log at a time.
    static ref LOG_FILE: Mutex<()> = Mutex::new(());
}

/// Append an error message to `error_log.txt`, with auto-rotation:
/// - Rotates if legacy format detected
/// - Rotates if file exceeds 500 KB
pub fn log_error(message: &str) -> io::Result<()> {
    let log_path = "error_log.txt";

    // Rotate if old-format log detected
    if let Ok(meta) = metadata(log_path) {
        if meta.len() > 0 {
            let mut contents = String::new();
            OpenOptions::new().read(true).open(log_path)?.read_to_string(&mut contents)?;

            if contents.contains("JsonParsingError(") {
                let ts = Local::now().format("%Y%m%d_%H%M%S");
                rename(log_path, format!("error_log_{}.txt", ts))?;
            }
        }
    }

    // Rotate if oversized
    if let Ok(meta) = metadata(log_path) {
        if meta.len() > 500_000 {
            let ts = Local::now().format("%Y%m%d_%H%M%S");
            rename(log_path, format!("error_log_{}.txt", ts))?;
        }
    }

    // Write entry
    let mut file = OpenOptions::new().create(true).append(true).open(log_path)?;
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
    let entry = format!("[{}] {}\n", ts, message);

    let _lock = LOG_FILE.lock().unwrap();
    file.write_all(entry.as_bytes())
}

/// Load miners.json into a parsed MinersData struct.
pub fn load_miners_data() -> Result<MinersData, MyError> {
    let path = "miners.json";
    let data = fs::read_to_string(path).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            MyError::FileNotFound(format!("'{}' not found.", path))
        } else {
            MyError::Io(e)
        }
    })?;

    Ok(serde_json::from_str(&data)?)
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   PERCENT NORMALIZATION (for charts)
// ────────────────────────────────────────────────────────────────────────────────
//

/// Normalize a list of counts into percentages summing to exactly 100.
/// Used for pie/bar chart displays.
///
/// Uses "largest remainder" method:
/// 1. Compute raw floating percentages
/// 2. Floor them
/// 3. Track remainders
/// 4. Distribute leftover percentage points to largest remainders
pub fn normalize_percentages(counts: &[u64]) -> Vec<u64> {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return vec![0; counts.len()];
    }

    let raw: Vec<f64> = counts
        .iter()
        .map(|c| (*c as f64 / total as f64) * 100.0)
        .collect();

    let mut floored: Vec<u64> = raw.iter().map(|v| v.floor() as u64).collect();

    let mut remainders: Vec<(f64, usize)> = raw
        .iter()
        .enumerate()
        .map(|(i, v)| (v - v.floor(), i))
        .collect();

    remainders.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let mut sum: u64 = floored.iter().sum();

    for &(_, idx) in remainders.iter() {
        if sum >= 100 {
            break;
        }
        floored[idx] += 1;
        sum += 1;
    }

    floored
}