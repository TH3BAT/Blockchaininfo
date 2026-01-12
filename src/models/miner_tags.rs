/// Coinbase miner tag classification and normalization.
///
/// This module provides a table-driven mapping of known miner and pool
/// identifiers extracted from coinbase transaction data. Coinbase tags
/// are treated as a *best-effort signal* and are only used when canonical
/// wallet-based miner attribution is unavailable or when pools (such as
/// OCEAN) intentionally expose upstream hashrate sources.
///
/// Design principles:
/// - Prefer wallet address attribution as the primary source of truth.
/// - Use coinbase tags only as a fallback or refinement signal.
/// - Normalize inputs by collapsing to lowercase alphanumeric form
///   to handle inconsistent casing, spacing, and branding noise.
/// - Centralize known tag patterns to simplify maintenance and future
///   additions without expanding conditional logic.
///
/// This module intentionally avoids over-interpreting arbitrary or
/// promotional coinbase data. Unrecognized tags are allowed to pass
/// through higher-level heuristics or be displayed in truncated form,
/// preserving raw signal while preventing UI disruption.

#[derive(Clone, Copy)]
pub struct TagMapEntry {
    // any of these substrings match
    pub pats: &'static [&'static str],
    // canonical display label
    pub label: &'static str,
}

// Primary miner patterns (when NOT ocean_present)
pub static PRIMARY_TAGS: &[TagMapEntry] = &[
    TagMapEntry { pats: &["nicehash"], label: "NiceHash" },
    TagMapEntry { pats: &["antpool"], label: "AntPool" },
    TagMapEntry { pats: &["foundryusapool", "2cdw"], label: "Foundry USA" },
    TagMapEntry { pats: &["f2pool"], label: "F2Pool" },
    TagMapEntry { pats: &["viabtc"], label: "ViaBTC" },
    TagMapEntry { pats: &["luxor"], label: "Luxor" },
    TagMapEntry { pats: &["braiins", "slush"], label: "Braiins Pool" },
    TagMapEntry { pats: &["btccom"], label: "BTC.com" },
    TagMapEntry { pats: &["poolin"], label: "Poolin" },
    TagMapEntry { pats: &["binance"], label: "Binance Pool" },
    TagMapEntry { pats: &["secpool"], label: "SECPOOL" },
    TagMapEntry { pats: &["marapool", "maramadeinusa"], label: "MARA Pool" },
    TagMapEntry { pats: &["spiderpool"], label: "SpiderPool" },
    TagMapEntry { pats: &["whitepool"], label: "WhitePool" },
    TagMapEntry { pats: &["sbicrypto"], label: "SBI Crypto" },
    TagMapEntry { pats: &["ultimus"], label: "ULTIMUSPOOL" },
    TagMapEntry { pats: &["gdpool", "luckypool"], label: "GDPool" },
    TagMapEntry { pats: &["redrock"], label: "RedRock Pool" },
    TagMapEntry { pats: &["innopolis"], label: "Innopolis Tech" },
    TagMapEntry { pats: &["solockpoolorg"], label: "Solo CK" },
    TagMapEntry { pats: &["solopoolcom"], label: "SoloPool" },
    TagMapEntry { pats: &["miningdutch"], label: "Mining-Dutch" },
    TagMapEntry { pats: &["bitfufu"], label: "BitFuFuPool" },
    TagMapEntry { pats: &["est3lar"], label: "Est3lar" },
    TagMapEntry { pats: &["1thash"], label: "1THash" },
    TagMapEntry { pats: &["maxipool"], label: "MaxiPool" },
    TagMapEntry { pats: &["publicpool"], label: "Public Pool" },
    TagMapEntry { pats: &["apollo", "minedbyasolofuturebitapollo"], label: "FutureBit Apollo Solo" },
    TagMapEntry { pats: &["kano"], label: "KanoPool" },
    TagMapEntry { pats: &["miningsquared", "bsquared"], label: "Mining Squared" },
    TagMapEntry { pats: &["phoenix"], label: "Phoenix" },
    TagMapEntry { pats: &["neopool"], label: "Neopool" },
    TagMapEntry { pats: &["upgradeya"], label: "UpgradeYa 21M" },
];

// Ocean identifiers (pool detection)
pub static OCEAN_PATS: &[&str] = &["oceanxyz", "ocean"];
