use anyhow::Result;
use clap::Parser;
use hw_checker::cli::{Args, OutputFormat};
use hw_checker::config::Config;
use hw_checker::discovery::get_hardware_report;
use hw_checker::exporter::export_report;
use hw_checker::formatter::{
    print_cpu, print_health, print_network, print_pci, print_ram, print_report, print_storage,
    print_usb,
};

fn main() -> Result<()> {
    let args = Args::parse();

    let config = if let Some(path) = args.config {
        Config::load_from_file(path)?
    } else {
        Config::default()
    };

    let report = get_hardware_report();

    if args.format == OutputFormat::Table {
        let any_filter = args.cpu
            || args.ram
            || args.storage
            || args.network
            || args.usb
            || args.pci
            || args.health
            || args.full;

        if any_filter {
            if args.cpu || args.full {
                print_cpu(&report.cpu, &config.cpu_thresholds);
            }
            if args.ram || args.full {
                print_ram(&report.ram, &config.ram_thresholds);
            }
            if args.storage || args.full {
                print_storage(&report.storage, &config.storage_thresholds);
            }
            if args.network || args.full {
                print_network(&report.network);
            }
            if args.usb || args.full {
                print_usb(&report.usb);
            }
            if args.pci || args.full {
                print_pci(&report.pci);
            }
            if args.health || args.full {
                print_health(report.motherboard.as_ref(), &report.battery);
            }
        } else {
            print_report(&report, &config);
        }
    } else {
        export_report(&report, args.format)?;
    }

    Ok(())
}
