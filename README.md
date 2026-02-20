# hwchecker 🚀

A fast, lightweight, and cross-platform hardware information CLI tool written in Rust.

`hwchecker` provides real-time insights into your system's hardware, system health, and peripheral devices with a professional terminal interface.

## ✨ Features (v2.1)

- **📊 Comprehensive Discovery**:
  - **CPU**: Model, physical cores, frequency, usage, and Vendor/Brand info.
  - **RAM**: Main memory and Swap usage analysis.
  - **Storage**: Mount points, total/used space, **Model Name**, and **Serial Number**.
  - **Motherboard & BIOS**: Manufacturer, Product, and BIOS version/date.
  - **Battery**: Real-time status and capacity tracking.
  - **Network**: Interface MAC addresses and total data transferred.
  - **USB & PCI**: Full enumeration of connected devices with Vendor/Product IDs.
  - **System Summary**: OS version, Kernel version, Hostname, and Uptime.
- **🎨 Visual Excellence**: Professional terminal tables with semantic color coding (Green/Yellow/Red).
- **🛠️ Customizable Thresholds**: Define your own warning and critical limits via YAML.
- **🔍 Advanced Filtering**: Isolated views with `--cpu`, `--ram`, `--storage`, `--network`, `--usb`, `--pci`, and `--health`.
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
# Full detailed report
cargo run -- --health --cpu --ram --storage --network --usb --pci

# System health only (Motherboard & Battery)
cargo run -- --health

# Export everything to JSON
cargo run -- --format json > report.json
```

## 🛠️ Tech Stack

- **sysinfo**: System metrics core.
- **rusb**: USB discovery.
- **pci-info**: PCI enumeration.
- **raw-cpuid**: X86 CPU features.
- **comfy-table**: Professional UI tables.

## 📄 License

MIT / Open Source.
