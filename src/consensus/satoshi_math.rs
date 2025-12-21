// -------------------------------------------------------------
// satoshi_math.rs
//
// Consensus timing constants expressed in Satoshi-style formulas.
// These are not arbitrary numbers — they are the mathematical
// structure of Bitcoin’s time cadence.
//
// We avoid “magic constants” like 600 or 2016, and instead
// express them as unit-based formulas to preserve both clarity
// and intention.
//
// -------------------------------------------------------------

/// Number of seconds in one minute.
/// Satoshi rarely hardcoded time values — he multiplied units.
pub const SECONDS_PER_MINUTE: u64 = 60;

/// Bitcoin’s target time per block: 10 minutes.
/// This is the basis for the difficulty adjustment mechanism.
pub const TARGET_MINUTES_PER_BLOCK: u64 = 10;

/// Expected block interval in seconds.
/// (10 minutes × 60 seconds)
pub const BLOCK_TIME_SECONDS: u64 =
    TARGET_MINUTES_PER_BLOCK * SECONDS_PER_MINUTE;

// -------------------------------------------------------------
// Difficulty Adjustment Interval
//
// Satoshi defined difficulty over a *period of two weeks*,
// not by a raw constant like “2016”. The value 2016 comes from:
//
//   6 blocks/hour × 24 hours/day × 14 days/difficulty period
//
// Expressing this formula preserves the meaning.
// -------------------------------------------------------------

/// Blocks produced (on average) each hour at 10-minute intervals.
pub const BLOCKS_PER_HOUR: u64 = 6;

/// Hours per day.
pub const HOURS_PER_DAY: u64 = 24;

/// Length of the difficulty period in days.
pub const DIFFICULTY_PERIOD_DAYS: u64 = 14;

/// Total blocks in one difficulty adjustment window.
/// (6 × 24 × 14 = 2016)
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 =
    BLOCKS_PER_HOUR * HOURS_PER_DAY * DIFFICULTY_PERIOD_DAYS;

// -------------------------------------------------------------
// Notes:
// - These values are consensus truths, not configurable settings.
// - They are written as formulas to reflect how Satoshi conveyed
//   system behavior in the original source code and documentation.
// - Using formulas instead of constants improves clarity,
//   reduces the chance of misunderstanding, and preserves the
//   pedagogical nature of the BCI codebase.
// -------------------------------------------------------------
