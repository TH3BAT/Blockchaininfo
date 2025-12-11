
# **🚀 ROADMAP.md — 2025 Sovereign Edition**

*A modern, accurate roadmap for BlockchainInfo, reflecting the tool as it exists today.*

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

## **3. Upcoming Enhancements (v1.0 → v1.2)**

These are real ideas — not promises — designed to expand clarity without bloating the interface.

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
