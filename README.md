# 🔭 BlockChainInfo (BCI)

## A Bitcoin Network Observatory

**Measure. Verify. Understand.**

![Rust][rust-badge]

[rust-badge]: https://img.shields.io/badge/Rust-1.70+-orange

![BCI Observatory Logo](assets/BCI_Logo.png)

---

## Overview

**BlockChainInfo (BCI)** is a real-time Bitcoin network observatory built in Rust.

BCI connects directly to your Bitcoin Core or Knots node and transforms raw
network data into a live observatory of Bitcoin's blockchain, mempool, network,
and consensus activity.

Unlike web explorers, BCI operates locally against your own node, providing
immediate visibility into network conditions without relying on third-party services.

BCI doesn't shout. It endures.

The longer you run it, the more you discover.

---

## Why BCI Exists

Bitcoin's strength comes from decentralization, transparency, and verifiable consensus.

BCI exists to help operators observe those properties directly.

Rather than focusing on price, speculation, or market noise, BCI focuses on the
health and behavior of the network itself.

BCI allows operators to observe:

* Blockchain activity
* Mempool conditions
* Network client diversity
* Consensus security
* Miner participation
* Transaction propagation
* UASF signalling
* Difficulty cycle behavior

All from a single terminal window.

---

## Observatory Instruments

### ⛓️ Blockchain Observatory

Monitor:

* Best block
* Difficulty
* Estimated network hashrate
* Difficulty adjustment progress
* Verification progress
* Chainwork
* Disk usage
* Median time
* Block timestamps

### ⚡ Hashphase Monitoring

Track the current difficulty epoch through:

* 10%
* 25%
* 50%
* 75%
* 100%

phase checkpoints while observing estimated network hashrate changes throughout
the cycle.

### 📦 Mempool Observatory

Observe:

* Transaction count
* Memory usage
* Fee pressure
* Transaction size distribution
* Dust-free transaction views
* Fee-rate statistics
* Transaction age distribution

### 🌎 Network Observatory

Monitor:

* Connected peers
* Client diversity
* Version distribution
* UASF signals
* Propagation timing
* Data transfer statistics

### ⛏️ Miner Observatory

Track:

* Last 20 blocks
* Daily miner activity
* Weekly miner activity
* Miner participation trends
* Observed mining entities

### 🛡️ Consensus Observatory

Observe:

* Active chain status
* Stale forks
* Fork lengths
* Consensus divergence signals
* Automatic fork warning notifications

---

## Design Philosophy

BCI is intentionally conservative.

The goal is not to create another dashboard.

The goal is to create an instrument panel that operators can leave running for
weeks or months at a time.

Features are added slowly.

Interface changes are minimized.

Observations are prioritized over interpretations.

The observatory remains familiar release after release.

---

## Key Characteristics

* Built in Rust
* Works with Bitcoin Core and Bitcoin Knots
* Supports pruned nodes
* Local-first architecture
* Real-time updates
* Memory efficient
* Long-running observatory design
* Terminal-based interface
* No external services required

---

## Installation

```bash
git clone https://github.com/TH3BAT/Blockchaininfo.git
cd Blockchaininfo
cargo build --release
```

## Launch

```bash
./target/release/blockchaininfo
```

Requires a running Bitcoin Core or Knots node with RPC enabled.

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
