//
// utils.rs
//
use std::process::Command;
use crate::models::errors::MyError;  


// Constants for bytes formatting.
const KB: u64 = 1024;
const MB: u64 = KB * 1024;
const GB: u64 = MB * 1024;
const TB: u64 = GB * 1024;


// Formats a size in bytes into a more readable format (KB, MB, etc.).
pub fn format_size(bytes: u64) -> String {
    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}


// Retrieves the RPC password stored in macOS Keychain.
#[cfg(target_os = "macos")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    let output = Command::new("security")
        .arg("find-generic-password")
        .arg("-s")
        .arg("rpc-password")
        .arg("-a")
        .arg("bitcoin")
        .arg("-w")
        .output()
        .map_err(|e| MyError::Keychain(format!("Keychain access error: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr).to_string();
        Err(MyError::Keychain(format!("Password not found in keychain: {}", error_message)))
    }
}


// Linux-specific logic (placeholder, implement accordingly).
#[cfg(target_os = "linux")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Linux keyring access not supported".to_string()))
}


// Windows-specific logic (placeholder, implement accordingly).
#[cfg(target_os = "windows")]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Windows keychain access not supported".to_string()))
}


// Fallback for unsupported OS.
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn get_rpc_password_from_keychain() -> Result<String, MyError> {
    Err(MyError::Keychain("Unsupported OS for keychain access".to_string()))
}