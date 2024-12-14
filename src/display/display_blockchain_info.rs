
// display/display_blockchain_info.rs

use colored::*;
use num_format::{Locale, ToFormattedString};
use crate::utils::format_size;
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::errors::MyError;

// Displays the blockchain information.
pub fn display_blockchain_info(blockchain_info: &BlockchainInfo) -> Result<(), MyError> {
    let mediantime = blockchain_info.parse_mediantime()?;
    let time = blockchain_info.parse_time()?;
    let formatted_size_on_disk = format_size(blockchain_info.size_on_disk);
    let time_since_block = blockchain_info.calculate_time_diff()?;
    let formatted_difficulty = blockchain_info.formatted_difficulty()?; 
    let formatted_chainwork_bits = blockchain_info.formatted_chainwork_bits()?;

    println!();
    println!("{}", "[Blockchain]".bold().underline().cyan());
    println!("Chain: {}", blockchain_info.chain.yellow());
    println!("Number of Blocks: {}", 
        blockchain_info.blocks.to_formatted_string(&Locale::en).green());
    println!("Chainwork: {}", formatted_chainwork_bits.bright_yellow());
    println!("Difficulty: {}", formatted_difficulty.bright_red());
    println!("Verification progress: {}%", format!("{:.4}", 
        blockchain_info.verificationprogress * 100.0).yellow());
    println!("Size on Disk: {}", formatted_size_on_disk);
    println!("Median Time: {}", mediantime);
    println!("Block Time: {}", time);
    println!("Time since block: {}", time_since_block.red());
    blockchain_info.display_blocks_until_difficulty_adjustment()?;

    if !blockchain_info.warnings.is_empty() {
        println!("{}: {}", "Warnings".bold().red(), blockchain_info.warnings);
    }

    println!();
    Ok(())
}