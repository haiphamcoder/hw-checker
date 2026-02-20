use crate::model::{CpuInfo, HardwareReport, NetworkInfo, RamInfo, StorageInfo};
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System};

pub fn get_hardware_report() -> HardwareReport {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(sysinfo::MemoryRefreshKind::everything()),
    );

    // Initial refresh to get valid CPU usage
    sys.refresh_cpu_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_all();

    let cpu_info = sys
        .cpus()
        .iter()
        .map(|cpu| CpuInfo {
            model: cpu.brand().to_string(),
            cores: System::physical_core_count().unwrap_or(0),
            frequency: cpu.frequency(),
            usage: cpu.cpu_usage(),
        })
        .collect();

    let ram_info = RamInfo {
        total: sys.total_memory(),
        used: sys.used_memory(),
        free: sys.free_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
    };

    let disks = Disks::new_with_refreshed_list();
    let storage_info = disks
        .iter()
        .map(|disk| StorageInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total: disk.total_space(),
            used: disk.total_space() - disk.available_space(),
            free: disk.available_space(),
            filesystem: disk.file_system().to_string_lossy().to_string(),
        })
        .collect();

    let networks = Networks::new_with_refreshed_list();
    let network_info = networks
        .iter()
        .map(|(name, data)| NetworkInfo {
            name: name.clone(),
            received: data.total_received(),
            transmitted: data.total_transmitted(),
            mac_address: data.mac_address().to_string(),
        })
        .collect();

    HardwareReport {
        os_name: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        kernel_version: System::kernel_version().unwrap_or_default(),
        hostname: System::host_name().unwrap_or_default(),
        uptime: System::uptime(),
        cpu: cpu_info,
        ram: ram_info,
        storage: storage_info,
        network: network_info,
    }
}
