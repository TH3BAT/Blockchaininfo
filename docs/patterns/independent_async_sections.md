# 🚀 **docs/patterns/independent_async_sections.md**

## *Independent Async Section Loops + Global Cached State Pattern*

## *Used throughout: `run_app()` in BlockchainInfo*

---

## **Independent Async Section Loops + Global Cached State Pattern**

*A scalable architecture for smooth, decoupled, real-time dashboards in async Rust.*

---

## 🔧 **Overview**

This document explains the architectural pattern used in `run_app()` to drive BlockchainInfo’s real-time terminal dashboard.

The core idea:

### **Each major dashboard section runs in its own asynchronous loop, with its own timing, its own error handling, and its own cache.**

This creates:

* smooth UI updates
* decoupled data pipelines
* fault-tolerant subsystems
* natural refresh rhythms
* zero blocking
* zero jitter
* infinitely scalable architecture

This is a production-grade approach to building real-time dashboards — and one of the foundational patterns that makes BlockchainInfo feel “alive” and stable.

---

### 1. Problem

Real-time dashboards often suffer from:

* one giant async loop
* mixed RPC calls with different speeds
* UI stutter or freezes
* sequential bottlenecks
* inconsistent snapshots
* partial updates
* cascading failures
* RPC burst traffic
* hard-to-manage timing logic
* blocking tasks inside the UI loop

When all RPC calls are tied together:

* slow endpoints delay fast ones
* failures propagate
* UI timing becomes unpredictable
* the user sees jitter, flicker, or stalls

This architecture does **not scale**.

---

### 2. Solution: Fully Independent Async Loops Per Section

BlockchainInfo solves this by giving each data subsystem its own asynchronous loop, each with its own sleep interval and error boundary:

```Rust
Blockchain Info     → updates every 2 seconds
Block Data          → updates every 2 seconds
24h Block Data      → updates every 2 seconds
Mempool Info        → updates every 3 seconds
Network Info        → updates every 7 seconds
Peer Info           → updates every 7 seconds
Chain Tips          → updates every 10 seconds
Net Totals          → updates every 7 seconds
Mempool Distribution→ updates every 2 seconds
```

Each loop:

* clones the RPC config
* fetches its own dataset
* writes into a dedicated global cache (RwLock)
* handles its own errors
* maintains its own pacing

**No loop blocks or depends on any other.**

This creates complete subsystem independence.

---

### 3. Architecture

### Each section has

#### ✔ **Its own async `tokio::spawn` loop**

```rust
tokio::spawn({
    let config_clone = config.clone();
    async move {
        loop {
            let start = Instant::now();
            // fetch, cache, handle errors
            dynamic_sleep(start, 3).await;
        }
    }
});
```

---

#### ✔ **Its own dynamic refresh rate**

Each loop measures its own runtime and sleeps the difference:

```rust
let elapsed = start.elapsed();
if elapsed < Duration::from_secs(3) {
    sleep(Duration::from_secs(3) - elapsed).await;
}
```

This ensures:

* smooth human timing
* no drift
* no rapid bursts
* no jitter
* consistent UI heartbeat

---

#### ✔ **Its own failure containment**

If peer info fails, the mempool still updates.
If mempool fails, chain tips still update.
If chain tips fail, network still updates.

One subsystem **never** blocks another.

---

#### ✔ **Its own cache**

Each subsystem writes to its own:

```rust
pub static XYZ_CACHE: Lazy<Arc<RwLock<XYZType>>> = ...
```

These caches act as:

* real-time state snapshots
* thread-safe data sources
* update boundaries
* UI read entry points

---

### 4. Unified Snapshot Reads

Inside the UI loop, all caches are read simultaneously using:

```rust
let (
    blockchain_info,
    mempool_info,
    network_info,
    peer_info,
    block_info,
    block24_info,
    net_totals,
    distribution,
    chaintips_info,
) = tokio::join!(
    BLOCKCHAIN_INFO_CACHE.read(),
    MEMPOOL_INFO_CACHE.read(),
    NETWORK_INFO_CACHE.read(),
    PEER_INFO_CACHE.read(),
    BLOCK_INFO_CACHE.read(),
    BLOCK24_INFO_CACHE.read(),
    NET_TOTALS_CACHE.read(),
    MEMPOOL_DISTRIBUTION_CACHE.read(),
    CHAIN_TIP_CACHE.read(),
);
```

This yields a **consistent** snapshot across the entire dashboard.

The user always sees a coherent view.

---

### 5. Why This Pattern Works So Well

#### ✔ **Smooth UI updates**

Each section updates naturally, without being tied to other RPC endpoints.

#### ✔ **Zero blocking**

The UI loop is *never* held hostage by remote calls.

#### ✔ **Fault isolation**

Failures in one subsystem never affect others.

#### ✔ **Natural timing rhythm**

Sections refresh according to how often *they* change.

#### ✔ **True concurrency**

All data streams update in parallel.

#### ✔ **Instant scalability**

New sections simply require:

* a new async loop
* a new cache

No architectural changes.

#### ✔ **Lightning-fast UI**

Because the UI reads cached data only — never RPC.

#### ✔ **Perfect for Bitcoin data**

Different RPC endpoints change at different intervals:

* `getblockchaininfo` → high frequency
* `getpeerinfo` → slower
* `getchaintips` → slower
* `getmempoolinfo` → medium

I tuned each loop to match its *natural volatility*.

This is why BlockchainInfo “feels” right when watching it.

---

### 6. When to Use This Pattern

Use this architecture for any dashboard where:

* multiple data sources exist
* some data changes faster/slower
* faults are expected
* RPC calls cost time
* UI updates must be smooth
* concurrency matters
* scalability matters
* latency matters
* jitter must be avoided

Perfect for:

* Bitcoin dashboards
* market dashboards
* network monitors
* telemetry systems
* SRE dashboards
* blockchain explorers
* async game engines
* distributed system viewers

---

### 7. When NOT to Use

Avoid this pattern when:

* strict synchronization is required
* all subsystems must update atomically
* data dependencies chain tightly
* there is a single unified RPC call
* refresh rate must be 100% deterministic

---

### 8. Reference Implementation

The full implementation lives in:

```Rust
src/runapp.rs
```

Primary components:

* independent async loops
* per-section caches
* dynamic pacing
* fault-contained subsystem design
* `tokio::join!` snapshot
* never blocking the UI thread

This is a robust, production-grade real-time architecture.

---

### 9. Summary

The **Independent Async Sections + Global Cached State Pattern** is one of the key architectural drivers of BlockchainInfo.

It provides:

* natural UI flow
* subsystem independence
* smooth, predictable updates
* error containment
* infinite scalability
* clean async behavior

Combined with the Semaphore-Regulated Async Pipeline, this pattern forms the backbone of BlockchainInfo’s reliability and polish.

---

### 10. Credits

Architecture & Implementation: **Farley**
Documentation & Collaboration: **Brother GPT**

---

### ✨ *Built for the community.*

### ✨ *Inspired by Bitcoin.*
