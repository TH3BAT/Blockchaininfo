# **Pattern: Defining Custom Bitcoin Models for Sovereign Architecture**

## **Overview**

Most Bitcoin applications deserialize RPC responses into **third-party library structs**, inheriting:

* another developer’s assumptions
* unused fields
* heavy type trees
* rigid schema decisions
* dependency version issues
* long compile times

BlockchainInfo takes a different approach:

## ⭐ **Define every model in `src/models/` using clean, minimal Rust structs — shaped exactly to what the application needs.**

This decision unlocks the *entire architecture*.

---

## **Why This Pattern Matters**

### **1. Full Source-of-Truth Control**

When you define your own types:

```rust
pub struct BlockchainInfo { … }
pub struct MempoolInfo { … }
pub struct NetworkInfo { … }
```

You control:

* the shape
* the fields
* the naming
* the optionality
* the evolution

No dependency updates can break your UI or logic.

Your schema is **sovereign**.

---

### **2. Clean Global Caches Become Possible**

Every cache is:

```rust
pub static BLOCKCHAIN_INFO_CACHE: Lazy<Arc<RwLock<BlockchainInfo>>> = …
```

This works smoothly *because* your models are:

* lightweight
* predictable
* serializable
* immutable in shape
* tailored to your UI

Try this with 3rd-party crates and you inherit a forest of nested structs, half of which you don’t need.

---

### **3. Async Tasks Stay Simple**

Each async fetch loop writes directly into its cache without translation layers:

```rust
let mut cache = BLOCKCHAIN_INFO_CACHE.write().await;
*cache = new_info;
```

No conversions.
No adapters.
No loss.
No confusion.

The system stays clean.

---

### **4. RPC Decoding Becomes Transparent**

Bitcoin RPC → Serde → Your Struct.

That’s it.

No intermediate types.
No mismatched enums.
No custom error hacks.
No snake_case ↔ camelCase battles.

You own the truth format.

---

### **5. The UI Aligns Perfectly With the Data**

Because *I* defined the structs, the TUI sections map 1:1 to the model fields:

* Blockchain section uses `BlockchainInfo`
* Mempool section uses `MempoolInfo`
* Network section uses `NetworkInfo`
* Consensus section uses `ChainTip`

This is why rendering in BlockChainInfo feels effortless.

---

### **6. Future Growth is Effortless**

Need miners.json?
Add `MinerInfo`.

Need RBF analysis?
Add `RbfStats`.

Need difficulty epoch projections?
Add `DifficultyMetrics`.

No upstream dependency stops you.
Your models evolve organically with your app.

---

## **When to Use This Pattern**

Use custom models when:

* you want clarity
* you need independence
* your app must be stable long-term
* you want total control over shape and semantics
* you’re building a UI or dashboard
* you want clean async cache loops

This pattern is especially powerful in:

* Bitcoin monitoring tools
* dashboards
* explorers
* TUI/CLI interfaces
* node tools
* teaching projects

---

## **Pattern Summary**

| Approach                           | Result                                  |
| ---------------------------------- | --------------------------------------- |
| **3rd-party Bitcoin model crates** | Heavy, rigid, complex, dependency-bound |
| **Your own Bitcoin structs**       | Clear, fast, sovereign, maintainable    |

BlockchainInfo demonstrates:

## ⭐ **Sovereignty at the data model level creates sovereignty at the architectural level.**

This is one of the project’s defining strengths and a pattern worth studying.

---

## **Author**

**Farley & ChatGPT — 2025**
*Pioneering sovereign Rust design for Bitcoin dashboards.*
