//
// benches/benchmark.rs
//
/// -------------------------------------------------------------------------------- 
/// Running benches/benchmark.rs (target/release/deps/benchmark-005d84f413f11729)
/// formatted_chainwork_bits
///                         time:   [223.89 ns 224.09 ns 224.35 ns]
///                         change: [-0.0890% +0.0386% +0.1733%] (p = 0.57 > 0.05)
///                         No change in performance detected.
/// Found 9 outliers among 100 measurements (9.00%)
///   4 (4.00%) low mild
///   5 (5.00%) high severe
/// 
/// formatted_difficulty    time:   [154.90 ns 155.19 ns 155.51 ns]
///                         change: [-0.1123% +0.1220% +0.3936%] (p = 0.34 > 0.05)
///                         No change in performance detected.
/// Found 14 outliers among 100 measurements (14.00%)
///   4 (4.00%) high mild
///   10 (10.00%) high severe
/// 
/// parse_mediantime        time:   [115.64 ns 115.80 ns 115.94 ns]
///                         change: [-0.3696% -0.1814% +0.0139%] (p = 0.07 > 0.05)
///                         No change in performance detected.
/// Found 20 outliers among 100 measurements (20.00%)
///   16 (16.00%) low mild
///   4 (4.00%) high mild
/// 
/// parse_time              time:   [114.90 ns 115.10 ns 115.35 ns]
///                         change: [-0.2491% -0.0472% +0.1669%] (p = 0.66 > 0.05)
///                         No change in performance detected.
/// 
/// calculate_time_diff     time:   [87.446 ns 87.562 ns 87.724 ns]
///                         change: [-1.2060% -1.0264% -0.8292%] (p = 0.00 < 0.05)
///                         Change within noise threshold.
/// Found 14 outliers among 100 measurements (14.00%)
///   3 (3.00%) high mild
///   11 (11.00%) high severe
/// ------------------------------------------------------------------------------------

use criterion::{criterion_group, criterion_main, Criterion};
use blockchaininfo::models::blockchain_info::BlockchainResult;


fn bench_formatted_chainwork_bits(c: &mut Criterion) {
    let result = BlockchainResult {
        bestblockhash: "00000000000000000000919329462180d22b1a3c51761b64832b8047a2554f2d".to_string(),
        blocks: 1000000,
        chain: "main".to_string(),
        chainwork: "00000000000000000000000000000000000000009ee5b59a20b79d3f8e277a28".to_string(),
        difficulty: 103919634711492.2, // Example value
        headers: 873627,
        initialblockdownload: false,
        mediantime: 1609459200, // Example timestamp (January 1, 2021)
        pruned: false,
        size_on_disk: 500000000, // Example size in bytes
        time: 1609459200, // Example timestamp
        verificationprogress: 0.9999912438474318,
        warnings: "".to_string(),
    };

    c.bench_function("formatted_chainwork_bits", |b| {
        b.iter(|| result.formatted_chainwork_bits());
    });
}

fn bench_formatted_difficulty(c: &mut Criterion) {
    let result = BlockchainResult {
        bestblockhash: "00000000000000000000919329462180d22b1a3c51761b64832b8047a2554f2d".to_string(),
        blocks: 1000000,
        chain: "main".to_string(),
        chainwork: "00000000000000000000000000000000000000009ee5b59a20b79d3f8e277a28".to_string(),
        difficulty: 103919634711492.2, // Example value
        headers: 873627,
        initialblockdownload: false,
        mediantime: 1609459200, // Example timestamp (January 1, 2021)
        pruned: false,
        size_on_disk: 500000000, // Example size in bytes
        time: 1609459200, // Example timestamp
        verificationprogress: 0.9999912438474318,
        warnings: "".to_string(),
    };

    c.bench_function("formatted_difficulty", |b| {
        b.iter(|| result.formatted_difficulty());
    });
}

fn bench_parse_mediantime(c: &mut Criterion) {
    let result = BlockchainResult {
        bestblockhash: "00000000000000000000919329462180d22b1a3c51761b64832b8047a2554f2d".to_string(),
        blocks: 1000000,
        chain: "main".to_string(),
        chainwork: "00000000000000000000000000000000000000009ee5b59a20b79d3f8e277a28".to_string(),
        difficulty: 103919634711492.2, // Example value
        headers: 873627,
        initialblockdownload: false,
        mediantime: 1609459200, // Example timestamp (January 1, 2021)
        pruned: false,
        size_on_disk: 500000000, // Example size in bytes
        time: 1609459200, // Example timestamp
        verificationprogress: 0.9999912438474318,
        warnings: "".to_string(),
    };

    c.bench_function("parse_mediantime", |b| {
        b.iter(|| result.parse_mediantime());
    });
}

fn bench_parse_time(c: &mut Criterion) {
    let result = BlockchainResult {
        bestblockhash: "00000000000000000000919329462180d22b1a3c51761b64832b8047a2554f2d".to_string(),
        blocks: 1000000,
        chain: "main".to_string(),
        chainwork: "00000000000000000000000000000000000000009ee5b59a20b79d3f8e277a28".to_string(),
        difficulty: 103919634711492.2, // Example value
        headers: 873627,
        initialblockdownload: false,
        mediantime: 1609459200, // Example timestamp (January 1, 2021)
        pruned: false,
        size_on_disk: 500000000, // Example size in bytes
        time: 1609459200, // Example timestamp
        verificationprogress: 0.9999912438474318,
        warnings: "".to_string(),
    };

    c.bench_function("parse_time", |b| {
        b.iter(|| result.parse_time());
    });
}

fn bench_calculate_time_diff(c: &mut Criterion) {
    let result = BlockchainResult {
        bestblockhash: "00000000000000000000919329462180d22b1a3c51761b64832b8047a2554f2d".to_string(),
        blocks: 1000000,
        chain: "main".to_string(),
        chainwork: "00000000000000000000000000000000000000009ee5b59a20b79d3f8e277a28".to_string(),
        difficulty: 103919634711492.2, // Example value
        headers: 873627,
        initialblockdownload: false,
        mediantime: 1609459200, // Example timestamp (January 1, 2021)
        pruned: false,
        size_on_disk: 500000000, // Example size in bytes
        time: 1609459200, // Example timestamp
        verificationprogress: 0.9999912438474318,
        warnings: "".to_string(),
    };

    c.bench_function("calculate_time_diff", |b| {
        b.iter(|| result.calculate_time_diff());
    });
}

criterion_group!(
    benches,
    bench_formatted_chainwork_bits,
    bench_formatted_difficulty,
    bench_parse_mediantime,
    bench_parse_time,
    bench_calculate_time_diff
);
criterion_main!(benches);


