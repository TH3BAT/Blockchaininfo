//! Data models for `getblockhash` and `getblock` (verbose=1 and verbose=2).
//!
//! These types mirror Bitcoin Core’s block-level RPC responses. They are
//! intentionally kept close to Core’s JSON schema so upstream changes
//! (new fields, renamed fields, removed fields) are easy to detect.
//!
//! Verbose level 1 (`verbose = 1`) returns only high-level metadata,
//! including the list of TXIDs.  
//! Verbose level 2 (`verbose = 2`) expands each transaction into a fully
//! decoded structure containing VIN, VOUT, scripts, witnesses, and sizes.
//!
//! Additionally, this module provides:
//! - models for miner-tagged metadata,
//! - support types for 24h block-history tracking,
//! - helpers for extracting addresses from verbose transactions.

use serde::Deserialize;
use std::collections::{VecDeque, HashMap};
use std::sync::{Mutex, Arc};
use crate::utils::{hex_decode, extract_ascii_runs};
use crate::consensus::satoshi_math::*;

/// Wrapper for `getblockhash`.  
/// Bitcoin Core returns `{ result: "blockhash", id, error }`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockHash {
    pub error: Option<String>,
    pub id: Option<String>,
    pub result: String, // The block hash in hex form
}

/// Wrapper for verbose=1 block data.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfoJsonWrap {
    pub result: BlockInfo,
}

/// Block metadata returned by `getblock` (verbose=1).
///
/// Contains no full transaction information — only TXIDs.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfo {
    pub hash: String,
    pub confirmations: u64,
    pub height: u64,
    pub version: u32,
    #[serde(rename = "versionHex")]
    pub version_hex: String,
    pub merkleroot: String,
    pub time: u64,
    pub mediantime: u64,
    pub nonce: u64,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    #[serde(rename = "nTx")]
    pub n_tx: u32,
    #[serde(skip)]
    #[allow(dead_code)]
    pub previousblockhash: Option<String>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub nextblockhash: Option<String>,
    pub strippedsize: u64,
    pub size: u64,
    pub weight: u64,
    pub tx: Vec<String>, // Only TXIDs at this verbosity level
}

/// Wrapper for verbose=2 block data.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfoFullJsonWrap {
    pub result: BlockInfoFull,
}

/// Full block information including decoded transactions.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct BlockInfoFull {
    pub hash: String,
    pub confirmations: u64,
    pub height: u64,
    pub version: u32,
    #[serde(rename = "versionHex")]
    pub version_hex: String,
    pub merkleroot: String,
    pub time: u64,
    pub mediantime: u64,
    pub nonce: u64,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    #[serde(rename = "nTx")]
    pub n_tx: u32,
    #[serde(skip)]
    #[allow(dead_code)]
    pub previousblockhash: Option<String>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub nextblockhash: Option<String>,
    pub strippedsize: u64,
    pub size: u32, // Bitcoin Core returns size as u32 in verbose=2, so we mirror this
    pub weight: u32,

    /// Full transaction objects for verbose=2
    #[serde(default)]
    pub tx: Vec<Transaction>,
}

/// Full Bitcoin transaction returned in verbose block mode.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Transaction {
    pub txid: String,
    pub hash: String,
    pub version: u32,
    pub size: u32,
    pub vsize: u32,
    pub weight: u32,
    pub locktime: u64,
    pub vin: Vec<TxIn>,
    pub vout: Vec<TxOut>,
}

impl Transaction {
    /// Extracts recipient wallet addresses from the transaction outputs.
    ///
    /// Bitcoin Core may omit addresses for nonstandard scripts; this method
    /// filters out empty address fields gracefully.
    pub fn extract_wallet_addresses(&self) -> Vec<String> {
        self.vout
            .iter()
            .filter(|o| !o.script_pub_key.address.is_empty())
            .map(|o| o.script_pub_key.address.clone())
            .collect()
    }

    /// Returns coinbase scriptSig bytes (decoded from hex) if this TX is a coinbase TX.
    /// Bitcoin Core provides `vin[0].coinbase` as hex string for coinbase transactions.
    fn extract_coinbase_bytes(&self) -> Option<Vec<u8>> {
        let vin0 = self.vin.get(0)?;
        let hex = vin0.coinbase.as_ref()?;
        hex_decode(hex).ok()
    }

