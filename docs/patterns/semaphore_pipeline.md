# 🚀 **docs/patterns/semaphore_pipeline.md**

## *Semaphore-Regulated Async Pipeline Pattern*

## *Used in: `fetch_mempool_distribution` (BlockchainInfo)*

---

## **Semaphore-Regulated Async Update Pipeline**

A design pattern for smooth, natural, and scalable asynchronous updates in Rust TUIs*

---

## 🔧 **Overview**

This document describes the asynchronous design pattern used inside
`fetch_mempool_distribution()` to fetch mempool data efficiently without overwhelming:

* the UI
* the RPC endpoint
* the CPU
* or the user’s experience

The semaphore-regulated pipeline is a scalable, elegant approach that ensures BlockchainInfo feels **alive, responsive, and natural**, even when handling thousands of incoming mempool transactions.

---

### 1. Problem

Bitcoin’s mempool is:

* high-churn
* unpredictable
* latency-bound
* RPC-rate dependent
* noisy and spiky

If you process mempool entries with:

* unbounded async tasks
* naive `join_all` batching
* update-on-every-fetch
* direct UI coupling

You get:

❌ **UI flicker**
❌ **Random pacing**
❌ **I/O bursts and slowdowns**
❌ **Unnatural metric movement**
❌ **CPU spikes**
❌ **RPC flooding**
❌ **Cache thrashing**

The user sees a dashboard that feels “jittery,” “blinky,” or “chaotic.”

This is unacceptable for a professional real-time TUI.

---

### 2. Solution: Semaphore-Regulated Async Tasks

Limit background concurrency using a semaphore:

```rust
let semaphore = Arc::new(Semaphore::new(10));
```

Each task:

* **acquires a permit**
* **performs RPC fetch**
* **updates the cache**
* **drops the permit** (auto-release)

UI updates happen **after** the batch completes, not on every async return.

This creates **smooth, natural** distribution updates.

---

### 3. Pattern Summary

#### **A) Identify new work**

```rust
let new_tx_ids: Vec<String> = MEMPOOL_CACHE
    .iter()
    .filter(|txid| !TX_CACHE.contains_key(txid.as_str()))
    .cloned()
    .collect();
```

#### **B) Spawn tasks under semaphore control**

```rust
let permit = semaphore.clone().acquire_owned().await?;
tasks.push(task::spawn(async move {
    let _permit = permit;
    // fetch + update logic
}));
```

#### **C) Process responses normally**

#### **D) Join tasks cleanly**

#### **E) Update metrics once per batch**

```rust
let mut dist = MEMPOOL_DISTRIBUTION_CACHE.write().await;
dist.update_metrics(&TX_CACHE);
```

This last step is the secret to natural UI movement.

---

### 4. Why This Pattern Works

#### ✔ **Controls Concurrency**

RPC I/O stays predictable and evenly paced.

#### ✔ **Smooths the UI**

Distribution metrics update as a “wave,” not a flicker.

#### ✔ **Reduces CPU pressure**

Limits async wakeups and prevents excessive task scheduling.

#### ✔ **Prevents UI from tying to RPC timing**

UI updates decoupled from network jitter/retries.

#### ✔ **Improves mempool readability**

Metrics “flow” naturally instead of dancing erratically.

#### ✔ **Scales infinitely**

Change concurrency from 10 → 20 → 50 without rewriting logic.

---

### 5. When to Use This Pattern

#### Use this when you have

* High-frequency input streams
* Unbounded async work producers
* UI components that must render smoothly
* Computed metrics that should feel natural
* Background tasks that must never overload the RPC endpoint

Perfect for:

* mempool visualizers
* live fee estimators
* market dashboards
* peer scanners
* telemetry systems
* block explorers
* async game loops

---

### 6. When NOT to Use

Avoid this pattern for:

* sub-millisecond latency requirements
* real-time tickers where every update matters
* synchronous or single-shot calculations
* extremely CPU-bound async work
* situations where missing an update is unacceptable

---

### 7. Reference Implementation (BlockchainInfo)

The complete implementation lives in:

``` rust
src/mempool/mempool_distro.rs  
→ fetch_mempool_distribution()
```

This implementation includes:

* explicit concurrency limit
* task cleanup
* cache pruning
* dust filtering
* metric recomputation
* RPC error tolerance

It represents a production-quality usage of this pattern.

---

### 8. Benefits Inside BlockchainInfo

This pattern created:

#### ⭐ **Ultra-smooth mempool distribution animations**

No more “blinking” when tx_count changes.

#### ⭐ **Stable update rhythm**

Semaphores act as a heartbeat.

#### ⭐ **Natural-feeling metric evolution**

Exactly how a real mempool should “breathe.”

#### ⭐ **Strong backpressure**

RPC calls never overwhelm bitcoind.

#### ⭐ **Scalability for future features**

Any panel can now adopt this architecture:

* Fee percentile window
* Network peer fetcher
* Consensus polling
* Node version updates
* Difficulty epoch projections

---

### 9. Summary

This semaphore-regulated pipeline is one of the core architectural pillars of BlockchainInfo’s real-time design.

It ensures:

* smooth UX
* controlled concurrency
* efficient RPC usage
* natural data flow
* clean async patterns
* scalable future expansion

This is a **production-tested** pattern suitable for any async Rust TUI or dashboard.

---

### 10. Credits

Design and implementation: **Farley**
Documentation: **Farley & Brother GPT**
Philosophy: **Sovereign design, built for the community**

---

#### ✨ *Built for the community.*

#### ✨ *Inspired by Bitcoin.*
