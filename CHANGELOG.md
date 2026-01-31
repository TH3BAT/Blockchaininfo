# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/).  
v1.0.0 marks the first stable release of BlockChainInfo.

---

## **v1.2.6 - 2026-01-30**

Added

* Mempool Size Lens filtering for transaction distribution:
  * Small, Medium, and Large transaction views
  * Lens applies consistently across fee, age, and RBF metrics
* Size lens integrates with existing Dust-Free mode (filters compose cleanly)

Improved

* Operator clarity by allowing mempool statistics to reflect current transaction
context
* Fee metrics now better distinguish urgency vs typical behavior by transaction
size

Notes

* Size lens is a view filter only — no changes to underlying mempool data
* Default behavior remains unchanged when no lens is active

---

## **v1.2.5 - 2026-01-25**

* Fixed OCEAN miner attribution for multi-word upstream tags (e.g., "Peak Mining").
* Refined OCEAN coinbase parsing to extract secondary tags only from the post-banner
 region, preventing pool-tag contamination.

* Refactored transaction byte parsing helpers into the Transaction layer for improved
semantic scoping and code clarity.
* Updated dependencies:
colored 3.0.0 => 3.1.1
chrono 0.4.42 => 0.4.43

---

## **v1.2.4 - 2026-01-22**

* Improved OCEAN coinbase tag parsing to better reconstruct split miner identifiers.
* Improved client detection logic to correctly classify multi-segment user agents
(e.g. Knots + UASF-BIP110), ensuring accurate client distribution reporting.
* Synced peer filtering with version distribution to ensure consistent Bitcoin
protocol node reporting.

---

## **v1.2.3 - 2026-01-15**

Fixed

* Correct client distribution filtering by using peer protocol version (>= 70016)
* Stabilize propagation time slot updates during same-block observation
* Refine miner attribution via expanded coinbase tag mappings

---

## **v1.2.2 — 2026-01-08**

### **Miner Attribution & Network Insight**

* Restored **wallet-first miner attribution** as the primary source of truth.
* Added **table-driven coinbase tag classification** as a sanitized fallback when
wallet attribution is unavailable.
* Improved miner specificity when pools (e.g. OCEAN) expose upstream hashpower
sources, enabling displays such as:

  * `NiceHash`
  * `NiceHash (via OCEAN)`
* Hardened coinbase tag sanitization to safely handle malformed or non-ASCII data.

### **RPC Security & Reliability**

* Added **Linux password store support** for secure RPC credential retrieval.
* Exit cleanly when RPC password retrieval fails (prevents startup hangs).
* Hide RPC password input during terminal prompts.

### **Performance & Correctness**

* Reduced cache write-lock contention via read-first comparisons.
* Restored **single-slot cache invariants** for difficulty reference blocks, fixing
missing difficulty direction indicators.
* Tightened internal block constants and clarified TXID conversion paths.

### **UI & Usability**

* Centralized accent color definitions.
* Improved informational text contrast for better terminal readability.
* Graceful truncation of long labels using Unicode-aware width calculations.

### **Documentation & Maintenance**

* Added documentation for:

  * Coinbase tag sanitization and classification helpers
  * Linux RPC password support
  * `hex_decode()` utility
* Expanded internal documentation in preparation for future releases.

Notes

This release focuses on correctness, clarity, and long-term maintainability.
No user action is required.

---

## v1.2.0 — Miner history panel & consensus hygiene (2026-01-03)

Added

New Last 20 Blocks / Miners panel in the Blockchain section.
Displays recent block heights alongside their associated mining pools.
Toggleable via [L], mutually exclusive with the Hash Rate Distribution view.
Designed for quick situational awareness (newest blocks shown first).

Changed

Improved visual hierarchy across ASCII charts:
Percentage values remain crisp.
Horizontal ASCII bars are dimmed to reduce visual noise.
Miner names in the Last 20 view are rendered with secondary emphasis for clearer
pattern recognition.

Improved

Replaced hard-coded block and mempool constants with consensus-derived values.
Normalized block-range calculations using saturating arithmetic for safety and clarity.
Reduced implicit cache invalidation by correcting edge-triggered state handling.
Improved cache stability and behavior under higher-latency environments (e.g. Tor).

Notes

The Last 20 Blocks / Miners panel builds on the existing rolling block history and
does not introduce new RPC calls.
Under Tor/onion access, this view provides a lightweight alternative to heavier
distribution charts.
Startup behavior gracefully fills from 1 → 20 blocks as history accumulates.

---

## v1.1.5 — Centralize reqwest client construction (proxy-ready) (2025-12-22)

Changed

Improved RPC client handling for high-latency connections by adjusting timeouts
when a proxy is in use.
Stabilized background RPC jobs (e.g. miner detection) when accessing nodes over
Tor/onion services.

Notes

When using Start9 onion RPC interfaces, be sure to include the explicit RPC port
(:8332) at the end of the onion address.
Initial loads over Tor (e.g. mempool and transaction-related views) may take longer
than LAN connections due to network latency.

Added / Changed (Original commit)

* Centralized RPC client construction behind a single builder function.
* Unified timeout and connection behavior across all RPC modules.
* Added optional SOCKS proxy support for RPC connections (env-driven).
* Laid groundwork for future Tor/onion RPC access where supported by the node environment.

Notes

* Onion RPC support depends on upstream node/platform exposure and policies.
* Refactor provides immediate benefits even when using LAN or clearnet RPC.

