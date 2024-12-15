
// display/display_blockchain_info.rs

use colored::*;
use num_format::{Locale, ToFormattedString};
use crate::utils::{format_size, estimate_difficulty_change};
use crate::models::blockchain_info::BlockchainInfo;
use crate::models::block_info::BlockInfo;
use crate::models::errors::MyError;

// Displays the blockchain information.
pub fn display_blockchain_info(blockchain_info: &BlockchainInfo, block_info: &BlockInfo) -> Result<(), MyError> {
    let mediantime = blockchain_info.parse_mediantime()?;
    let time = blockchain_info.parse_time()?;
    let formatted_size_on_disk = format_size(blockchain_info.size_on_disk);
    let time_since_block = blockchain_info.calculate_time_diff()?;
    let formatted_difficulty = blockchain_info.formatted_difficulty()?; 
    let formatted_chainwork_bits = blockchain_info.formatted_chainwork_bits()?;
    let estimate_difficulty_change = estimate_difficulty_change(blockchain_info.blocks,
        blockchain_info.time, block_info.time);

    println!();
    println!("{}", r" ____  __    _____   ___  _  _   ___  _   _    __    ____  _  _  ____  _  _  ____  _____ ".normal());
    println!("{}", r"(  _ \(  )  (  _  ) / __)( )/ ) / __)( )_( )  /__\  (_  _)( \( )(_  _)( \( )( ___)(  _  )".bright_white());
    println!("{}", r" ) _ < )(__  )(_)( ( (__  )  ( ( (__  ) _ (  /(__)\  _)(_  )  (  _)(_  )  (  )__)  )(_)(".cyan());
    println!("{}", r"(____/(____)(_____) \___)(_)\_) \___)(_) (_)(__)(__)(____)(_)\_)(____)(_)\_)(__)  (_____)".blue());
    println!();
    println!("{}", "[Blockchain]".bold().underline().cyan());
    println!("Chain: {}", blockchain_info.chain.yellow());
    println!("Best Block: {}", 
        blockchain_info.blocks.to_formatted_string(&Locale::en).green());
    println!("  Time since block: {}", time_since_block.red());
    println!("Difficulty: {}", formatted_difficulty.bright_red());
    blockchain_info.display_blocks_until_difficulty_adjustment()?;
    
    // Choose an arrow and color based on the value
    let arrow = if estimate_difficulty_change > 0.0 {
        "↑".green().bold() // Green up arrow in bold for positive values
    } else {
        "↓".red().bold() // Red down arrow in bold for negative values
    };

    println!("  Estimated change: {} {:.2}%", arrow, estimate_difficulty_change);
    println!("Chainwork: {}", formatted_chainwork_bits.bright_yellow());
    println!("Verification progress: {}%", format!("{:.4}", 
        blockchain_info.verificationprogress * 100.0).yellow());
    println!("Size on Disk: {}", formatted_size_on_disk);
    println!("Median Time: {}", mediantime);
    println!("Block Time: {}", time);

    if !blockchain_info.warnings.is_empty() {
        println!("{}: {}", "Warnings".bold().red(), blockchain_info.warnings);
    }

    println!();
    Ok(())
}