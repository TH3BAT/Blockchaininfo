//
// display.rs
//
/// Summary of Color Usage:
/// Green: Used for healthy values like block count, transactions, and connections.
///
/// Yellow: Used for values that are important but not urgent, like chain info, verification progress, 
/// and min relay transaction fees.
///
/// Bright Yellow: For metrics like chainwork that represent cumulative effort.
///
/// Bright Red: Used for metrics like difficulty to make them stand out, signaling critical importance.
///
/// Red: Used for critical time-related fields like "Time since block" to signal urgency.
///
/// White: Used for neutral data, like timestamps, sizes, and fees, where urgency is not a factor.
///
/// This approach ensures that critical, urgent, and less critical information is presented in a way 
/// that emphasizes what’s most important at a glance.
/// 
use colored::*;
use num_format::{Locale, ToFormattedString};
use crate::utils::format_size;
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::mempool_info::MempoolInfo;
use crate::models::network_info::NetworkInfo;
use crate::models::errors::MyError;

// Displays the blockchain information.
pub fn display_blockchain_info(blockchain_info: &BlockchainInfo) -> Result<(), MyError> {
    let mediantime = blockchain_info.result.parse_mediantime()?;
    let time = blockchain_info.result.parse_time()?;
    let formatted_size_on_disk = format_size(blockchain_info.result.size_on_disk);
    let time_since_block = blockchain_info.result.calculate_time_diff()?;
    let formatted_difficulty = blockchain_info.result.formatted_difficulty()?; 
    let formatted_chainwork_bits = blockchain_info.result.formatted_chainwork_bits()?;

    println!();
    println!("{}", "[Blockchain]".bold().underline().cyan());
    println!("Best Block Hash: {}", blockchain_info.result.bestblockhash.white());
    println!("Number of Blocks: {}", 
        blockchain_info.result.blocks.to_formatted_string(&Locale::en).green());
    println!("Chain: {}", blockchain_info.result.chain.yellow());
    println!("Chainwork: {}", formatted_chainwork_bits.bright_yellow());
    println!("Difficulty: {}", formatted_difficulty.bright_red());
    println!("Verification progress: {}%", format!("{:.4}", 
        blockchain_info.result.verificationprogress * 100.0).yellow());
    println!("Size on Disk: {}", formatted_size_on_disk.white());
    println!("Median Time: {}", mediantime.white());
    println!("Block Time: {}", time.white());
    println!("Time since block: {}", time_since_block.red());

    if !blockchain_info.result.warnings.is_empty() {
        println!("{}: {}", "Warnings".bold().red(), blockchain_info.result.warnings);
    }

    println!();
    Ok(())
}

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

// Displays the network information.
pub fn display_network_info(network_info: &NetworkInfo) -> Result<(), MyError> {
    println!("{}", "[Network]".bold().underline().cyan());
    println!("Connections in: {}", network_info.connections_in.to_string().green());
    println!("Connections out: {}", network_info.connections_out.to_string().yellow());
    println!();

    Ok(())
}
