
// models/flashing_text.rs

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tui::style::{Style, Color};

// Global initialization
lazy_static! {
    pub static ref BEST_BLOCK_TEXT: Mutex<FlashingText> = Mutex::new(FlashingText::new());
}

pub struct FlashingText {
    pub last_value: u64,          // The last value that was displayed
    pub flash_until: Option<Instant>, // When the flashing effect should end
}

impl FlashingText {
    pub fn new() -> Self {
        Self {
            last_value: 0,
            flash_until: None,
        }
    }

    pub fn update(&mut self, new_value: u64) {
        if new_value != self.last_value {
            self.last_value = new_value;
            self.flash_until = Some(Instant::now() + Duration::from_millis(500)); // Flash for 200ms
        }
    }

    pub fn style(&self) -> Style {
        if let Some(flash_until) = self.flash_until {
            if Instant::now() < flash_until {
                return Style::default().fg(Color::White); //Highlight style
            }
        }
        Style::default().fg(Color::Green) // Default style
    }
}

