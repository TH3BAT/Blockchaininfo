
# **🚀 ROADMAP.md — 2025 Sovereign Edition**

*A modern, accurate roadmap for BlockchainInfo, reflecting the tool as it exists
today.*

---

## **1. Philosophy of the Roadmap**

BlockchainInfo has evolved into a **Bitcoin telemetry instrument** —
a sovereign, real-time observatory revealing the living patterns of:

* hash power
* consensus health
* miner identity
* mempool energy
* node diversity

This roadmap is built on three principles:

### **Accuracy** — show what *is*, not what “should be.”

### **Clarity** — every metric has purpose

### **Sovereignty** — no APIs, no middlemen; pure RPC

---

## **2. Completed Milestones (v0.6 → v1.0 Era)**

### ✔ **Independent Async Engine**

All sections update independently with zero UI blocking.

### ✔ **Global Cache Pattern**

Every dataset available instantly, shared across the entire app.

### ✔ **Semaphore-Controlled Mempool Sampling**

Prevents RPC overload during tx storms.

### ✔ **TX Lookup Popup**

Instant search via RPC, paste detection, backspace-safe input.

### ✔ **Help Panel**

Minimal, clean, self-documenting UX.

### ✔ **Hash Phase System**

Live visualization of miner progress inside the 2016-block difficulty epoch.

### ✔ **Miner Identity via `miners.json`**

Human-readable pool identification for every new block.

### ✔ **Consensus Fork Monitoring**

Active + stale branches shown in real time.

### ✔ **⭐ Consensus Alerts v2 (Completed)**

Popup triggers when any fork reaches **length ≥ 2**, warning the user of:

* potential reorg
* chain instability
* miner conflict
* consensus turbulence

Includes cooldown to prevent spam.
This is a flagship BlockchainInfo innovation.

### ✔ **Dust-Free Fee Signal**

Pure mempool analytics, undistorted by spam, dust, or non-minable tx.

---

## **3. Upcoming Enhancements (v1.2 → v1.3)**

These are real ideas — not promises — designed to expand clarity without bloating
the interface.

---

### **🔥 3.1 — Hash Phase Line Toggle**

Show the last epoch’s hash rate at:

* 10%
* 25%
* 50%
* 75%
* 100%

A 5-point sparkline or micro-chart revealing miner momentum through the epoch.

---

### **🔥 3.2 — Miner Region Classification**

`miners.json` expansion:

* pool name
* region/country
* wallet cluster
* optional anonymity flags

Allows small, optional region distribution displays.

---

### **🔥 3.3 — Bandwidth Sparkline**

Based on net totals:

* live inbound/outbound bytes
* optional toggle
* minimal footprint

---

### **🔥 3.4 — Blockchain Snapshot Export (Text)**

Single key → exports:

* block height
* hash phase
* mempool state
* network stats
* fork status
* miner identity

A simple archival feature for node operators.

---

### **🔥 3.5 — 30 Chain-Day Miner Trend (Block-Based Momentum)**

Implemented in 1.4.0-beta

A rolling 30 chain-day miner trend view based on actual blocks rewarded (144 blocks
= 1 chain-day).

This feature reveals which miners and pools are gaining or losing momentum over
time by tracking block distribution across consecutive chain-days.

Intent:
Provide operators with a clean, manipulation-resistant signal of miner trend and
network participation shifts — based purely on block reality, not hash-rate
estimates or external metrics.

Design:

* Rolling 30 chain-day window
* Each chain-day = 144 blocks
* Miner counts aggregated per chain-day
* Trend deltas computed day-over-day and across full window
* Optional compact sparkline per miner

Insights Provided:

* Miner momentum (rising / falling participation)
* Pool dominance changes
* Emerging miner activity
* Structural network shifts

Philosophy:
Blocks are the final truth layer of mining activity.
This feature surfaces what the network actually rewarded, not what was advertised
or estimated.

---

### **3.6 - On-Demand Hashrate Check (getnetworkhashps)**

Status: Implemented in 1.3.0

Add a keybind to trigger a lightweight popup
Popup displays:
    Current estimated hashrate (EH/s)
    Uses same logic: getnetworkhashps(144, current_height)

Intent:
Provides an immediate comparison:
Current state vs. sampled phase progression

Operator can quickly answer:

