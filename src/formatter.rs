use crate::config::{Config, Thresholds};
use crate::model::{BatteryInfo, HardwareReport, MotherboardInfo, PciDevice, UsbDevice};
use colored::Colorize;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};

pub fn print_report(report: &HardwareReport, config: &Config) {
    print_summary(report);
    print_cpu(&report.cpu, &config.cpu_thresholds);
    print_ram(&report.ram, &config.ram_thresholds);
    print_storage(&report.storage, &config.storage_thresholds);
    print_network(&report.network);
    print_usb(&report.usb);
    print_pci(&report.pci);
    print_health(report.motherboard.as_ref(), &report.battery);
}

pub fn print_summary(report: &HardwareReport) {
    println!("\n{}", "System Summary".bold().cyan());
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Hostname", "OS", "Kernel", "Uptime"]);

    table.add_row(vec![
        Cell::new(&report.hostname).fg(Color::Magenta),
        Cell::new(format!("{} {}", report.os_name, report.os_version)),
        Cell::new(&report.kernel_version),
        Cell::new(format_uptime(report.uptime)),
    ]);
    println!("{table}");
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / (24 * 3600);
    let hours = (seconds % (24 * 3600)) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn print_cpu(cpus: &[crate::model::CpuInfo], thresholds: &Thresholds) {
    println!("\n{}", "CPU Information".bold().cyan());
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Core",
            "Model",
            "Vendor",
            "Frequency (MHz)",
            "Usage (%)",
        ]);

    for (i, cpu) in cpus.iter().enumerate() {
        let usage_color = if cpu.usage > thresholds.critical {
            Color::Red
        } else if cpu.usage > thresholds.warning {
            Color::Yellow
        } else {
            Color::Green
        };

        table.add_row(vec![
            Cell::new(i.to_string()),
            Cell::new(&cpu.model),
            Cell::new(&cpu.vendor_id),
            Cell::new(cpu.frequency.to_string()),
            Cell::new(format!("{:.1}", cpu.usage)).fg(usage_color),
        ]);
    }
    println!("{table}");
}

pub fn print_ram(ram: &crate::model::RamInfo, thresholds: &Thresholds) {
    println!("\n{}", "RAM Information & Usage".bold().cyan());
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Component",
            "Total (MiB)",
            "Used (MiB)",
            "Free (MiB)",
            "Usage (%)",
        ]);

    let ram_usage = (ram.used as f32 / ram.total as f32) * 100.0;
    let ram_color = if ram_usage > thresholds.critical {
        Color::Red
    } else if ram_usage > thresholds.warning {
        Color::Yellow
    } else {
        Color::Green
    };

    table.add_row(vec![
        Cell::new("Main Memory"),
        Cell::new((ram.total / 1024 / 1024).to_string()),
        Cell::new((ram.used / 1024 / 1024).to_string()),
        Cell::new((ram.free / 1024 / 1024).to_string()),
        Cell::new(format!("{:.1}", ram_usage)).fg(ram_color),
    ]);

    let swap_usage = if ram.swap_total > 0 {
        (ram.swap_used as f32 / ram.swap_total as f32) * 100.0
    } else {
        0.0
    };
    table.add_row(vec![
        Cell::new("Swap"),
        Cell::new((ram.swap_total / 1024 / 1024).to_string()),
        Cell::new((ram.swap_used / 1024 / 1024).to_string()),
        Cell::new(((ram.swap_total - ram.swap_used) / 1024 / 1024).to_string()),
        Cell::new(format!("{:.1}", swap_usage)),
    ]);

    println!("{table}");

    if !ram.sticks.is_empty() {
        println!("\n{}", "Physical RAM Sticks".bold().cyan());
        let mut stick_table = Table::new();
        stick_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                "Slot",
                "Manufacturer",
                "Part Number",
                "Serial Number",
                "Speed (MT/s)",
            ]);

        for (i, stick) in ram.sticks.iter().enumerate() {
            stick_table.add_row(vec![
                Cell::new(format!("DIMM {}", i)),
                Cell::new(stick.manufacturer.as_deref().unwrap_or("Unknown")),
                Cell::new(stick.part_number.as_deref().unwrap_or("Unknown")),
                Cell::new(stick.serial_number.as_deref().unwrap_or("Unknown")),
                Cell::new(
                    stick
                        .speed
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                ),
            ]);
        }
        println!("{stick_table}");
    } else {
        println!(
            "\n{}",
            "Note: Run with sudo to see physical RAM stick details."
                .yellow()
                .italic()
        );
    }
}

