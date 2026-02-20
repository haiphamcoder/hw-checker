use crate::model::{
    CpuInfo, HardwareReport, NetworkInfo, PciDevice, RamInfo, StorageInfo, UsbDevice,
};
use raw_cpuid::CpuId;
use rusb::UsbContext;
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

    let cpuid = CpuId::new();

    // Simpler way for cache if supported
    let info = cpuid.get_vendor_info();
    let vendor_name = info.as_ref().map(|v| v.as_str()).unwrap_or("Unknown");

    let cpu_info = sys
        .cpus()
        .iter()
        .map(|cpu| CpuInfo {
            model: cpu.brand().to_string(),
            vendor_id: cpu.vendor_id().to_string(),
            brand: vendor_name.to_string(),
            cores: System::physical_core_count().unwrap_or(0),
            frequency: cpu.frequency(),
            usage: cpu.cpu_usage(),
            l1_cache: None,
            l2_cache: None,
            l3_cache: None,
        })
        .collect();

    let ram_info = RamInfo {
        total: sys.total_memory(),
        used: sys.used_memory(),
        free: sys.free_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
        manufacturer: None,
        part_number: None,
        serial_number: None,
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
            vendor: None,
            model_name: None,
            serial_number: None,
            disk_type: Some(format!("{:?}", disk.kind())),
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

    let usb_devices = get_usb_devices();
    let pci_devices = get_pci_devices();

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
        usb: usb_devices,
        pci: pci_devices,
    }
}

fn get_usb_devices() -> Vec<UsbDevice> {
    let mut devices = Vec::new();
    if let Ok(context) = rusb::Context::new() {
        if let Ok(list) = context.devices() {
            for device in list.iter() {
                if let Ok(desc) = device.device_descriptor() {
                    let handle = device.open();
                    let (m_string, p_string) = if let Ok(mut h) = handle {
                        let m = h.read_manufacturer_string_ascii(&desc).ok();
                        let p = h.read_product_string_ascii(&desc).ok();
                        (m, p)
                    } else {
                        (None, None)
                    };

                    devices.push(UsbDevice {
                        bus: device.bus_number(),
                        address: device.address(),
                        vendor_id: desc.vendor_id(),
                        product_id: desc.product_id(),
                        manufacturer: m_string,
                        product: p_string,
                    });
                }
            }
        }
    }
    devices
}

fn get_pci_devices() -> Vec<PciDevice> {
    let mut devices = Vec::new();
    if let Ok(pci) = pci_info::PciInfo::enumerate_pci() {
        for function_res in pci {
            if let Ok(function) = function_res {
                devices.push(PciDevice {
                    slot: format!("{:?}", function.location()),
                    vendor_id: function.vendor_id(),
                    device_id: function.device_id(),
                    vendor_name: None,
                    device_name: None,
                    class_name: None,
                });
            }
        }
    }
    devices
}
