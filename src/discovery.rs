use crate::model::{
    BatteryInfo, CpuInfo, HardwareReport, MotherboardInfo, NetworkInfo, PciDevice, RamInfo,
    RamStick, StorageInfo, UsbDevice,
};
use raw_cpuid::{CpuId, CpuIdReaderNative};
use rusb::UsbContext;
use smbioslib::table_load_from_device;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
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

    // Cache discovery
    let (l1, l2, l3) = get_cpu_caches(&cpuid);

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
            l1_cache: l1.clone(),
            l2_cache: l2.clone(),
            l3_cache: l3.clone(),
        })
        .collect();

    let ram_sticks = get_ram_details();
    let ram_info = RamInfo {
        total: sys.total_memory(),
        used: sys.used_memory(),
        free: sys.free_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
        sticks: ram_sticks,
    };

    let disks = Disks::new_with_refreshed_list();
    let storage_info = disks
        .iter()
        .map(|disk| {
            let name = disk.name().to_string_lossy().to_string();
            let (vendor, model, sn, interface) = get_disk_metadata(&name);
            StorageInfo {
                name: name.clone(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total: disk.total_space(),
                used: disk.total_space() - disk.available_space(),
                free: disk.available_space(),
                filesystem: disk.file_system().to_string_lossy().to_string(),
                vendor,
                model_name: model,
                serial_number: sn,
                disk_type: Some(format!("{:?}", disk.kind())),
                interface,
            }
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
    let motherboard = get_motherboard_info();
    let battery = get_battery_info();

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
        motherboard,
        battery,
    }
}

fn get_cpu_caches(
    cpuid: &CpuId<CpuIdReaderNative>,
) -> (Option<String>, Option<String>, Option<String>) {
    let mut l1 = None;
    let mut l2 = None;
    let mut l3 = None;

    if let Some(cache_params) = cpuid.get_cache_parameters() {
        for cache in cache_params {
            // raw-cpuid levels: 1 = L1, 2 = L2, etc.
            // associativity() etc. return raw values which might need +1 if they follow the CPUID spec literally.
            // But let's check if we can get a cleaner value.
            // Re-calculating with +1 only where necessary.
            let ways = cache.associativity() as u64;
            let partitions = cache.physical_line_partitions() as u64;
            let line_size = cache.coherency_line_size() as u64;
            let sets = cache.sets() as u64;

            // Intel docs say: Size = (Ways + 1) * (Partitions + 1) * (LineSize + 1) * (Sets + 1)
            // But raw-cpuid might already add them.
            // Let's test if the values are raw.
            // If they are raw, 66KB (ways=8+1=9?) -> 64KB should be ways=8.
            let size_kb = (ways * partitions * line_size * sets) / 1024;

            let level = cache.level();
            let label = format!("{} KB", size_kb);
            match level {
                1 => l1 = Some(label),
                2 => l2 = Some(label),
                3 => l3 = Some(label),
                _ => {}
            }
        }
    }

    (l1, l2, l3)
}

fn get_ram_details() -> Vec<RamStick> {
    let mut sticks = Vec::new();
    #[cfg(target_os = "linux")]
    {
        use smbioslib::{SMBiosMemoryDevice, SMBiosStruct};

        if let Ok(data) = table_load_from_device() {
            for sm_struct in data.iter() {
                if sm_struct.header.struct_type() == 17 {
                    let dev = SMBiosMemoryDevice::new(sm_struct);

                    let manufacturer_raw = format!("{}", dev.manufacturer());
                    let part_number = format!("{}", dev.part_number());
                    let serial_number = format!("{}", dev.serial_number());

                    let speed = dev.configured_memory_speed().map(|s| {
                        let s_str = format!("{:?}", s);
                        s_str
                            .chars()
                            .filter(|c| c.is_digit(10))
                            .collect::<String>()
                            .parse::<u16>()
                            .unwrap_or(0)
                    });

                    let clean = |s: String| {
                        let t = s.trim();
                        if t.is_empty()
                            || t.to_lowercase() == "unknown"
                            || t.to_lowercase() == "none"
                            || t.to_lowercase() == "not specified"
                            || t.to_lowercase().contains("empty")
                            || t == "0"
                        {
                            None
                        } else {
                            Some(t.to_string())
                        }
                    };

                    if let Some(m) = clean(manufacturer_raw) {
                        sticks.push(RamStick {
                            manufacturer: Some(map_ram_manufacturer(&m)),
                            part_number: clean(part_number),
                            serial_number: clean(serial_number),
                            speed: speed.and_then(|s| if s > 0 { Some(s) } else { None }),
                        });
                    }
                }
            }
        }
    }
    sticks
}

fn map_ram_manufacturer(id: &str) -> String {
    let id_upper = id.to_uppercase();
    if id_upper.contains("0198") {
        "Kingston".to_string()
    } else if id_upper.contains("04CB") {
        "ADATA".to_string()
    } else if id_upper.contains("00AD") || id_upper.contains("80AD") {
        "SK Hynix".to_string()
    } else if id_upper.contains("00CE") || id_upper.contains("80CE") {
        "Samsung".to_string()
    } else if id_upper.contains("012F") || id_upper.contains("812F") {
        "Micron".to_string()
    } else if id_upper.contains("029E") || id_upper.contains("829E") {
        "Corsair".to_string()
    } else if id_upper.contains("0423") || id_upper.contains("8423") {
        "Crucial".to_string()
    } else if id_upper.contains("059B") || id_upper.contains("859B") {
        "Crucial".to_string()
    } else {
        id.to_string()
    }
}

fn get_disk_metadata(
    name: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    #[cfg(target_os = "linux")]
    {
        let mut parent_name = name.to_string();
        if name.starts_with("sd") || name.starts_with("hd") {
            parent_name = name.trim_end_matches(char::is_numeric).to_string();
        } else if name.contains('p') && name.starts_with("nvme") {
            if let Some(pos) = name.rfind('p') {
                parent_name = name[..pos].to_string();
            }
        }

        let model = fs::read_to_string(format!("/sys/block/{}/device/model", parent_name))
            .ok()
            .map(|s| s.trim().to_string());
        let sn = fs::read_to_string(format!("/sys/block/{}/device/serial", parent_name))
            .ok()
            .map(|s| s.trim().to_string());
        let vendor = fs::read_to_string(format!("/sys/block/{}/device/vendor", parent_name))
            .ok()
            .map(|s| s.trim().to_string());

        let interface = if parent_name.starts_with("nvme") {
            Some("NVMe".to_string())
        } else if parent_name.starts_with("sd") {
            Some("SATA/SAS".to_string())
        } else {
            None
        };

        (vendor, model, sn, interface)
    }
    #[cfg(not(target_os = "linux"))]
    {
        (None, None, None, None)
    }
}

fn get_usb_devices() -> Vec<UsbDevice> {
    let mut devices = Vec::new();
    if let Ok(context) = rusb::Context::new() {
        if let Ok(list) = context.devices() {
            for device in list.iter() {
                if let Ok(desc) = device.device_descriptor() {
                    let handle = device.open();
                    let (m_string, p_string) = if let Ok(h) = handle {
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
    let pci_db = load_pci_db();

    if let Ok(pci) = pci_info::PciInfo::enumerate_pci() {
        for function_res in pci {
            if let Ok(function) = function_res {
                let v_id = function.vendor_id();
                let d_id = function.device_id();

                let (v_name, d_name) = pci_db
                    .get(&(v_id, d_id))
                    .map(|(v, d)| (v.clone(), d.clone()))
                    .unwrap_or_else(|| {
                        let v_only = pci_db
                            .get(&(v_id, 0xFFFF))
                            .map(|(v, _)| (v.clone(), None))
                            .unwrap_or((None, None));
                        v_only
                    });

                devices.push(PciDevice {
                    slot: format!("{:?}", function.location()),
                    vendor_id: v_id,
                    device_id: d_id,
                    vendor_name: v_name,
                    device_name: d_name,
                    class_name: None,
                });
            }
        }
    }
    devices
}

fn load_pci_db() -> HashMap<(u16, u16), (Option<String>, Option<String>)> {
    let mut db = HashMap::new();
    let paths = ["/usr/share/misc/pci.ids", "/var/lib/pciutils/pci.ids"];

    for path in paths {
        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            let mut current_vendor_id: Option<u16> = None;
            let mut current_vendor_name: Option<String> = None;

            for line in reader.lines().flatten() {
                if line.trim().is_empty() || line.starts_with('#') || line.starts_with('C') {
                    continue;
                }

                if line.starts_with("\t\t") {
                    continue;
                }

                if line.starts_with('\t') {
                    if let Some(v_id) = current_vendor_id {
                        let content = line.trim();
                        let mut parts = content.splitn(2, ' ');
                        if let Some(id_str) = parts.next() {
                            if let Ok(d_id) = u16::from_str_radix(id_str, 16) {
                                if let Some(name) = parts.next() {
                                    db.insert(
                                        (v_id, d_id),
                                        (
                                            current_vendor_name.clone(),
                                            Some(name.trim().to_string()),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                } else {
                    let mut parts = line.splitn(2, ' ');
                    if let Some(id_str) = parts.next() {
                        if let Ok(v_id) = u16::from_str_radix(id_str, 16) {
                            if let Some(name) = parts.next() {
                                current_vendor_id = Some(v_id);
                                current_vendor_name = Some(name.trim().to_string());
                                db.insert((v_id, 0xFFFF), (current_vendor_name.clone(), None));
                            }
                        }
                    }
                }
            }
            break;
        }
    }
    db
}

fn get_motherboard_info() -> Option<MotherboardInfo> {
    #[cfg(target_os = "linux")]
    {
        let read_sys = |path: &str| {
            fs::read_to_string(format!("/sys/class/dmi/id/{}", path))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "Unknown".to_string())
        };

        Some(MotherboardInfo {
            vendor: read_sys("board_vendor"),
            product: read_sys("board_name"),
            bios_vendor: read_sys("bios_vendor"),
            bios_version: read_sys("bios_version"),
            bios_date: read_sys("bios_date"),
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

fn get_battery_info() -> Vec<BatteryInfo> {
    let mut batteries = Vec::new();
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply/") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("BAT") {
                    let status = fs::read_to_string(entry.path().join("status"))
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|_| "Unknown".to_string());
                    let capacity = fs::read_to_string(entry.path().join("capacity"))
                        .map(|s| s.trim().parse::<u8>().unwrap_or(0))
                        .unwrap_or(0);

                    batteries.push(BatteryInfo {
                        name,
                        status,
                        capacity,
                    });
                }
            }
        }
    }
    batteries
}