* Is hashrate rising since earlier phase samples?
* Is it dropping off mid-epoch?
* Is current value out of line with recent checkpoints?
* Add a keybind to trigger a lightweight popup
* Popup displays:
* Current estimated hashrate (EH/s)

Design:

* On-demand only (no constant polling)
* Uses same 144-block window for consistency
* Minimal UI:
  * small popup
  * no history, just “now”

Optional future extension (if it evolves):

* Color hint vs. last phase:
  * ↑ higher than last sample
  * ↓ lower than last sample
* Or simple delta:
  * +32 EH/s from last phase

---

### **3.7 - Economic Flow / Value Pressure Metrics**

* Explore rolling BTC-value metrics within mempool windows
  * Potential metrics:
    * Total BTC awaiting confirmation
    * BTC competing for next block
    * Median BTC per transaction
    * BTC distribution across fee-rate buckets
    * Rolling BTC cleared per block
  * Intended as observational/economic context rather than forensic analysis
  * Could correlate with hashrate breathing, miner distribution, and fee urgency
    windows

Observability Review (2026)

Status: Deferred / Likely Retired

Reason:
While the concept is interesting from an economic-observability perspective,
deriving meaningful BTC-value pressure metrics requires continuous inspection
of the raw mempool.

For large mempools this effectively becomes:

Repeated getrawmempool retrievals
Continuous transaction processing
Ongoing value aggregation
Frequent recomputation of bucket distributions

This workload is disproportionate to the value provided within a lightweight
terminal observatory.

BCI's design philosophy favors:

Low RPC impact
Fast refresh cycles
Broad node compatibility
Sustainable long-running observation

The proposed metrics are computationally expensive and derive from data that is
not directly exposed through lightweight RPC summaries.

Lesson learned:

The question was not:
"Can we calculate economic flow metrics?"

The answer is yes.

The question became:
"Can we calculate them efficiently enough to justify their place in a real-time
TUI observatory?"

Current answer:
Probably not.

This concept may be better suited for a dedicated daemon, research tool, or
offline analytics platform rather than BCI's live observatory model.

## **4. Longer-Term Concepts (Fantasy Stage)**

### **🌱 Node-to-Node Dashboard Sync**

Instances mirroring each other over LAN.
A sovereign “observer cluster.”

### **🌱 User-Defined Plugins**

Let advanced users add custom RPC queries to display.

### **🌱 Hash Phase Animation Pack**

Flip-dot inspired graphics, or more cinematic toggles.

---

## **5. Removed / Deprecated Ideas**

The original roadmap included explorations around:

* RBF manipulation logs
* mempool forensics
* miner censorship detection
* replacement storms
* priority fee comparisons

These are no longer aligned with the direction of BlockchainInfo.

The tool has matured into a **clarity instrument**, not a forensics engine.

---

### **3.8 - BIP Monitoring**

Status: Implemented in v1.4.0 as UASF Signals.

Future BCI feature: runtime BIP/User-Agent monitoring. Allow operator to set or
update a monitored token while BCI is running and display peer adoption
count/percent in a new panel within the Client/Version Distribution area.

---

### **Hashrate Distribution (HRD) / Estimated Pool Hashrate**

Status: Deferred / Likely Retired

Reason:
Miner share already represents observed block attribution.
Estimated pool hashrate would be derived from the same block-share data and
therefore does not provide an independent signal.

Example:
Network HR (On-Demand): 856 EH/s
Foundry Share: 25%

Operator can already infer:
856 × 0.25 ≈ 214 EH/s

This adds arithmetic, not observability.

BCI favors new signals over transformed versions of existing signals.

Lesson learned:
The question was not "How much estimated hashrate does a miner have?"
The question was "Has the miner's underlying hashrate changed?"

Block attribution alone cannot answer that.

---

## **6. Go-Forward Rules for All Features**

### **No clutter**

Every new toggle must earn its space.

### **No external dependencies**

RPC-only, trust-minimized.

### **No UI noise**

If it distracts from the signal, it doesn’t ship.

### **Always async, always smooth**

The UI must remain alive — never waiting on RPC.

---

## **7. The Spirit of BlockchainInfo**

This roadmap represents a tool built with care, precision, and intention —
designed to be the clearest window into Bitcoin’s heartbeat.
