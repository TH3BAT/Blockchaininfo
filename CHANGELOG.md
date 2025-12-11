# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/).  
v1.0.0 marks the first stable release of BlockChainInfo.

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
