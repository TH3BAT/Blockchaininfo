
// spinner.rs

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::task;
use tokio::time::{self, Duration};

pub static SPINNER_INDEX: Lazy<Arc<AtomicUsize>> = Lazy::new(|| Arc::new(AtomicUsize::new(0)));
const SPINNER_FRAMES: [&str; 4] = ["|", "/", "-", "\\"];

/// ✅ Background spinner thread that updates independently
pub fn start_spinner_thread(spinner_index: Arc<AtomicUsize>) {
    task::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(100)); // ✅ Faster updates
        loop {
            interval.tick().await;
            spinner_index.fetch_add(1, Ordering::Relaxed);
        }
    });
}

/// ✅ Returns the current spinner frame dynamically
pub fn get_spinner_frame() -> String {
    let index = SPINNER_INDEX.load(Ordering::Relaxed) % SPINNER_FRAMES.len();
    format!("{} Loading Mempool Data...", SPINNER_FRAMES[index])
}
