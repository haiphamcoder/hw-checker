# hwchecker 🚀

A fast, lightweight, and cross-platform hardware information CLI tool written in Rust.

`hwchecker` provides real-time insights into your system's hardware, system health, and peripheral devices with a professional terminal interface.

## ✨ Features (v1.0.0)

- **📊 Deep Hardware Discovery**:
  - **CPU**: Model, physical cores, frequency, usage, and **L1/L2/L3 Cache** details.
  - **RAM**: Main memory, Swap usage, and **Multi-DIMM details** (Manufacturer, SN, Part No, Speed) using SMBIOS.
  - **Storage**: Mount points, total/used space, **Interface (NVMe/SATA)**, Model, and Serial Number.
  - **Motherboard & BIOS**: Full DMI/SMBIOS information (Vendor, Product, BIOS version/date).
  - **PCI & USB**: Comprehensive device discovery with **mapped human-readable names** (via pci.ids).
  - **Network**: Interface MAC addresses and total data transferred.
  - **System Summary**: OS version, Kernel version, Hostname, and Uptime.
- **🎨 Visual Excellence**: Professional terminal tables with semantic color coding (Green/Yellow/Red).
- **🛠️ Customizable Thresholds**: Define your own warning and critical limits via YAML.
- **🔍 Advanced Filtering**: Isolated views with `--cpu`, `--ram`, `--storage`, `--network`, `--usb`, `--pci`, and `--health`.
- **🚀 All-in-one Report**: Use `--full` (or `--all`) for a complete hardware diagnostics report.
- **🌐 Cross-platform**: Core metrics work on Linux, macOS, and Windows. Deep metadata prioritized for Linux.

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/haiphamcoder/hw-checker.git
cd hw-checker
cargo build --release
```

### Usage

```bash
# Get "full" hardware information (Linux: recommended with sudo for RAM/DMI details)
sudo ./target/release/hw-checker --full

# Specific module discovery
./target/release/hw-checker --cpu
./target/release/hw-checker --ram
./target/release/hw-checker --pci

# Export everything to JSON
./target/release/hw-checker --full --format json > report.json
```

## 🛠️ Tech Stack

- **sysinfo**: System metrics core.
- **smbios-lib**: SMBIOS/DMI table parsing.
- **rusb**: USB discovery.
- **pci-info**: PCI enumeration core.
- **raw-cpuid**: High-fidelity CPU feature discovery.
- **comfy-table**: Professional UI tables.

## 📄 License

MIT / Open Source.
