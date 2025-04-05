# [v0.3.1] - 2025-03-24

Added

- **Memory Efficiency Improvements**:
  - `Arc<str>` usage for miner data in BlockHistory and distribution functions
  - Optimized cache update logic with change detection
- **Performance Enhancements**:
  - Improved spawn logic to skip redundant operations
  - Streamlined block number/propagation time handling

Changed

- Updated `tokio` to v1.44.1
- Updated `reqwest` to v0.12.15
- Narrowed Version Distribution chart bars for better terminal compatibility

Fixed

- Improved error messaging for missing miners.json
- Removed redundant conversions and improved type safety
- Optimized PeerInfo cache updates to reuse allocations

---

[v3.0.0] - 2025-03-10

Added

- **Last Miner in Best Block View**:
  - Display the last miner in the best block view for better insights.
- **Hash Rate Distribution Chart**:
  - A new toggleable chart showing the distribution of hash rate over the past 24 hours.
  - Toggle between the original metrics and the chart using the `h` key.
  - Displays up to the **Top 8 miners** of x rewarded in the past 144 blocks.
  - Tracks data from the moment the dashboard loads (no historical data is loaded).
- **miners.json Support**:
  - A new file to map coinbase wallet addresses to miner names.
  - Loaded at runtime and can be maintained by end-users.
  - File must be placed in the same folder as the executable (default location if running from the Blockchaininfo parent folder).

Changed

- Updated `serde` to v1.0.219.
- Updated `once_cell` to v1.21.0.

Fixed

- Improved handling of miner data to ensure accurate display in the Hash Distribution Chart.

---

[v0.2.12] - 2025-03-08

Added

- Paste detection to handle large chunks of text gracefully.
- Neutral UI message ("Press Enter to validate TxID") while typing or pasting.

Changed

- Delayed TxID validation until Enter is pressed, eliminating red text flashes.

Fixed

- Prevented accidental app quit during pasting (thanks, bumper sticker manifesto 🚗).
- Ensured invalid TxID message only appears after Enter is pressed.

---

[v0.2.11] - 2025-03-06

Added

- Input validation for transaction IDs.
- User-friendly error messages for invalid inputs.

Changed

- Optimized `LOGGED_TXS` to use `Arc<String>` for memory efficiency.

Fixed

- Fixed a crash caused by invalid input in the transaction lookup function.
