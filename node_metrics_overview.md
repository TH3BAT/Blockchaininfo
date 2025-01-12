
# Bitcoin Node Metrics Overview

This document provides an overview of the metrics tracked by the **Blockchaininfo** application. Each section explains the significance of the data and why it matters for decentralization, security, and network health.

---

## [Blockchain Metrics]

### Chain Details

- **Chain**: Indicates the active chain being monitored (e.g., `main` for mainnet).
- **Best Block**: The most recent block in the blockchain.
- **Time Since Block**: Tracks how recently the last block was mined, ensuring consistent block discovery.

### Difficulty and Adjustments

- **Difficulty**: Represents the network's mining difficulty, ensuring block times remain consistent.
- **Blocks Until Adjustment**: Tracks how many blocks are left before the next difficulty adjustment.
- **Estimated Change**: Predicts whether difficulty will increase or decrease based on current mining activity.

### Chainwork and Verification

- **Chainwork**: A measure of the cumulative work done to create the chain, reflecting its security.
- **Verification Progress**: Shows the percentage of blocks verified, crucial for node synchronization.

### Disk Usage and Median Time

- **Size on Disk**: The total storage used by the blockchain data.
- **Median Time**: The median time of the last 11 blocks, used in consensus mechanisms.
- **Block Time**: The timestamp of the most recent block, offering a snapshot of network activity.

---

## [Mempool Metrics]

### Transaction Activity

- **Transactions**: The total number of unconfirmed transactions in the mempool.
- **Memory**: The memory used by the mempool compared to its configured limit.
- **Total Fees**: The cumulative fees for all transactions in the mempool.
- **Min Transaction Fee**: The minimum fee rate required to get a transaction accepted into the mempool.

*Why It Matters*: Mempool activity reflects network demand, congestion, and fee market dynamics.

---

## [Network Metrics]

### Peer Connectivity

- **Connections In**: The number of peers connected to the node as inbound connections.
- **Connections Out**: The number of peers connected as outbound connections.

### Bandwidth Monitoring

- **Total Bytes Received**: Tracks data received by the node, indicating network load.
- **Total Bytes Sent**: Tracks data sent by the node, reflecting propagation efficiency.

### Node Version Distribution

- Displays the software versions of connected peers and their counts. This reveals diversity in node versions, critical for decentralization.
  
  ```yaml
  Node Version Distribution:
    - 27.1.0: 9 peers 
    - 28.0.0: 8 peers 
    - 26.0.0: 7 peers
    - ...
  ```

*Why It Matters*: Monitoring bandwidth ensures efficient block propagation, while version diversity minimizes vulnerabilities to coordinated attacks.

---

## [Consensus Security Metrics]

### Fork Monitoring

- **Height**: The block height of chains or forks being monitored.
- **Status**: Indicates whether the chain is the active chain or a stale fork.
- **Branch Length**: The length of the branch from the main chain.

*Why It Matters*: Identifying forks helps ensure the node is on the correct chain and detects potential consensus issues.

### Block Propagation Time

- Measures how quickly new blocks propagate across the network, reflecting decentralization and security.
