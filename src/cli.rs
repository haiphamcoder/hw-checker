use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Show only CPU info
    #[arg(long)]
    pub cpu: bool,

    /// Show only RAM info
    #[arg(long)]
    pub ram: bool,

    /// Show only Storage info
    #[arg(long)]
    pub storage: bool,

    /// Show only Network info
    #[arg(long)]
    pub network: bool,

    /// Show only USB devices
    #[arg(long)]
    pub usb: bool,

    /// Show only PCI devices
    #[arg(long)]
    pub pci: bool,

    /// Show System Health (Motherboard, BIOS, Battery)
    #[arg(long)]
    pub health: bool,

    /// Path to configuration file (YAML)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Show full information (all filters enabled)
    #[arg(long, aliases = ["all"])]
    pub full: bool,

    /// Launch interactive Terminal User Interface (TUI)
    #[arg(long)]
    pub tui: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}
