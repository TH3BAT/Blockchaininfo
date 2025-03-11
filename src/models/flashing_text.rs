
// models/flashing_text.rs

use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tui::style::{Style, Color};

// Global initialization
lazy_static! {
    pub static ref BEST_BLOCK_TEXT: Mutex<FlashingText> = Mutex::new(FlashingText::new());
}

lazy_static! {
    pub static ref TRANSACTION_TEXT: Mutex<FlashingText> = Mutex::new(FlashingText::new());
}


lazy_static! {
    pub static ref CONNECTIONS_IN_TEXT: Mutex<FlashingText> = Mutex::new(FlashingText::new());
}

lazy_static! {
    pub static ref MINER_TEXT: Mutex<FlashingMiner> = Mutex::new(FlashingMiner::new());
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
            self.flash_until = Some(Instant::now() + Duration::from_millis(200)); 
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


pub struct FlashingMiner {
    pub last_value: String,          // The last value that was displayed
    pub flash_until: Option<Instant>, // When the flashing effect should end
}

impl FlashingMiner {
    pub fn new() -> Self {
        Self {
            last_value: " ".to_string(),
            flash_until: None,
        }
    }

    pub fn update(&mut self, new_value: String) {
        if new_value != self.last_value {
            self.last_value = new_value;
            self.flash_until = Some(Instant::now() + Duration::from_millis(400)); 
        }
    }

    pub fn style(&self) -> Style {
        if let Some(flash_until) = self.flash_until {
            if Instant::now() < flash_until {
                return Style::default().fg(Color::LightYellow); //Highlight style
            }
        }
        Style::default().fg(Color::Yellow) // Default style
    }
}
