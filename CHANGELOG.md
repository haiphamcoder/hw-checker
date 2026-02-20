# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-02-20

### Added

- **Deep Hardware Discovery**:
  - CPU: Physical cores, frequency, usage, and L1/L2/L3 cache capacities via `raw-cpuid`.
  - RAM: Multi-DIMM detection with Manufacturer (mapped from JEDEC IDs), Part Number, Serial Number, and Speed using `smbios-lib`.
  - Storage: Mount points, total/used space, interface type (NVMe/SATA/SAS), Model, and Serial Number.
  - Motherboard & BIOS: Full DMI/SMBIOS information (Vendor, Product, BIOS version/date).
  - PCI: Comprehensive device discovery mapping raw hex IDs to human-readable vendor and device names via `pci.ids` database.
  - USB: Full enumeration of connected devices with Vendor/Product IDs.
  - Network: Interface MAC addresses and data transferred metrics.
  - System Health: Core metrics tracking including Motherboard and Battery details.
- **Reporting & Interface**:
  - `hw-checker --full` (or `--all`) flag for an all-in-one comprehensive hardware diagnostics report.
  - Advanced filtering tools (`--cpu`, `--ram`, `--storage`, `--network`, `--usb`, `--pci`, `--health`) for isolated output.
  - Automated detection and graceful fallback if run without `sudo`/root privileges for modules like RAM.
  - Output formats: Support for `json`, `yaml`, and professional terminal `table` with semantic color coding.
- **Customization**:
  - Threshold definitions in YAML to trigger warnings and critical status indicators across the system.

### Changed

- Standardized the core metrics base across Windows, macOS, and Linux with "deep discovery" features heavily optimized for Linux.

### Fixed

- Fixed issues parsing RAM speed from SMBIOS raw data.
- Addressed `raw-cpuid` struct updates for cross-generation compatibility.
