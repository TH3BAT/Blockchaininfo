//! Provides lightweight visual "flash" indicators for values that change on the dashboard.
//!
//! These indicators are used in the TUI to briefly highlight updated fields
//! (e.g., Best Block, Mempool Transactions, Incoming Connections, Last Miner).
//!
//! Each flash object stores:
//! - the last displayed value
//! - an expiration timestamp (`flash_until`) that determines how long the value flashes
//!
//! The flashing duration is intentionally short (200–400ms) to mimic a natural signal pulse,
//! not a long animation. The dashboard simply re-renders using the `.style()` method.

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tui::style::{Style, Color};

// Global flash tracker for the Best Block height.
// Updated whenever a new block is detected.
// Provides a quick white flash in the TUI to signal a chain tip update.
lazy_static! {
    pub static ref BEST_BLOCK_TEXT: Mutex<FlashingText> = Mutex::new(FlashingText::new());
}

// Global flash tracker for Mempool Transaction Count.
// Used to visually highlight when the mempool size changes.
lazy_static! {
    pub static ref TRANSACTION_TEXT: Mutex<FlashingText> = Mutex::new(FlashingText::new());
}

// Global flash tracker for **incoming peer connections**.
// Useful for spotting sudden spikes or drops in inbound traffic.
lazy_static! {
    pub static ref CONNECTIONS_IN_TEXT: Mutex<FlashingText> = Mutex::new(FlashingText::new());
}

// Global flash tracker for the **latest miner**.
// This uses a slightly longer flash duration (400ms) so miner changes stand out more.
lazy_static! {
    pub static ref MINER_TEXT: Mutex<FlashingMiner> = Mutex::new(FlashingMiner::new());
}

/// Tracks flashing behavior for numeric dashboard values (u64).
///
/// - `last_value` stores the previously rendered value
/// - When `update()` detects a change, it sets `flash_until`
/// - `style()` determines whether the value should appear highlighted
pub struct FlashingText {
    pub last_value: u64,                 // Previously displayed value
    pub flash_until: Option<Instant>,    // When the flash highlight should expire
}

impl FlashingText {
    /// Creates a new, non-flashing instance with no prior value.
    pub fn new() -> Self {
        Self {
            last_value: 0,
            flash_until: None,
        }
    }

    /// Updates the stored value and triggers a short flash if the value changed.
    ///
    /// Flash duration: **200 milliseconds**
    pub fn update(&mut self, new_value: u64) {
        if new_value != self.last_value {
            self.last_value = new_value;
            self.flash_until = Some(Instant::now() + Duration::from_millis(200));
        }
    }

    /// Determines the appropriate `tui` style based on whether the flash is active.
    ///
    /// - Active flash → **White**
    /// - Idle → **Green**
    pub fn style(&self) -> Style {
        if let Some(flash_until) = self.flash_until {
            if Instant::now() < flash_until {
                return Style::default().fg(Color::White); // Highlight style
            }
        }
        Style::default().fg(Color::Green) // Default style
    }
}

/// Flashing behavior for miner names (String-based values).
///
/// This differs from `FlashingText` only by value type and flash length.
pub struct FlashingMiner {
    pub last_value: String,              // Previously displayed miner name
    pub flash_until: Option<Instant>,    // When the flash highlight should expire
}

impl FlashingMiner {
    /// Creates a new miner flash tracker with a blank initial value.
    pub fn new() -> Self {
        Self {
            last_value: " ".to_string(),
            flash_until: None,
        }
    }

    /// Updates the miner name and triggers a slightly longer flash.
    ///
    /// Flash duration: **400 milliseconds**  
    /// (Miner changes are less frequent, so longer highlight is useful.)
    pub fn update(&mut self, new_value: String) {
        if new_value != self.last_value {
            self.last_value = new_value;
            self.flash_until = Some(Instant::now() + Duration::from_millis(400));
        }
    }

    /// Determines the style for miner text:
    ///
    /// - Active flash → **LightYellow**
    /// - Idle → **Yellow**
    pub fn style(&self) -> Style {
        if let Some(flash_until) = self.flash_until {
            if Instant::now() < flash_until {
                return Style::default().fg(Color::LightYellow); // Highlight
            }
        }
        Style::default().fg(Color::Yellow) // Default
    }
}
