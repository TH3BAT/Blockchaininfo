# **ROADMAP.md**  

## **Network Health Surveillance: RBF & Mempool Integrity**  

**🔍 Our Mission**  
Unlike generic mempool trackers, we monitor **Bitcoin’s security and decentralization in real-time**—exposing stealth RBF, miner frontrunning, and mempool anomalies that others miss.  

**🛡️ How We Differ From mempool.space**  

| Feature | mempool.space | **Our Tool** |  
|---------|--------------|-------------|  
| **RBF Monitoring** | Basic opt-in RBF tracking | **Stealth RBF detection, miner replacement logs** |  
| **Fee Analysis** | Priority fee estimates | **Dust-free, CPFP/RBF-aware fee signals** |  
| **Focus** | General mempool visualization | **Network health & attack detection** |  
| **Data Depth** | Standard mempool filtering | **Forensic-grade replacement logging** |  

---

## **Core Features**  

### **🚨 RBF Attack Radar (Unique to Us)**  

- Logs **every `getmempoolentry` error** to detect:  
  - ✅ **Silent RBF** (replacements without opt-in flag)  
  - ✅ **Miner frontrunning** (high-fee tx kicking out low-fee)  
  - ✅ **Mempool censorship** (transactions mysteriously vanishing)  
- **Friday congestion analysis** → catch weekly replacement spikes.  

### **📊 Clean Fee Analytics (vs. mempool.space)**  

- **Excludes dust** (≤546 sats) for accurate fee signals.  
- **Ancestor/descendant-aware** → reflects real mining incentives.  
- **Modified fee tracking** → shows miner-adjusted economics.  

### **⚡ Performance Optimizations**  

- **Cached critical data** → no lag, no bloat.  
- **Real-time updates** → no stale mempool views.  

---

## **🔜 Next Steps (Roadmap)**  

### **🛠️ Phase 1: RBF Alert System (Priority)**  

- Live alerts for **non-compliant BIP 125 replacements**.  
- Public log of **worst offender miners/pools**.  

### **📈 Phase 2: Network Health Dashboard**  

- **"RBF Storm" tracker** (replacement heatmaps by hour/day).  
- **0-conf risk scoring** for merchants/exchanges.  

### **🔍 Phase 3: Community Forensics**  

- **User-submitted tx investigations** → cross-check our logs.  
- **Weekly reports** on mempool manipulation trends.  

---

## **Why This Matters**  

We’re not just another fee tracker—we’re **Bitcoin’s network watchdog**. While others show the mempool, we expose **its hidden battles**.  

**Stay tuned. Stay paranoid.** ⚡  

👇 **Here's how our system stacks up against other mempool trackers…**  

---

## **🎯 Understanding the Fee Rate Differences**  

Our mempool distribution provides a **clean, dust-free signal** of transaction activity, while platforms like **mempool.info** calculate priority fees differently.  

### **Key Differences:**  

🚫 **Dust-Free Transactions:**  

- We **exclude dust transactions** (546 sats or less), which often sit in the mempool for **hours or days** without getting mined.  
- This prevents **low-fee TXs from skewing the average fee rate downward.**  

📊 **Comprehensive Fee Calculation:**  

- Our approach considers **all relevant fee metrics**:  
  ✅ **Base Fee** (what the TX pays directly)  
  ✅ **Ancestor & Descendant Fees** (impact of CPFP & RBF strategies)  
  ✅ **Modified Fees** (miner-adjusted incentives)  
- This provides a **more accurate representation** of what transactions actually pay.  

⚡ **Mempool.info’s Priority Fee vs. Our Average Fee Rate:**  

- **Mempool.info estimates the *lowest* fee rate** needed to get into the next block.  
- **Our average fee rate (e.g., 7.65 sat/vB)** shows the **actual fees being paid across active transactions**, free from dust and noise.  

💡 **Applying These Insights:**  
✅ **Need the minimum fee to be mined soon?** → Refer to **mempool.info’s priority fees.**  
✅ **Want to understand real transaction behavior?** → Use our **mempool distribution data.**  

By keeping **dust where it belongs** and ensuring **pure signal**, our mempool insights provide a **more reliable view of network conditions.**  
