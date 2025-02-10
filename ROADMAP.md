# **ROADMAP Update**  

🔥 **Arcade-Style Mempool Machine**  

**🏆 Welcome to the Next Level!**  
Our application isn’t just another mempool tracker—it’s **a high-speed, dust-free, signal-strong arcade machine** for analyzing Bitcoin’s transaction flow. **No noise. No wasted cycles. Just precision.**  

🕹️ **Think of it like an old-school arcade game:**  

- We **avoid lag and noise** by cutting out dust transactions.  
- We **maximize efficiency** by caching key data.  
- We **keep the action smooth** by ensuring real-time updates.  

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

---

### **🔜 Next Levels on the Arcade Roadmap**  

🎯 **Optimizing Block Change Handling:**  

- Smoother transaction aging to prevent delays after a block change.  

🚀 **Further Performance Enhancements:**  

- Improving caching efficiency to reduce real memory usage even further.  

💡 **Additional User Customization:**  

- Future potential: Letting users tweak filtering thresholds for personal insights.  

**🏆 This is just the beginning!** Stay tuned as we continue to **level up** this mempool machine!  