pub fn print_storage(storage: &[crate::model::StorageInfo], thresholds: &Thresholds) {
    println!("\n{}", "Storage Information".bold().cyan());
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Name",
            "Mount",
            "FS",
            "Type",
            "Total (GiB)",
            "Used (GiB)",
            "Usage (%)",
        ]);

    for disk in storage {
        let usage_pct = (disk.used as f32 / disk.total as f32) * 100.0;
        let color = if usage_pct > thresholds.critical {
            Color::Red
        } else if usage_pct > thresholds.warning {
            Color::Yellow
        } else {
            Color::Green
        };

        table.add_row(vec![
            Cell::new(&disk.name),
            Cell::new(&disk.mount_point),
            Cell::new(&disk.filesystem),
            Cell::new(disk.disk_type.as_deref().unwrap_or("Unknown")),
            Cell::new(format!(
                "{:.1}",
                disk.total as f64 / 1024.0 / 1024.0 / 1024.0
            )),
            Cell::new(format!(
                "{:.1}",
                disk.used as f64 / 1024.0 / 1024.0 / 1024.0
            )),
            Cell::new(format!("{:.1}", usage_pct)).fg(color),
        ]);
    }
    println!("{table}");
}

pub fn print_network(network: &[crate::model::NetworkInfo]) {
    println!("\n{}", "Network Interfaces".bold().cyan());
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            "Interface",
            "MAC",
            "Received (MiB)",
            "Transmitted (MiB)",
        ]);

    for net in network {
        table.add_row(vec![
            Cell::new(&net.name),
            Cell::new(&net.mac_address),
            Cell::new(format!("{:.2}", net.received as f64 / 1024.0 / 1024.0)),
            Cell::new(format!("{:.2}", net.transmitted as f64 / 1024.0 / 1024.0)),
        ]);
    }
    println!("{table}");
}

pub fn print_usb(usb: &[UsbDevice]) {
    if usb.is_empty() {
        return;
    }
    println!("\n{}", "USB Devices".bold().cyan());
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Bus/Addr", "ID", "Manufacturer", "Product"]);

    for dev in usb {
        table.add_row(vec![
            Cell::new(format!("{:03}/{:03}", dev.bus, dev.address)),
            Cell::new(format!("{:04x}:{:04x}", dev.vendor_id, dev.product_id)),
            Cell::new(dev.manufacturer.as_deref().unwrap_or("Unknown")),
            Cell::new(dev.product.as_deref().unwrap_or("Unknown")),
        ]);
    }
    println!("{table}");
}

pub fn print_pci(pci: &[PciDevice]) {
    if pci.is_empty() {
        return;
    }
    println!("\n{}", "PCI Devices".bold().cyan());
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Slot", "ID", "Name/Class"]);

    for dev in pci {
        table.add_row(vec![
            Cell::new(&dev.slot),
            Cell::new(format!("{:04x}:{:04x}", dev.vendor_id, dev.device_id)),
            Cell::new(dev.device_name.as_deref().unwrap_or("Unknown Device")),
        ]);
    }
    println!("{table}");
}

pub fn print_health(motherboard: Option<&MotherboardInfo>, battery: &[BatteryInfo]) {
    if let Some(mb) = motherboard {
        println!("\n{}", "Motherboard & BIOS".bold().cyan());
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["Component", "Information"]);

        table.add_row(vec![Cell::new("Manufacturer"), Cell::new(&mb.vendor)]);
        table.add_row(vec![Cell::new("Product"), Cell::new(&mb.product)]);
        table.add_row(vec![Cell::new("BIOS Vendor"), Cell::new(&mb.bios_vendor)]);
        table.add_row(vec![Cell::new("BIOS Version"), Cell::new(&mb.bios_version)]);
        table.add_row(vec![Cell::new("BIOS Date"), Cell::new(&mb.bios_date)]);
        println!("{table}");
    }

    if !battery.is_empty() {
        println!("\n{}", "Battery Status".bold().cyan());
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["Battery", "Status", "Capacity (%)"]);

        for bat in battery {
            let color = if bat.capacity < 20 {
                Color::Red
            } else if bat.capacity < 50 {
                Color::Yellow
            } else {
                Color::Green
            };

            table.add_row(vec![
                Cell::new(&bat.name),
                Cell::new(&bat.status),
                Cell::new(bat.capacity.to_string()).fg(color),
            ]);
        }
        println!("{table}");
    }
}
