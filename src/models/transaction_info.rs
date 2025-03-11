
// models/transactions_info.rs

use serde::Deserialize;
use std::str;

/// This struct holds data from getrawtransaction RPC method.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct GetRawTransactionResponse {
    pub hex: String,                // The raw transaction in hexadecimal format.
    pub txid: String,               // The transaction ID (hash) in hexadecimal format.
    pub hash: String,               // The hash of the block containing this transaction.
    pub size: u32,                  // The size of the transaction in bytes.
    pub vsize: u32,                 // The virtual transaction size (used for fee calculation).
    pub weight: u32,                // The weight of the transaction (used for fee calculation).
    pub version: u32,               // The version number of the transaction format.
    pub locktime: u32,              // The lock time of the transaction (block height or timestamp).
    pub vin: Vec<TxIn>,             // A list of inputs (vin) in the transaction.
    pub vout: Vec<TxOut>,           // A list of outputs (vout) in the transaction.
    pub blockhash: Option<String>,  // The block hash containing this transaction (if confirmed).
    pub confirmations: Option<u32>, // The number of confirmations for this transaction.
    pub blocktime: Option<u64>,     // The block time in Unix epoch time (if confirmed).
    pub time: Option<u64>,          // The transaction time in Unix epoch time (if unconfirmed).
}

/// This struct holds TxIn data from GetRawTransactionResponse struct.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct TxIn {
    pub txid: String,                     // The transaction ID of the output being spent.
    pub vout: u32,                        // The index of the output being spent.
    #[serde(rename = "scriptSig")]
    pub script_sig: Option<ScriptSig>,    // The scriptSig for this input.
    pub sequence: u32,                    // The sequence number for this input.
    pub txinwitness: Option<Vec<String>>, // Witness data (if any) for this input.
}

/// This struct holds ScriptSig data from GetRawTransactionResponse->TxIn struct.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ScriptSig {
    pub asm: String, // The assembly representation of the script.
    pub hex: String, // The hexadecimal representation of the script.
}

/// This struct holds TxOut data from GetRawTransactionResponse struct.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct TxOut {
    pub value: f64,                   // The value of the output in BTC.
    pub n: u32,                       // The index of this output in the transaction.
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Option<ScriptPubKey>, // The scriptPubKey for this output.
}

/// This struct holds ScriptPubKey data from GetRawTransactionResponse->TxOut struct.
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ScriptPubKey {
    pub asm: Option<String>,             // The assembly representation of the script.
    pub hex: Option<String>,             // The hexadecimal representation of the script.
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<u32>,           // The number of signatures required to spend this output.
    pub r#type: Option<String>,          // The type of script (e.g., "pubkeyhash").
    pub addresses: Option<Vec<String>>,  // The addresses associated with this output (if any).
}

impl GetRawTransactionResponse {
    /// Returns whether the transaction is confirmed (has a blockhash) from getrawtransaction method.
    #[allow(dead_code)]
    pub fn is_confirmed(&self) -> bool {
        self.blockhash.is_some()
    }
    
    /// Returns the total value of all outputs in the transaction from getrawtransaction method.
    pub fn total_output_value(&self) -> f64 {
        self.vout.iter().map(|vout| vout.value).sum()
    }

     
    /// Returns true or false if the transaction contains any OP_RETURN outputs from getrawtransaction method.
    pub fn has_op_return(&self) -> bool {
        self.vout.iter().any(|output| output.is_op_return())
    }

    /// Returns the total value of all OP_RETURN outputs in the transaction from getrawtransaction method.
    pub fn total_op_return_value(&self) -> f64 {
        self.vout
            .iter()
            .filter(|output| output.is_op_return())
            .map(|output| output.value)
            .sum()
    }

    #[allow(dead_code)]
    /// Returns deciphered OP_RETURN message if any for TxOut from getrawtransaction method.
    pub fn get_op_return_msg(&self) -> Vec<String> {
        self.vout
            .iter()
            .filter_map(|vout| vout.decipher_op_return())
            .collect()
    }
    
}

 impl TxOut {
    /// Returns whether this output is spendable by the given address from getrawtransaction method.
    #[allow(dead_code)]
    pub fn is_spendable_by(&self, address: &str) -> bool {
        if let Some(addresses) = &<std::option::Option<ScriptPubKey> as Clone>::clone(&self.script_pub_key).unwrap().addresses {
            addresses.contains(&address.to_string())
        } else {
            false
        }
    }

    /// Returns whether this output is an OP_RETURN output from getrawtransaction method.    
    pub fn is_op_return(&self) -> bool {
        if let Some(script_pub_key) = &self.script_pub_key {
            if let Some(asm) = &script_pub_key.asm {
                // Check if the ASM script starts with "OP_RETURN"
                let is_op_return = asm.starts_with("OP_RETURN");

                is_op_return
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Deciphers OP_RETURN message
    #[allow(dead_code)]
    fn decipher_op_return(&self) -> Option<String> {
        if let Some(script_pub_key) = &self.script_pub_key {
            if let Some(asm) = &script_pub_key.asm {
                if asm.starts_with("OP_RETURN") {
                    // Split the ASM script into parts
                    let parts: Vec<&str> = asm.split_whitespace().collect();
                    if parts.len() > 1 {
                        // The data is the second part (hexadecimal string)
                        let hex_data = parts[1];
                        // Convert the hex string to bytes (Vec<u8>)
                        if let Ok(bytes) = hex::decode(hex_data) {
                            // Convert the bytes to a UTF-8 string (if possible)
                            return str::from_utf8(&bytes).map(|s| s.to_string()).ok();
                        }
                    }
                }
            }
        }
        None
    } 
}
