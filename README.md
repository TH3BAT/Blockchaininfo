
# 🌐 Blockchaininfo

A Real-Time Bitcoin Network Observatory — Built in Rust

![Rust][rust-badge] ![Uptime][uptime-badge]

[rust-badge]: https://img.shields.io/badge/Rust-1.70+-orange
[uptime-badge]: https://img.shields.io/badge/Uptime-60_days-brightgreen

![BlockchainInfo Avatar](https://image.nostr.build/98d63043b0980b9b5ffcb5c0aeb904a69e4054f432736f07b159411db669500f.jpg)

---

## Overview

**Blockchaininfo** is a high-performance, real-time terminal dashboard that reveals the **true heartbeat of the Bitcoin network**.
Built in **Rust** for safety and speed, it connects directly to your Bitcoin Knots/Core RPC and delivers:

* live blockchain metrics
* mempool analytics
* network decentralization health
* consensus security / fork monitoring
* transaction lookup
* and more

All packaged in a clean, color-coded TUI.

This is a tool built for those who care about **decentralization**, **sovereignty**, and the **integrity of Bitcoin’s proof-of-work network**.

---

## Why Blockchaininfo Exists

Bitcoin is decentralized — but **only if the network stays diverse and transparent**.

Blockchaininfo monitors:

* which versions are running
* how healthy mempool activity is
* how distributed the hash rate is
* whether forks appear
* how nodes behave across the network

It offers a **real-time lens** into the state of Bitcoin’s decentralization — something no block explorer or web UI can deliver with this immediacy, clarity, and local privacy.

---

## Key Features

### ⚡ **Real-Time Insights**

Every section updates independently using asynchronous tasks and global caches — ensuring smooth, flicker-free updates.

### 🧠 **Decentralization Monitoring**

Track node version diversity and client distribution to identify centralizing trends.

### 🔥 **Consensus Security**

Live fork monitoring displays active chain vs stale forks — with an automatic warning popup when a fork grows long.

### 🧩 **Mempool Distribution**

Custom mempool sampling logic (backed by semaphore concurrency + atomic dust filters) surfaces real-world fee pressure and distribution patterns.

### 🎛️ **Interactive Toggles**

Switch views instantly:

* Hashrate Distribution
* Last 20 Blocks / Miners
* Dust-Free mempool view
* Version vs Client distribution
* Propagation Times vs Averages
* Transaction lookup
* Help panel

### 🦀 **Rust-Powered Reliability**

* Memory-safe
* Panic-free
* Concurrency tuned
* 60+ day uptime proven

---

## File Structure

```plaintext
.
├── benches/
├── cargo.toml
├── miners.json
└── src/
    ├── config.rs
    ├── consensus/
    ├── display/
    ├── models/
    ├── rpc/
    ├── runapp.rs
    ├── ui/
    ├── utils.rs
    └── main.rs
```

Each folder has a clear role:

* **display/** → TUI rendering
* **rpc/** → Bitcoin RPC operations
* **models/** → local data structures (no external Bitcoin crates)
* **runapp.rs** → async orchestration & global cache system

---

## Configuration

The app supports multiple configuration paths, from zero-setup to hardened security.

### **1. Automatic Failsafe Mode (Zero Setup)**

If no config exists, Blockchaininfo prompts for RPC credentials and auto-creates:

``` Rust
./target/release/config.toml
```

### **2. config.toml File**

Preferred for custom setups.

```toml
[bitcoin_rpc]
username = "your_username"
password = "your_password"
address = "http://127.0.0.1:8332"
```

### **3. Environment Variables**

```bash
export RPC_USER="user"
export RPC_PASSWORD="password"
export RPC_ADDRESS="http://127.0.0.1:8332"
```

### **4. macOS Keychain Support**

Secure password retrieval:

```bash
security add-generic-password -a bitcoin -s rpc-password -w "your_password"
```

Optional: specify a custom entry path

```bash
export BCI_PASS_ENTRY=bitcoin-nasty/rpc-password
```

### **5. Linux Password Store (`pass`) Support**

Secure password retrieval using the standard Linux password manager.

### **Requirements**

* `pass` (Password Store)
* `gpg` with an initialized key
* a pinentry program (e.g. `pinentry-curses`)

**Initialize `pass` (one-time setup):**

```bash
pass init <GPG_KEY_ID>
```

**Store the Bitcoin RPC password:**

```bash
pass insert bitcoin/rpc-password
```

(Only the **first line** is used as the password; additional notes are ignored.)

Optional: specify a custom entry path

```bash
export BCI_PASS_ENTRY=bitcoin/rpc-password
```

BCI will automatically retrieve the RPC password from `pass` on Linux.

### **6. Optional SOCKS Proxy (Tor / Onion RPC)**

Blockchaininfo supports routing RPC traffic through a SOCKS proxy (e.g. Tor)
by setting an environment variable.
This enables access to RPC endpoints exposed over .onion services.

```bash
export BCI_RPC_PROXY="socks5h://127.0.0.1:9050"
```

### **Priority Order**

1. CLI flag (`--config`)
2. Env var (`BLOCKCHAININFO_CONFIG`)
3. Default path
4. Env variables (`RPC_*`)
5. Optional SOCKS proxy (`BCI_RPC_PROXY`)
6. macOS Keychain / Linux Password Store

---

## Installation

```bash
git clone https://github.com/TH3BAT/Blockchaininfo.git
cd Blockchaininfo
cargo build --release
```

---

## Usage

```bash
./target/release/blockchaininfo
```

Requires a running Bitcoin Knots/Core node with RPC enabled.

---

## Demo Video

A full demonstration video (`BlockChainInfoLiveDemo.mov`) is available in the Releases section.

Shows:

* Hash Phase flip
* Mempool distribution
* Node version/client charts
* Fork monitoring
* All toggles in action

---

## Error Handling

Blockchaininfo is built to survive:

* RPC timeouts
* invalid responses
* json parsing failures
* node restarts
* mempool storms

Errors are logged, not fatal.

---

## Contributions

PRs welcome!
Fork → branch → PR.

---

## License

MIT — do whatever your sovereign soul wants.
