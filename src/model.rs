use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub frequency: u64,
    pub usage: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RamInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub filesystem: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub mac_address: String,
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
}
