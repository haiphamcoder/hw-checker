use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Tabs},
};
use std::{io, time::Duration};
use sysinfo::{CpuRefreshKind, Networks, RefreshKind, System};

use crate::model::HardwareReport;

const TABS: [&str; 4] = [
    " 1: Overview ",
    " 2: CPU & RAM ",
    " 3: Storage & Network ",
    " 4: PCI & USB ",
];

struct App {
    report: HardwareReport,
    active_tab: usize,
    sys: System,
    networks: Networks,
    last_refresh: std::time::Instant,
}

impl App {
    fn new(report: HardwareReport) -> App {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(sysinfo::MemoryRefreshKind::everything()),
        );
        sys.refresh_cpu_all();

        App {
            report,
            active_tab: 0,
            sys,
            networks: Networks::new_with_refreshed_list(),
            last_refresh: std::time::Instant::now(),
        }
    }

    fn update_metrics(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.networks.refresh(true);

        // Update CPU
        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            if let Some(r_cpu) = self.report.cpu.get_mut(i) {
                r_cpu.usage = cpu.cpu_usage();
                r_cpu.frequency = cpu.frequency();
            }
        }

        // Update RAM
        self.report.ram.used = self.sys.used_memory();
        self.report.ram.free = self.sys.free_memory();
        self.report.ram.swap_used = self.sys.used_swap();

        // Update Uptime
        self.report.uptime = System::uptime();

        // Update Network
        for net in self.report.network.iter_mut() {
            if let Some((_, data)) = self.networks.iter().find(|(name, _)| *name == &net.name) {
                net.received = data.total_received();
                net.transmitted = data.total_transmitted();
            }
        }

        self.last_refresh = std::time::Instant::now();
    }
}

pub fn run_tui(report: HardwareReport) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(report);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()>
where
    B::Error: std::error::Error + Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Right | KeyCode::Tab => {
                        app.active_tab = (app.active_tab + 1) % TABS.len()
                    }
                    KeyCode::Left => {
                        if app.active_tab > 0 {
                            app.active_tab -= 1;
                        } else {
                            app.active_tab = TABS.len() - 1;
                        }
                    }
                    KeyCode::Char('1') => app.active_tab = 0,
                    KeyCode::Char('2') => app.active_tab = 1,
                    KeyCode::Char('3') => app.active_tab = 2,
                    KeyCode::Char('4') => app.active_tab = 3,
                    _ => {}
                }
            }
        }

        if app.last_refresh.elapsed() >= Duration::from_secs(1) {
            app.update_metrics();
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(4),    // Content
            Constraint::Length(5), // Footer
        ])
        .split(size);

    // Header
    let header_text = format!(
        " hwchecker v2.0 - TUI Mode | Host: {} | OS: {} {} ",
        app.report.hostname, app.report.os_name, app.report.os_version
    );
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(header, main_chunks[0]);

    // Tabs
    let titles = TABS.iter().map(|t| Line::from(*t)).collect::<Vec<_>>();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL))
        .select(app.active_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, main_chunks[1]);

    // Content
    match app.active_tab {
        0 => render_overview(f, app, main_chunks[2]),
        1 => render_cpu_ram(f, app, main_chunks[2]),
        2 => render_storage_network(f, app, main_chunks[2]),
        3 => render_peripherals(f, app, main_chunks[2]),
        _ => {}
    }

    // Health / Logs Block -> footer
    let mut health_text = String::new();
    if let Some(mobo) = &app.report.motherboard {
        health_text.push_str(&format!(
            " Motherboard: {} | Bios: {} ({})\n",
            mobo.vendor, mobo.bios_version, mobo.bios_date
        ));
    }
    for bat in &app.report.battery {
        health_text.push_str(&format!(
            " Battery {}: {}% ({})\n",
            bat.name, bat.capacity, bat.status
        ));
    }

    let health_block = Paragraph::new(health_text).block(
        Block::default()
            .title(" System Health / [Press 'q' or 'Esc' to quit, Arrows/1-4 to navigate] ")
            .borders(Borders::ALL),
    );
    f.render_widget(health_block, main_chunks[3]);
}

