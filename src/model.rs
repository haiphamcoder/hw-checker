use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub vendor_id: String,
    pub brand: String,
    pub cores: usize,
    pub frequency: u64,
    pub usage: f32,
    pub l1_cache: Option<String>,
    pub l2_cache: Option<String>,
    pub l3_cache: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RamStick {
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub serial_number: Option<String>,
    pub speed: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RamInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub sticks: Vec<RamStick>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub filesystem: String,
    pub vendor: Option<String>,
    pub model_name: Option<String>,
    pub serial_number: Option<String>,
    pub disk_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub mac_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsbDevice {
    pub bus: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PciDevice {
    pub slot: String,
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: Option<String>,
    pub device_name: Option<String>,
    pub class_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MotherboardInfo {
    pub vendor: String,
    pub product: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub bios_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatteryInfo {
    pub name: String,
    pub status: String,
    pub capacity: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareReport {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub uptime: u64,
    pub cpu: Vec<CpuInfo>,
    pub ram: RamInfo,
    pub storage: Vec<StorageInfo>,
    pub network: Vec<NetworkInfo>,
    pub usb: Vec<UsbDevice>,
    pub pci: Vec<PciDevice>,
    pub motherboard: Option<MotherboardInfo>,
    pub battery: Vec<BatteryInfo>,
}
