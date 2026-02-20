# hwchecker 🚀

A fast, lightweight, and cross-platform hardware information CLI tool written in Rust.

`hwchecker` provides real-time insights into your system's hardware, including CPU, RAM, Storage, and Network interfaces, with a professional and customizable terminal interface.

## ✨ Features

- **📊 Comprehensive Discovery**:
  - **CPU**: Model, physical cores, frequency, and real-time usage.
  - **RAM**: Main memory and Swap usage analysis.
  - **Storage**: Mount points, total/used space, and filesystem types.
  - **Network**: Interface MAC addresses and total data transferred.
  - **System Summary**: OS version, Kernel version, Hostname, and Uptime.
- **🎨 Visual Excellence**: Professional terminal tables with semantic color coding (Green/Yellow/Red) based on usage thresholds.
- **🛠️ Highly Customizable**: Define your own warning and critical thresholds via a YAML configuration file.
- **📁 Multiple Output Formats**: Supports `Table` (default), `JSON`, and `YAML`.
- **🔍 Filtering**: Display only the components you care about using CLI flags.
- **🌐 Cross-platform**: Supports Linux, macOS (Intel/ARM), and Windows.

## 🚀 Quick Start

### Installation

Ensure you have Rust and Cargo installed. Clone the repository and build:

```bash
git clone https://github.com/yourusername/hwchecker.git
cd hwchecker
cargo build --release
```

The binary will be available at `./target/release/hw-checker`.

### Usage

Run the tool to see a full system report:

```bash
cargo run -- --format table
```

#### Filtering Components

```bash
# Show only CPU and RAM
cargo run -- --cpu --ram
```

#### Exporting Data

```bash
# Export as JSON
cargo run -- --format json > report.json
```

#### Using Custom Thresholds

Create a `config.yaml` file:

```yaml
cpu_thresholds:
  warning: 80.0
  critical: 95.0
ram_thresholds:
  warning: 75.0
  critical: 90.0
```

Run with the config:

```bash
cargo run -- --config config.yaml
```

## 🛠️ Tech Stack

- **[clap](https://crates.io/crates/clap)**: Command line argument parsing.
- **[sysinfo](https://crates.io/crates/sysinfo)**: System information retrieval.
- **[comfy-table](https://crates.io/crates/comfy-table)**: Beautiful terminal display.
- **[colored](https://crates.io/crates/colored)**: Terminal string styling.
- **[serde](https://crates.io/crates/serde)**: Serialization/Deserialization.

## 🤖 CI/CD

This project uses **GitHub Actions** to automatically build and package binaries for multiple operating systems on every release tag.

## 📄 License

This project is open-source. See the [LICENSE](LICENSE) file for details.

---
*Built with ❤️ using Rust.*
