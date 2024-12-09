//
// display/display_mempool_info.rs
//
use colored::*;
use num_format::{Locale, ToFormattedString};
use crate::utils::format_size;
use crate::models::mempool_info::MempoolInfo;
use crate::models::errors::MyError;

// Displays the mempool information.
pub fn display_mempool_info(mempool_info: &MempoolInfo) -> Result<(), MyError> {
    let mempool_size_in_memory = format_size(mempool_info.usage);
    let max_mempool_size_in_memory = format_size(mempool_info.maxmempool);

    let mempool_size_in_memory_color = if mempool_info.usage < mempool_info.maxmempool / 3 {
        mempool_size_in_memory.green()
    } else if mempool_info.usage < 2 * mempool_info.maxmempool / 3 {
        mempool_size_in_memory.yellow()
    } else {
        mempool_size_in_memory.red()
    };

    let min_relay_fee_vsats = mempool_info.min_relay_tx_fee_vsats();

    println!("{}", "[Mempool]".bold().underline().cyan());
    println!("Transactions: {}", mempool_info.size.to_formatted_string(&Locale::en).green());
    println!("Memory: {} / {}", mempool_size_in_memory_color, max_mempool_size_in_memory.white());
    println!("Total fees: {}", mempool_info.total_fee.to_string().white());
    println!("Min Transaction Fee: {} vSats/vByte", 
        min_relay_fee_vsats.to_formatted_string(&Locale::en).yellow());
    println!();

    Ok(())
}