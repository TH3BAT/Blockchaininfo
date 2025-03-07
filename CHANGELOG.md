# [v0.2.11] - 2025-03-06

## Added

- Input validation for transaction IDs.
- User-friendly error messages for invalid inputs.

### Changed

- Optimized `LOGGED_TXS` to use `Arc<String>` for memory efficiency.

### Fixed

- Fixed a crash caused by invalid input in the transaction lookup function.