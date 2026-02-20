use crate::cli::OutputFormat;
use crate::model::HardwareReport;
use anyhow::Result;

pub fn export_report(report: &HardwareReport, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(report)?;
            println!("{json}");
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(report)?;
            println!("{yaml}");
        }
        OutputFormat::Table => unreachable!("Table format should be handled by formatter"),
    }
    Ok(())
}
