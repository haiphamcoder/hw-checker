# hwchecker 🚀

A fast, lightweight, and cross-platform hardware information CLI tool written in Rust.

`hwchecker` provides real-time insights into your system's hardware, including CPU, RAM, Storage, and Network interfaces, with a professional and customizable terminal interface.

## ✨ Features (v2.0)

- **📊 Comprehensive Discovery**:
  - **CPU**: Model, physical cores, frequency, usage, and **Vendor/Brand** info.
  - **RAM**: Main memory and Swap usage analysis.
  - **Storage**: Mount points, total/used space, filesystem types, and **Disk Kind** (SSD/HDD).
  - **Network**: Interface MAC addresses and total data transferred.
  - **USB & PCI**: Full enumeration of connected Bus/Slot devices with Vendor/Product IDs.
  - **System Summary**: OS version, Kernel version, Hostname, and Uptime.
- **🎨 Visual Excellence**: Professional terminal tables with semantic color coding (Green/Yellow/Red) based on usage thresholds.
- **🛠️ Highly Customizable**: Define your own warning and critical thresholds via a YAML configuration file.
- **📁 Multiple Output Formats**: Supports `Table` (default), `JSON`, and `YAML`.
- **🔍 Filtering**: Display only the components you care about: `--cpu`, `--ram`, `--storage`, `--network`, `--usb`, `--pci`.
- **🌐 Cross-platform**: Supports Linux, macOS, and Windows.

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/haiphamcoder/hw-checker.git
cd hw-checker
cargo build --release
```

### Usage

```bash
# Full report
cargo run -- --format table

# USB devices only
cargo run -- --usb

# Export to JSON
cargo run -- --format json > report.json
```

## 🛠️ Tech Stack

- **[sysinfo](https://crates.io/crates/sysinfo)**: System metrics.
- **[rusb](https://crates.io/crates/rusb)**: USB discovery.
- **[pci-info](https://crates.io/crates/pci-info)**: PCI enumeration.
- **[raw-cpuid](https://crates.io/crates/raw-cpuid)**: X86 CPU features.
- **[comfy-table](https://crates.io/crates/comfy-table)**: UI layout.

## 🤖 CI/CD

Automatic cross-platform binaries are generated on every tag push via GitHub Actions.

## 📄 License

MIT / Open Source.
