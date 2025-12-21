// src/ui/colors.rs
//
// Theme customization:
// You can freely change these colors to suit your terminal theme.
// BCI does not load themes dynamically by design — colors are compile-time
// constants to keep the UI predictable and lightweight.
//
// Suggested alternatives:
// - Monochrome: White / DarkGray
// - Solarized: Blue / Cyan / Yellow
// - High-contrast: White / Red / Green

use tui::style::Color;

/// Borders
pub const C_BLOCKCHAIN_BORDER: Color = Color::DarkGray;
pub const C_MEMPOOL_BORDER: Color = Color::DarkGray;
pub const C_NETWORK_BORDER: Color = Color::DarkGray;
pub const C_CONSENSUS_BORDER: Color = Color::DarkGray;

/// Core identity colors
pub const C_APP_TITLE: Color = Color::Cyan;
pub const C_APP_VERSION: Color = Color::DarkGray;
pub const C_MAIN_LABELS: Color = Color::Gray;
pub const C_SECTION_LABELS: Color = Color::DarkGray;
pub const C_HORIZONTAL_ASCII_BAR: Color = Color::Gray;
pub const C_SEPARATORS: Color = Color::DarkGray;
pub const C_KEYTOGGLE_HIGHLIGHT: Color = Color::Yellow;
pub const C_KEYTOGGLE_DIM: Color = Color::DarkGray;
pub const C_HASH_PHASE: Color = Color::Yellow;
pub const C_HASH_PHASE_NEW: Color = Color::DarkGray;
pub const C_FOOTER_DISPLAY: Color = Color::Gray;

// Used for Best Block, Transactions, Connections In
// *_FLASH colors are used for brief visual emphasis on state change.
pub const C_PREFLASH: Color = Color::Green;
pub const C_FLASH: Color = Color::White;

/// Blockchain section
pub const C_CHAIN: Color = Color::Yellow;

pub const C_MINER: Color = Color::Yellow;
// *_FLASH colors are used for brief visual emphasis on state change.
pub const C_MINER_FLASH: Color = Color::LightYellow;

pub const C_TIME_SINCE_BLOCK: Color = Color::Red;
pub const C_DIFFICULTY: Color = Color::LightRed;
pub const C_ESTIMATE_POS: Color = Color::Green;
pub const C_ESTIMATE_NEG: Color = Color::Red;
pub const C_CHAINWORK: Color = Color::LightYellow;
pub const C_VERIFICATION: Color = Color::Yellow;
pub const C_HASHRATE_CHART_BARS: Color = Color::DarkGray;
pub const C_HASHRATE_CHART_VALUES: Color = Color::White;

/// Mempool
pub const C_MEMPOOL_DIST_LABELS: Color = Color::Yellow;
pub const C_MEMPOOL_USAGE_GAUGE_FG: Color = Color::DarkGray;
pub const C_MEMPOOL_USAGE_GAUGE_BG: Color = Color::Black;
pub const C_MEMPOOL_VALUES: Color = Color::Gray;
pub const C_DUST_FREE_PCT: Color = Color::Gray;
pub const C_DUST_FREE_LABEL: Color = Color::DarkGray;


/// Network
pub const C_CONNECTIONS_OUT: Color = Color::Yellow;
pub const C_VERSION_CHART_BARS: Color = Color::DarkGray;
pub const C_VERSION_CHART_VALUES: Color = Color::White;
pub const C_CLIENT_DIST_MINER_LABEL: Color = Color::Cyan;
pub const C_SPARKLINE: Color = Color::DarkGray;

/// Consensus
pub const C_CONSENSUS_STATUS_SECTION: Color = Color::Yellow;

/// Status colors
pub const C_STATUS_LOW: Color = Color::Green;
pub const C_STATUS_MED: Color = Color::Yellow;
pub const C_STATUS_HIGH: Color = Color::Red;