fn render_overview(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    // System Info Block
    let sys_text = format!(
        "\n OS: {} {}\n Kernel: {}\n Uptime: {}s\n",
        app.report.os_name, app.report.os_version, app.report.kernel_version, app.report.uptime
    );
    let sys_block = Paragraph::new(sys_text).block(
        Block::default()
            .title(" System Summary ")
            .borders(Borders::ALL),
    );
    f.render_widget(sys_block, top_chunks[0]);

    // CPU Info Block (Simple)
    let mut cpu_text = String::new();
    if let Some(first_cpu) = app.report.cpu.first() {
        cpu_text.push_str(&format!(" Model: {}\n", first_cpu.model));
        cpu_text.push_str(&format!(" Physical Cores: {}\n\n", first_cpu.cores));
    }
    for (i, cpu) in app.report.cpu.iter().enumerate().take(8) {
        cpu_text.push_str(&format!(
            " Core {}: {:>5.1}% | {} MHz\n",
            i, cpu.usage, cpu.frequency
        ));
    }
    if app.report.cpu.len() > 8 {
        cpu_text.push_str(" ... (See CPU & RAM tab for more)\n");
    }
    let cpu_block =
        Paragraph::new(cpu_text).block(Block::default().title(" CPU Info ").borders(Borders::ALL));
    f.render_widget(cpu_block, top_chunks[1]);

    let bot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // RAM Info Block (Gauge)
    let ram_used = app.report.ram.used as f64;
    let ram_total = app.report.ram.total as f64;
    let ram_ratio = if ram_total > 0.0 {
        ram_used / ram_total
    } else {
        0.0
    };

    let ram_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(bot_chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().title(" RAM Usage ").borders(Borders::ALL))
        .gauge_style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        )
        .percent((ram_ratio * 100.0) as u16)
        .label(format!(
            "{:.1} / {:.1} GB",
            ram_used / 1024.0 / 1024.0 / 1024.0,
            ram_total / 1024.0 / 1024.0 / 1024.0
        ));
    f.render_widget(gauge, ram_chunks[0]);

    let ram_text = format!(
        "\n Free: {:.1} GB\n (See CPU & RAM tab for DIMM details)\n",
        app.report.ram.free as f64 / 1024.0 / 1024.0 / 1024.0
    );
    let ram_details = Paragraph::new(ram_text)
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM));
    f.render_widget(ram_details, ram_chunks[1]);

    // Storage Block
    let header_cells = ["Mount", "FS", "Total (GB)", "Used (%)"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let mut rows = Vec::new();
    for disk in app.report.storage.iter().take(5) {
        let total_gb = disk.total as f64 / 1024.0 / 1024.0 / 1024.0;
        let usage = if disk.total > 0 {
            (disk.used as f64 / disk.total as f64) * 100.0
        } else {
            0.0
        };

        rows.push(Row::new(vec![
            Cell::from(disk.mount_point.clone()),
            Cell::from(disk.filesystem.clone()),
            Cell::from(format!("{:.1}", total_gb)),
            Cell::from(format!("{:.1}%", usage)),
        ]));
    }

    let storage_table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(30),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(" Storage (Top 5) ")
            .borders(Borders::ALL),
    );
    f.render_widget(storage_table, bot_chunks[1]);
}

fn render_cpu_ram(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // CPU Detail
    let mut cpu_text = String::new();
    if let Some(first_cpu) = app.report.cpu.first() {
        cpu_text.push_str(&format!(" Model: {}\n", first_cpu.model));
        cpu_text.push_str(&format!(" Brand: {}\n", first_cpu.brand));
        cpu_text.push_str(&format!(" Vendor: {}\n", first_cpu.vendor_id));
        cpu_text.push_str(&format!(" Cores: {}\n\n", first_cpu.cores));

        cpu_text.push_str(" Caches:\n");
        cpu_text.push_str(&format!(
            " L1: {}\n",
            first_cpu.l1_cache.as_deref().unwrap_or("N/A")
        ));
        cpu_text.push_str(&format!(
            " L2: {}\n",
            first_cpu.l2_cache.as_deref().unwrap_or("N/A")
        ));
        cpu_text.push_str(&format!(
            " L3: {}\n\n",
            first_cpu.l3_cache.as_deref().unwrap_or("N/A")
        ));
    }
    for (i, cpu) in app.report.cpu.iter().enumerate() {
        cpu_text.push_str(&format!(
            " Core {}: {:>5.1}% | {} MHz\n",
            i, cpu.usage, cpu.frequency
        ));
    }
    let cpu_block = Paragraph::new(cpu_text).block(
        Block::default()
            .title(" CPU Details ")
            .borders(Borders::ALL),
    );
    f.render_widget(cpu_block, chunks[0]);

    // RAM Detail
    let mut ram_text = String::new();
    ram_text.push_str(&format!(
        " Swap Total: {:.1} GB\n",
        app.report.ram.swap_total as f64 / 1024.0 / 1024.0 / 1024.0
    ));
    ram_text.push_str(&format!(
        " Swap Used:  {:.1} GB\n\n",
        app.report.ram.swap_used as f64 / 1024.0 / 1024.0 / 1024.0
    ));

    ram_text.push_str(" DIMM Details:\n");
    for (i, stick) in app.report.ram.sticks.iter().enumerate() {
        ram_text.push_str(&format!(" Slot {}:\n", i));
        ram_text.push_str(&format!(
            "   Manufacturer: {}\n",
            stick.manufacturer.as_deref().unwrap_or("Unknown")
        ));
        ram_text.push_str(&format!(
            "   Part Number:  {}\n",
            stick.part_number.as_deref().unwrap_or("Unknown")
        ));
        ram_text.push_str(&format!(
            "   Serial Num:   {}\n",
            stick.serial_number.as_deref().unwrap_or("Unknown")
        ));
        ram_text.push_str(&format!(
            "   Speed:        {} MT/s\n\n",
            stick.speed.unwrap_or(0)
        ));
    }
    let ram_block = Paragraph::new(ram_text).block(
        Block::default()
            .title(" RAM & Swap Details ")
            .borders(Borders::ALL),
    );
    f.render_widget(ram_block, chunks[1]);
}