    /// Returns coinbase scriptSig hex string, if present.
    #[allow(dead_code)]
    fn extract_coinbase_hex(&self) -> Option<&str> {
        self.vin.get(0)?.coinbase.as_deref()
    }

    /// Extracts printable ASCII runs from the coinbase scriptSig bytes.
    /// This is useful for miner/pool attribution when payout address lookup fails.
    pub fn extract_coinbase_ascii_runs(&self, min_len: usize) -> Vec<String> {
        let Some(bytes) = self.extract_coinbase_bytes() else {
            return Vec::new();
        };
        extract_ascii_runs(&bytes, min_len)
    }

    /// OCEAN-only: Extracts “candidate tags” from coinbase bytes.
    /// Unlike extract_coinbase_ascii_runs(), this can merge runs across small control gaps
    /// *except* when the gap contains NUL (0x00), which we treat as a hard separator.
    pub fn extract_coinbase_ocean_candidates(&self, max_gap: usize) -> Vec<String> {
        let Some(bytes) = self.extract_coinbase_bytes() else {
            return Vec::new();
        };
        Self::extract_ocean_candidates(&bytes, max_gap)
    }

    /// Produce “human-ish” candidates for OCEAN secondaries.
    /// This will yield BDEHX, will NOT yield SoVAV (due to NUL break).
    fn extract_ocean_candidates(bytes: &[u8], max_gap: usize) -> Vec<String> {
        let runs = Self::extract_runs_idx(bytes);
        let merged = Self::merge_runs_no_nul(bytes, &runs, max_gap);

        let mut out = Vec::new();
        for span in merged {
            let s = Self::span_to_string(bytes, span);
            let s = s.trim();
            if !s.is_empty() {
                out.push(s.to_string());
            }
        }
        out
    }

    /// Returns true if the byte is considered part of a human-readable coinbase tag.
    /// 
    /// We intentionally restrict this to alphanumeric characters plus a small
    /// set of punctuation commonly used by pools (., _, -, /, :).
    /// 
    /// This avoids pulling in high-noise ASCII that often appears in coinbase
    /// scriptSig fields (control bytes, padding, flags, versioning data, etc).
    fn is_tag_byte(b: u8) -> bool {
        matches!(b,
            b'0'..=b'9' |
            b'a'..=b'z' |
            b'A'..=b'Z' |
            b'.' | b'_' | b'-' | b'/' | b':'
        )
    }

    /// Extracts contiguous ranges of printable tag bytes from a coinbase scriptSig.
    ///
    /// Instead of directly building strings, this returns index ranges (start, end)
    /// into the original byte buffer. This allows downstream logic to:
    ///   - merge adjacent runs across small control-byte gaps, and
    ///   - inspect gap contents (e.g. detect NUL separators).
    ///
    /// This is particularly important for OCEAN coinbase tags, where human-readable
    /// miner identifiers may be split across non-printable bytes.    
    fn extract_runs_idx(bytes: &[u8]) -> Vec<(usize, usize)> {
        let mut runs = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            while i < bytes.len() && !Self::is_tag_byte(bytes[i]) { i += 1; }
            let start = i;
            while i < bytes.len() && Self::is_tag_byte(bytes[i]) { i += 1; }
            let end = i;
            if end > start { runs.push((start, end)); }
        }
        runs
    }

    /// Merge adjacent printable runs if:
    /// - gap <= max_gap
    /// - and the gap does NOT contain a null byte (0x00)
    fn merge_runs_no_nul(bytes: &[u8], runs: &[(usize, usize)], max_gap: usize) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = Vec::new();

        for &(s, e) in runs {
            if let Some(last) = out.last_mut() {
                if s >= last.1 {
                    let gap = s - last.1;
                    if gap <= max_gap {
                        let gap_slice = &bytes[last.1..s];
                        let has_nul = gap_slice.iter().any(|&b| b == 0x00);
                        if !has_nul {
                            last.1 = e;
                            continue;
                        }
                    }
                }
            }
            out.push((s, e));
        }

        out
    }

    /// Builds a String from a merged byte span, skipping any non-tag bytes.
    ///
    /// This allows reconstruction of human-readable identifiers that were
    /// fragmented by control bytes inside the coinbase field
    /// (e.g. "BDE" + <ctrl> + "HX" → "BDEHX").
    ///
    /// Non-tag bytes are intentionally ignored rather than replaced.
    fn span_to_string(bytes: &[u8], span: (usize, usize)) -> String {
        let (start, end) = span;
        let mut s = String::with_capacity(end.saturating_sub(start));
        for &b in &bytes[start..end] {
            if Self::is_tag_byte(b) {
                s.push(b as char);
            }
        }
        s
    }


}

