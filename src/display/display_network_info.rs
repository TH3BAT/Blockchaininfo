//
// display/display_network_info.rs
//
use colored::*;
use crate::models::network_info::NetworkInfo;
use crate::models::errors::MyError;

// Displays the network information.
pub fn display_network_info(network_info: &NetworkInfo) -> Result<(), MyError> {
    println!("{}", "[Network]".bold().underline().cyan());
    println!("Connections in: {}", network_info.connections_in.to_string().green());
    println!("Connections out: {}", network_info.connections_out.to_string().yellow());
    println!();

    Ok(())
}
