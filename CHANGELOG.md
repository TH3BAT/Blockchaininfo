# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/).  
v1.0.0 marks the first stable release of BlockChainInfo.

---

## v1.1.2 (2025-12-16)

Fix: Replaced positional assumptions with block-anchored propagation slots
Fix: Grounded fee metrics in integer sats; floats used only at render time
Docs: Reduce model memory usage by skipping unused RPC fields with serde


---

## v1.1.1 (2025-12-14)

- Satoshi: Oopsie — trim Keychain stdout before parsing RPC password
- Satoshi: Add 5-block propagation slices to observe network drift
- Satoshi: skip unused mempool dependency fields
    Mark depends/spentby with serde::skip to reduce memory usage.
- UI: shorten label for narrow terminal support

---

## v1.1.0 – Network Observability Refinement (2025-12-14)

### Added

- Added a toggleable average block propagation view to the Network panel, providing a clear numerical anchor alongside the existing sparkline for improved network health interpretation.

---

## v1.0.2 – Optimize mempool TXID (2025-12-13)

### Improved

- Reduced mempool memory usage by switching TXID storage from hex strings to fixed-size byte arrays (`[u8; 32]`)
- Improved DashMap key efficiency under high mempool churn
- Lower allocator pressure during sustained congestion

### Internal

- Refactored mempool hot path to enforce byte-native TXID handling

---

## v1.0.1 — Stability & Foundations (2025-12-11)

### Fixes & Internal Improvements

- Corrected the first hash-phase threshold (0.10 → 10.0) to accurately represent the 10% phase change.
- Removed unintended `[dev-dependencies]` self-referencing path to prevent Rust Analyzer cyclic dependency warnings.
- Updated toolchain expectations to Rust 1.90+ for compatibility with recent rust-analyzer behavior.
- Resolved various analyzer warnings and ensured smoother development environment behavior.

### Documentation

- Introduced `docs/FGMO.md` — Photo-BIP for the **Floating Global Mesh Observer**, outlining the conceptual
  architecture for an optional, sovereign, global operator observability mesh.

### Notes

This release does not modify runtime behavior of the dashboard.
It consolidates correctness, improves developer environment stability, and
anchors a new conceptual direction for future Layer-3 observability features.

---

## [1.0.0] – 2025-12-11

### Sovereign Release

### Added

- **Full inline documentation (Hybrid Mode)** across all modules  
  (RPC, display, models, utils, run loop).
- **Consensus Security Panel** with fork detection and stale-branch warnings.
- **Hashrate Distribution Overlay** (toggle: `H`) with miner attribution.
- **Dust-Free Mempool Filtering** (toggle: `D`) with optimized distribution sampling.
- **Client ↔ Version Toggle** (Network Panel) for improved node diversity insight.
- **Graceful Shutdown Flow** with last-frame rendering.
- **Popup System Enhancements**:
  - Transaction Lookup (TxID validation, paste detection)
  - Help Panel
  - Consensus Warning Dialog
- **Propagation Time Tracking Enhancements** with block-change detection.
- **Dynamic Async Polling Intervals** for optimal UI smoothness.

### Changed

- Improved UI consistency across all panels with unified toggle styling.
- Refined layout for Blockchain, Mempool, Network, and Consensus sections.
- Optimized cache update strategy to avoid redundant writes.
- Cleaned concurrency logic to reduce lock contention.
- Consolidated and clarified miner distribution tracking in `BLOCK_HISTORY`.

### Fixed

- Stabilized safe indexing throughout UI rendering paths.
- Eliminated intermittent stale-data issues in async polling tasks.
- Corrected handling of chain tip responses.
- Improved paste behavior within the Tx Lookup popup.
- Resolved minor timing jitter during redraw cycles.

---

## Past Versions (Pre-Stable Series)

Prior versions represent the experimental, pre-1.0 development phase.  
Highlights included early dashboard rendering, miner distribution prototypes, propagation-time logic, and initial TUI features.

For historical context, previous entries remain below:

<details>
<summary>Show Pre-1.0.0 History</summary>

### [0.6.3], [0.3.1], [0.2.12], [0.2.11], and

others  
*(Full historical*