/// Output data from verbose block transaction.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct TxOut {
    pub value: f64,
    n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: ScriptPubKey,
}

/// ScriptPubKey metadata for outputs.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ScriptPubKey {
    pub asm: String,
    pub desc: String,
    pub hex: String,
    /// Some scripts include a decoded address; others do not.
    #[serde(default)]
    pub address: String,
    pub r#type: String,
}

/// Input data from verbose block transaction.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct TxIn {
    #[serde(default)]
    pub coinbase: Option<String>,
    #[serde(default)]
    pub txid: Option<String>,
    pub vout: Option<u32>,
    #[serde(rename = "scriptSig")]
    pub script_sig: Option<ScriptSig>,
    #[serde(default)]
    #[serde(skip)]
    #[allow(dead_code)]
    pub txinwitness: Option<Vec<String>>,
    pub sequence: u32,
}

/// ScriptSig metadata for transaction inputs.
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ScriptSig {
    pub asm: String,
    pub hex: String,
}

/// Miner → known payout address mapping.
/// Loaded from `miners.json`.
#[derive(Deserialize, Clone)]
pub struct MinersData {
    pub miners: Vec<Miner>,
}

/// A single miner record.
#[derive(Deserialize, Clone)]
pub struct Miner {
    pub name: String,
    pub wallet: String,
}

/// Rolling 24-hour miner distribution tracking.
/// Stores the last 144 block miners.
///
/// Used for the Hash Rate Distribution chart and “Last Miner” display.
pub struct BlockHistory {
    pub blocks: Mutex<VecDeque<Option<Arc<str>>>>,
}

impl BlockHistory {
    /// Create an empty 144-block rolling window.
    pub fn new() -> Self {
        BlockHistory {
            blocks: Mutex::new(VecDeque::with_capacity((BLOCKS_PER_HOUR * HOURS_PER_DAY) as usize)),
        }
    }

    /// Return up to the last `n` blocks as (height, miner).
    /// Assumes the most recent entry corresponds to `last_block`.
    pub fn last_n_with_heights(&self, last_block: u64, n: usize) -> Vec<(u64, Option<Arc<str>>)> {
        let blocks = self.blocks.lock().unwrap();

        // How many entries do we actually have?
        let k = blocks.len().min(n);

        // Take the last k entries (newest at the end)
        let tail = blocks.iter().rev().take(k);

        // Map index 0 => last_block, index 1 => last_block - 1, etc.
        tail.enumerate()
            .map(|(i, miner_opt)| (last_block.saturating_sub(i as u64), miner_opt.clone()))
            .collect()
    }

    /// Returns the miner of the most recent block (if known).
    pub fn last_miner(&self) -> Option<Arc<str>> {
        let blocks = self.blocks.lock().unwrap();
        blocks.back().cloned().flatten()
    }

    /// Add a miner label for the next block in the rolling window.
    pub fn add_block(&self, miner: Option<String>) {
        let mut blocks = self.blocks.lock().unwrap();

        if blocks.len() == 144 {
            blocks.pop_front(); // Maintain fixed-size window
        }

        blocks.push_back(miner.map(|m| Arc::from(m.into_boxed_str())));
    }

    /// Count block frequency by miner across the 144-block window.
    pub fn get_miner_distribution(&self) -> Vec<(Arc<str>, u64)> {
        let blocks = self.blocks.lock().unwrap().clone();
        let mut distribution: HashMap<Arc<str>, u64> = HashMap::new();

        for miner in blocks.iter().flatten() {
            *distribution.entry(miner.clone()).or_insert(0) += 1;
        }

        distribution.into_iter().collect()
    }
}