fn render_storage_network(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Storage Table
    let header_cells = ["Disk", "Mount", "FS", "Total", "Used", "Interface", "Model"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let mut rows = Vec::new();
    for disk in &app.report.storage {
        let total_gb = disk.total as f64 / 1024.0 / 1024.0 / 1024.0;
        let usage = if disk.total > 0 {
            (disk.used as f64 / disk.total as f64) * 100.0
        } else {
            0.0
        };

        rows.push(Row::new(vec![
            Cell::from(disk.name.clone()),
            Cell::from(disk.mount_point.clone()),
            Cell::from(disk.filesystem.clone()),
            Cell::from(format!("{:.1} GB", total_gb)),
            Cell::from(format!("{:.1}%", usage)),
            Cell::from(disk.interface.as_deref().unwrap_or("Unknown").to_string()),
            Cell::from(disk.model_name.as_deref().unwrap_or("Unknown").to_string()),
        ]));
    }
    let storage_table = Table::new(
        rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(30),
        ],
    )
    .header(header_row)
    .block(
        Block::default()
            .title(" Storage Details ")
            .borders(Borders::ALL),
    );
    f.render_widget(storage_table, chunks[0]);

    // Network Table
    let net_header = ["Interface", "MAC Address", "RX (MB)", "TX (MB)"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let net_header_row = Row::new(net_header).height(1).bottom_margin(1);

    let mut net_rows = Vec::new();
    for net in &app.report.network {
        let rx_mb = net.received as f64 / 1024.0 / 1024.0;
        let tx_mb = net.transmitted as f64 / 1024.0 / 1024.0;
        net_rows.push(Row::new(vec![
            Cell::from(net.name.clone()),
            Cell::from(net.mac_address.clone()),
            Cell::from(format!("{:.2} MB", rx_mb)),
            Cell::from(format!("{:.2} MB", tx_mb)),
        ]));
    }
    let network_table = Table::new(
        net_rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(net_header_row)
    .block(
        Block::default()
            .title(" Network Interfaces ")
            .borders(Borders::ALL),
    );
    f.render_widget(network_table, chunks[1]);
}

fn render_peripherals(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // PCI Table
    let pci_header = ["Slot", "Vendor", "Device"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let pci_header_row = Row::new(pci_header).height(1).bottom_margin(1);

    let mut pci_rows = Vec::new();
    for pci in &app.report.pci {
        let vendor = pci
            .vendor_name
            .clone()
            .unwrap_or_else(|| format!("0x{:04x}", pci.vendor_id));
        let device = pci
            .device_name
            .clone()
            .unwrap_or_else(|| format!("0x{:04x}", pci.device_id));
        pci_rows.push(Row::new(vec![
            Cell::from(pci.slot.clone()),
            Cell::from(vendor),
            Cell::from(device),
        ]));
    }
    let pci_table = Table::new(
        pci_rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(40),
            Constraint::Percentage(45),
        ],
    )
    .header(pci_header_row)
    .block(
        Block::default()
            .title(" PCI Devices ")
            .borders(Borders::ALL),
    );
    f.render_widget(pci_table, chunks[0]);

    // USB Table
    let usb_header = ["Bus:Addr", "Vendor", "Product"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let usb_header_row = Row::new(usb_header).height(1).bottom_margin(1);

    let mut usb_rows = Vec::new();
    for usb in &app.report.usb {
        let bus_addr = format!("{:03}:{:03}", usb.bus, usb.address);
        let vendor = usb
            .manufacturer
            .clone()
            .unwrap_or_else(|| format!("0x{:04x}", usb.vendor_id));
        let product = usb
            .product
            .clone()
            .unwrap_or_else(|| format!("0x{:04x}", usb.product_id));
        usb_rows.push(Row::new(vec![
            Cell::from(bus_addr),
            Cell::from(vendor),
            Cell::from(product),
        ]));
    }
    let usb_table = Table::new(
        usb_rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(40),
            Constraint::Percentage(45),
        ],
    )
    .header(usb_header_row)
    .block(
        Block::default()
            .title(" USB Devices ")
            .borders(Borders::ALL),
    );
    f.render_widget(usb_table, chunks[1]);
}
