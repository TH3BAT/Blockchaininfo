# ROADMAP

This document outlines planned features and improvements for the BlockchainInfo Rust application. These features are designed to enhance functionality, scalability, and user experience.

---

## Planned Features

### **1. Block-Based Alarm Clock**

- **Description**: Add an optional feature to alert users when a specified number of blocks have been mined beyond the current blockchain height.
- **Key Details**:
  - Enable with a CLI argument: `-a` followed by the block offset and an optional file path for an MP3 file to play (e.g., `-a18 some_mp3_file_name_and_path`).
  - Application will poll the blockchain RPC endpoint at regular intervals.
  - Trigger an audible alarm using the specified MP3 file or a default sound when the target block is reached.
  - Users can turn off the alarm interactively using any key.
- **Future Enhancements**:
  - Transition to a configuration file for setting default MP3 paths, snooze durations, and other preferences.
  - Add snooze functionality with adjustable time intervals.
  - Integrate volume control and additional alarm types (visual alerts, notifications).
- **Implementation Notes**:
  - Use async runtime for background polling.
  - Integrate terminal interactivity with `crossterm`.
  - Use `rodio` for MP3 playback.

---

### **2. Enhanced Modularization**

- **Description**: Continue refining code organization for scalability and readability.
- **Key Updates**:
  - Further modularize existing namespaces like `rpc.rs` and `display.rs`.
  - Create specialized submodules for repetitive tasks.

---

### **3. Configurable Options**

- **Description**: Provide greater flexibility through a configuration file.
- **Planned Settings**:
  - Default polling interval for RPC queries.
  - Alarm preferences (e.g., sound type, volume, or visual alerts).
  - Application themes for display customization.

---

### **4. Web UI Integration**

- **Description**: Build a lightweight web interface for real-time monitoring.
- **Features**:
  - View blockchain information via a browser.
  - Enable and configure alarms directly through the UI.
  - Display graphical data like block confirmation progress.

---

### **5. Multi-Language Support**

- **Description**: Translate the application output and CLI help into multiple languages for global accessibility.

---

### **Contributing**

We welcome contributions! If you have ideas or want to help with implementation, please check out our contribution guidelines.

---

Stay tuned for updates as we continue building features that align with the decentralized ethos and enhance user experience!
