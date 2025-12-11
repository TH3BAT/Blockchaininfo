//! Data models for Bitcoin Core’s `getrawtransaction` RPC.
//!
//! This module provides full transaction parsing for both confirmed and
//! unconfirmed transactions, including:
//! - inputs (vin)
//! - outputs (vout)
//! - scriptSig / scriptPubKey structures
//! - OP_RETURN message decoding
//! - helper methods for value aggregation & spendability
//!
//! All models intentionally mirror Core’s RPC format exactly. Interpretation
//!––such as OP_RETURN decoding or spendability checks––happens in helpers.

use serde::Deserialize;
use std::str;

//
// ────────────────────────────────────────────────────────────────────────────────
//   RPC WRAPPER & MAIN TX STRUCT
// ────────────────────────────────────────────────────────────────────────────────
//

/// Mirror of `getrawtransaction` (verbose=1 or verbose=2 depending on call).
///
/// A transaction may be:
/// - *confirmed* → includes `blockhash`, `confirmations`, `blocktime`  
/// - *unconfirmed* → includes only `time`
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct GetRawTransactionResponse {
    pub hex: String,
    pub txid: String,
    pub hash: String,
    pub size: u32,
    pub vsize: u32,
    pub weight: u32,
    pub version: u32,
    pub locktime: u32,

    /// Transaction inputs.
    pub vin: Vec<TxIn>,

    /// Transaction outputs.
    pub vout: Vec<TxOut>,

    /// Block metadata if confirmed.
    pub blockhash: Option<String>,
    pub confirmations: Option<u32>,
    pub blocktime: Option<u64>,

    /// Timestamp for unconfirmed transactions.
    pub time: Option<u64>,
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   TX-LEVEL HELPERS
// ────────────────────────────────────────────────────────────────────────────────
//

impl GetRawTransactionResponse {
    /// True if the transaction is confirmed and has a blockhash.
     #[allow(dead_code)]
    pub fn is_confirmed(&self) -> bool {
        self.blockhash.is_some()
    }

    /// Sum of all output values (in BTC).
    pub fn total_output_value(&self) -> f64 {
        self.vout.iter().map(|v| v.value).sum()
    }

    /// True if any output contains an OP_RETURN script.
    pub fn has_op_return(&self) -> bool {
        self.vout.iter().any(|out| out.is_op_return())
    }

    /// Total value of all OP_RETURN outputs (usually zero).
    pub fn total_op_return_value(&self) -> f64 {
        self.vout
            .iter()
            .filter(|out| out.is_op_return())
            .map(|out| out.value)
            .sum()
    }

    /// Returns all OP_RETURN messages decoded as UTF-8 strings.
     #[allow(dead_code)]
    pub fn get_op_return_msg(&self) -> Vec<String> {
        self.vout
            .iter()
            .filter_map(|out| out.decipher_op_return())
            .collect()
    }
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   INPUT STRUCTURES
// ────────────────────────────────────────────────────────────────────────────────
//

/// A transaction input.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct TxIn {
    /// The txid of the spent output.
    pub txid: String,

    /// Output index being spent.
    pub vout: u32,

    /// ScriptSig for legacy inputs.
    #[serde(rename = "scriptSig")]
    pub script_sig: Option<ScriptSig>,

    /// Sequence number.
    pub sequence: u32,

    /// Optional witness stack for segwit inputs.
    pub txinwitness: Option<Vec<String>>,
}

/// ScriptSig for legacy inputs.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ScriptSig {
    pub asm: String,
    pub hex: String,
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   OUTPUT STRUCTURES
// ────────────────────────────────────────────────────────────────────────────────
//

/// A transaction output.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct TxOut {
    /// Value in BTC.
    pub value: f64,

    /// Output index.
    pub n: u32,

    /// ScriptPubKey structure.
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: Option<ScriptPubKey>,
}

/// ScriptPubKey metadata.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct ScriptPubKey {
    pub asm: Option<String>,
    pub hex: Option<String>,
    #[serde(rename = "reqSigs")]
    pub req_sigs: Option<u32>,
    pub r#type: Option<String>,
    pub addresses: Option<Vec<String>>,
}

//
// ────────────────────────────────────────────────────────────────────────────────
//   OUTPUT HELPERS
// ────────────────────────────────────────────────────────────────────────────────
//

impl TxOut {
    /// Whether this output pays to a given Bitcoin address.
     #[allow(dead_code)]
    pub fn is_spendable_by(&self, address: &str) -> bool {
        if let Some(script) = &self.script_pub_key {
            if let Some(addrs) = &script.addresses {
                return addrs.contains(&address.to_string());
            }
        }
        false
    }

    /// True if the script begins with `OP_RETURN`.
    pub fn is_op_return(&self) -> bool {
        self.script_pub_key
            .as_ref()
            .and_then(|spk| spk.asm.as_ref())
            .map(|asm| asm.starts_with("OP_RETURN"))
            .unwrap_or(false)
    }

    /// Attempt to decode the OP_RETURN payload as UTF-8.
    ///
    /// Returns:
    /// - `Some(String)` on success
    /// - `None` if the script is not OP_RETURN or cannot be decoded
     #[allow(dead_code)]
    pub fn decipher_op_return(&self) -> Option<String> {
        let script = self.script_pub_key.as_ref()?;
        let asm = script.asm.as_ref()?;

        if !asm.starts_with("OP_RETURN") {
            return None;
        }

        // OP_RETURN <hex>
        let parts: Vec<&str> = asm.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let hex_data = parts[1];

        // Decode hex → bytes → utf8
        let bytes = hex::decode(hex_data).ok()?;
        let msg = str::from_utf8(&bytes).ok()?.to_string();

        Some(msg)
    }
}