---

## v1.1.4 — Refinements, Consistency, and Block-Native Hash-Rate Window (2025-12-19)

This release focuses on structural clarity, documentation improvements, and aligning
UI output with Bitcoin’s block-driven reality. No new features — just meaningful
refinements across several modules.

Changed

* Renamed ChainTipsResponse → ChainTipsJsonWrap and
  PeerInfoResponse → PeerInfoJsonWrap
  to maintain consistent naming across all JsonWrap structs.
* Hash-Rate panel title now displays the actual block window (derived from miner
  distribution counts)
  instead of a static “24 hrs” placeholder.
  This clarifies sampling scope and better reflects Bitcoin’s block-time cadence.

Improved

* Added documentation and preserved (commented-out) deserialize_wtxid for educational
  reference and
  potential future use.
* Marked wtxid with serde(skip) in MempoolEntry, removing redundant data and reducing
  memory + deserialization overhead.

Notes
This update completes the intended refinements for the v1.1.x line.
The codebase is now in a clean, consistent state ahead of introducing new views
and toggles planned in the upcoming roadmap.

---

## v1.1.3 (2025-12-17)

Fixed

* Corrected mempool transaction lookup to deserialize the full JSON-RPC response,
  matching the mempool distribution path. This replaces older result-only
  deserialization which could incorrectly surface "Transaction not found"
  despite successful RPC responses.

---

## v1.1.2 (2025-12-16)

Fix: Replaced positional assumptions with block-anchored propagation slots
Fix: Grounded fee metrics in integer sats; floats used only at render time
Docs: Reduce model memory usage by skipping unused RPC fields with serde

---

## v1.1.1 (2025-12-14)

* Satoshi: Oopsie — trim Keychain stdout before parsing RPC password
* Satoshi: Add 5-block propagation slices to observe network drift
* Satoshi: skip unused mempool dependency fields
    Mark depends/spentby with serde::skip to reduce memory usage.
* UI: shorten label for narrow terminal support

---

## v1.1.0 – Network Observability Refinement (2025-12-14)

Added

* Added a toggleable average block propagation view to the Network panel, providing
a clear numerical anchor alongside the existing sparkline for improved network health
interpretation.

---

## v1.0.2 – Optimize mempool TXID (2025-12-13)

Improved

* Reduced mempool memory usage by switching TXID storage from hex strings to fixed-size
  byte arrays (`[u8; 32]`)
* Improved DashMap key efficiency under high mempool churn
* Lower allocator pressure during sustained congestion

Internal

* Refactored mempool hot path to enforce byte-native TXID handling

---

## v1.0.1 — Stability & Foundations (2025-12-11)

Fixes & Internal Improvements

* Corrected the first hash-phase threshold (0.10 → 10.0) to accurately represent
  the 10% phase change.
* Removed unintended `[dev-dependencies]` self-referencing path to prevent Rust
  Analyzer cyclic dependency warnings.
* Updated toolchain expectations to Rust 1.90+ for compatibility with recent
  rust-analyzer behavior.
* Resolved various analyzer warnings and ensured smoother development environment
  behavior.

Documentation

* Introduced `docs/FGMO.md` — Photo-BIP for the **Floating Global Mesh Observer**,
  outlining the conceptual
  architecture for an optional, sovereign, global operator observability mesh.

Notes

This release does not modify runtime behavior of the dashboard.
It consolidates correctness, improves developer environment stability, and
anchors a new conceptual direction for future Layer-3 observability features.

---

## [1.0.0] – 2025-12-11

### Sovereign Release

Added

* **Full inline documentation (Hybrid Mode)** across all modules  
  (RPC, display, models, utils, run loop).
* **Consensus Security Panel** with fork detection and stale-branch warnings.
* **Hashrate Distribution Overlay** (toggle: `H`) with miner attribution.
* **Dust-Free Mempool Filtering** (toggle: `D`) with optimized distribution sampling.
* **Client ↔ Version Toggle** (Network Panel) for improved node diversity insight.
* **Graceful Shutdown Flow** with last-frame rendering.
* **Popup System Enhancements**:
  * Transaction Lookup (TxID validation, paste detection)
  * Help Panel
  * Consensus Warning Dialog
* **Propagation Time Tracking Enhancements** with block-change detection.
* **Dynamic Async Polling Intervals** for optimal UI smoothness.

Changed

* Improved UI consistency across all panels with unified toggle styling.
* Refined layout for Blockchain, Mempool, Network, and Consensus sections.
* Optimized cache update strategy to avoid redundant writes.
* Cleaned concurrency logic to reduce lock contention.
* Consolidated and clarified miner distribution tracking in `BLOCK_HISTORY`.

Fixed

* Stabilized safe indexing throughout UI rendering paths.
* Eliminated intermittent stale-data issues in async polling tasks.
* Corrected handling of chain tip responses.
* Improved paste behavior within the Tx Lookup popup.
* Resolved minor timing jitter during redraw cycles.

---

## Past Versions (Pre-Stable Series)

Prior versions represent the experimental, pre-1.0 development phase.  
Highlights included early dashboard rendering, miner distribution prototypes,
propagation-time logic, and initial TUI features.

For historical context, previous entries remain below:

<details>
<summary>Show Pre-1.0.0 History</summary>

### [0.6.3], [0.3.1], [0.2.12], [0.2.11], and

others  
*(Full historical*
