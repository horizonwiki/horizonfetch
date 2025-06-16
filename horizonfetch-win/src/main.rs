// HorizonFetch (hf) || tg: @xd_sergii || github: https://github.com/horizonl1nux/horizonfetch || v0.3.5-2 || 16.06.2025
use std::{fs, env, io};
use std::io::stdout;
use std::collections::HashMap;
use crossterm::{execute, cursor::MoveTo, style::Print};
use crossterm::terminal::{Clear, ClearType};
use once_cell::sync::Lazy;
use wmi::{WMIConnection, COMLibrary};
use serde::Deserialize;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;
use windows::{
    core::{PWSTR, PCWSTR},
    Win32::{
        Foundation::CloseHandle,
        System::{
            SystemInformation::{GetComputerNameExW, COMPUTER_NAME_FORMAT, GetTickCount64, MEMORYSTATUSEX, GlobalMemoryStatusEx},
            WindowsProgramming::GetUserNameW,
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
            },
            ProcessStatus::GetProcessImageFileNameW,
        },
        Graphics::Gdi::{
            EnumDisplayDevicesW, EnumDisplaySettingsW, DISPLAY_DEVICEW, DEVMODEW, ENUM_CURRENT_SETTINGS,
            DISPLAY_DEVICE_ATTACHED_TO_DESKTOP,
        },
        Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1},
        Globalization::GetUserDefaultLocaleName,
        Storage::FileSystem::{GetLogicalDriveStringsW, GetDiskFreeSpaceExW},
    },
};
use std::sync::mpsc;
use std::thread;

struct Config {
    ascii_art: String,
    color: String,
    info_color: String,
    title_color: String,
    show_user: bool,
    show_os: bool,
    show_uptime: bool,
    show_shell: bool,
    show_de: bool,
    show_screen: bool,
    show_motherboard: bool,
    show_cpu: bool,
    show_gpu: bool,
    show_ram: bool,
    show_swap: bool,
    show_locale: bool,
    show_disk: bool,
    show_vram_gb: bool,
    show_ram_ext_info: bool,
    show_color_scheme: bool,
}

#[derive(Deserialize, Debug)]
struct PhysicalMemory {
    #[serde(rename = "Capacity")]
    capacity: Option<u64>,
    #[serde(rename = "Speed")]
    speed: Option<u32>,
}

// default ascii
const DEFAULT_ASCII: &str = r#"


  1111111  1111111
  1111111  1111111
  1111111  1111111
ㅤ
  1111111  1111111
  1111111  1111111
  1111111  11111;.
"#;

// once_cell lazy
static CONFIG: Lazy<Config> = Lazy::new(|| {
    let user_profile = match env::var("USERPROFILE") {
        Ok(path) => path,
        Err(_) => return Config::default(),
    };
    
    // config path
    let config_path = format!("{}\\horizonfetch\\hf.config", user_profile);
    
    Config::load(&config_path).unwrap_or_else(|_| Config::default())
});

impl Default for Config {
    fn default() -> Self {
        Config {
            ascii_art: DEFAULT_ASCII.to_string(),
            color: "34".to_string(),
            info_color: "38;5;117".to_string(),
            title_color: "38;5;110".to_string(),
            show_user: true,
            show_os: true,
            show_uptime: true,
            show_shell: true,
            show_de: true,
            show_screen: true,
            show_motherboard: true,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_swap: true,
            show_locale: true,
            show_disk: true,
            show_vram_gb: false,
            show_ram_ext_info: false,
            show_color_scheme: true,
        }
    }
}

impl Config {
    fn load(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let ascii_art = extract_ascii_art(&content).unwrap_or(DEFAULT_ASCII).to_string();
        let color = extract_color_param(&content, "ascii_color").unwrap_or("34").to_string();
        let info_color = extract_color_param(&content, "info_color").unwrap_or("38;5;117").to_string();
        let title_color = extract_color_param(&content, "title_color").unwrap_or("38;5;110").to_string();  
        let show_user = extract_color_param(&content, "show_user").map_or(true, |v| v == "true");
        let show_os = extract_color_param(&content, "show_os").map_or(true, |v| v == "true");
        let show_uptime = extract_color_param(&content, "show_uptime").map_or(true, |v| v == "true");
        let show_shell = extract_color_param(&content, "show_shell").map_or(true, |v| v == "true");
        let show_de = extract_color_param(&content, "show_de").map_or(true, |v| v == "true");
        let show_screen = extract_color_param(&content, "show_screen").map_or(true, |v| v == "true");
        let show_motherboard = extract_color_param(&content, "show_motherboard").map_or(true, |v| v == "true");
        let show_cpu = extract_color_param(&content, "show_cpu").map_or(true, |v| v == "true");
        let show_gpu = extract_color_param(&content, "show_gpu").map_or(true, |v| v == "true");
        let show_ram = extract_color_param(&content, "show_ram").map_or(true, |v| v == "true");
        let show_swap = extract_color_param(&content, "show_swap").map_or(true, |v| v == "true");
        let show_locale = extract_color_param(&content, "show_locale").map_or(true, |v| v == "true");
        let show_disk = extract_color_param(&content, "show_disk").map_or(true, |v| v == "true");
        let show_vram_gb = extract_color_param(&content, "show_vram_gb").map_or(false, |v| v == "true");
        let show_ram_ext_info = extract_color_param(&content, "show_ram_ext_info").map_or(false, |v| v == "true");
        let show_color_scheme = extract_color_param(&content, "show_color_scheme").map_or(true, |v| v == "true");
    Ok(Config { ascii_art, color, info_color, title_color, show_user, show_os, show_uptime, show_shell, show_de, show_screen, show_motherboard, show_cpu, show_gpu, show_ram, show_swap, show_locale, show_disk, show_vram_gb, show_ram_ext_info, show_color_scheme })
    }
}

// {| |}
fn extract_ascii_art(content: &str) -> Option<&str> {
    let start = content.find("{|")? + 2;
    let end = content.find("|}")?;
    Some(&content[start..end])
}

fn extract_color_param<'a>(content: &'a str, param: &str) -> Option<&'a str> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with(param) {
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let value = parts[1].trim();
                if value.starts_with('"') && value.ends_with('"') && value.len() > 2 {
                    return Some(&value[1..value.len()-1]);
                } else if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn is_valid_ansi_code(code: &str) -> bool {
    if code.is_empty() {
        return false;
    }
    
    // validity check for ascii escape codes
    if let Ok(num) = code.parse::<u8>() {
        return matches!(num,
            30..=37 | 40..=47 | 
            90..=97 | 100..=107
        );
    }
    
    let parts: Vec<&str> = code.split(';').collect();
    match parts.as_slice() {
        ["38", "5", color] if color.parse::<u8>().is_ok() => true,
        ["38", "2", r, g, b] => {
            r.parse::<u8>().is_ok() &&
            g.parse::<u8>().is_ok() &&
            b.parse::<u8>().is_ok()
        }
        _ => false,
    }
}

fn get_username() -> String {
    let mut buffer = [0u16; 257];
    let mut size = buffer.len() as u32;
    unsafe {
        let _ = GetUserNameW(Some(PWSTR(buffer.as_mut_ptr())), &mut size);
    }
    String::from_utf16_lossy(&buffer[..size as usize]).trim().to_string()
}

fn get_hostname() -> String {
    let mut buffer = [0u16; 257];
    let mut size = buffer.len() as u32;
    unsafe {
        let _ = GetComputerNameExW(
            COMPUTER_NAME_FORMAT(0), 
            Some(PWSTR(buffer.as_mut_ptr())),
            &mut size,
        );
    }
    String::from_utf16_lossy(&buffer[..size as usize]).trim().to_string()
}

fn get_windows_edition() -> windows::core::Result<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .map_err(|_| windows::core::Error::from_win32())?;
    let product_name: String = key.get_value("ProductName").unwrap_or_default();
    let build_number: String = key.get_value("CurrentBuildNumber").unwrap_or_default();

    let name = if build_number.parse::<u32>().unwrap_or(0) >= 22000 {
        product_name.replace("Windows 10", "Windows 11")
    } else {
        product_name
    };
    Ok(name)
}

fn detect_shell() -> Result<String, ()> {
    let parent_pid = get_parent_pid(std::process::id()).ok_or(())?;
    let handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, parent_pid) }
        .map_err(|_| ())?;
    let mut buffer = [0u16; 260];
    let size = unsafe { GetProcessImageFileNameW(handle, &mut buffer) };
    unsafe { let _ = CloseHandle(handle); };
    let process_name = String::from_utf16_lossy(&buffer[..size as usize]).to_lowercase();
    Ok(
        if process_name.contains("powershell.exe") {
            "PowerShell"
        } else if process_name.contains("cmd.exe") {
            "CMD"
        } else if process_name.contains("wt.exe") || process_name.contains("windowsterminal.exe") {
            "Windows Terminal"
        } else {
            "CMD"
        }
        .to_string(),
    )
}

fn enum_display_device(index: u32) -> windows::core::Result<DISPLAY_DEVICEW> {
    let mut device = DISPLAY_DEVICEW {
        cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };
    unsafe { EnumDisplayDevicesW(PCWSTR::null(), index, &mut device, 0).ok()? }
    Ok(device)
}

fn get_screen_resolution(device_name: PCWSTR) -> Option<(u32, u32, u32)> {
    let mut devmode = DEVMODEW {
        dmSize: std::mem::size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };
    unsafe {
        if EnumDisplaySettingsW(device_name, ENUM_CURRENT_SETTINGS, &mut devmode).as_bool() {
            return Some((devmode.dmPelsWidth, devmode.dmPelsHeight, devmode.dmDisplayFrequency));
        }
    }
    None
}

fn get_motherboard_name() -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\BIOS") {
        if let Ok(product) = key.get_value::<String, _>("BaseBoardProduct") {
            let trimmed = product.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn get_used_ram_gb() -> Result<f64, Box<dyn std::error::Error>> {
    let mut mem_status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { GlobalMemoryStatusEx(&mut mem_status)?; }
    Ok((mem_status.ullTotalPhys - mem_status.ullAvailPhys) as f64 / 1_073_741_824.0)
}

fn get_swap_total_gb() -> f64 {
    let mut mem_status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { let _ = GlobalMemoryStatusEx(&mut mem_status); }
    let swap_total = mem_status.ullTotalPageFile.saturating_sub(mem_status.ullTotalPhys);
    swap_total as f64 / 1_073_741_824.0
}

fn get_drive_usage(drive: &str) -> (u64, u64) {
    let mut free_bytes = 0u64;
    let mut total_bytes = 0u64;
    let path: Vec<u16> = format!("{}\\", drive).encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(path.as_ptr()),
            Some(&mut free_bytes),
            Some(&mut total_bytes),
            None,
        )
        .ok();
    }
    let total_gb = total_bytes / 1_073_741_824;
    let used_gb = (total_bytes - free_bytes) / 1_073_741_824;
    (used_gb, total_gb)
}

fn get_parent_pid(pid: u32) -> Option<u32> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut found = None;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    found = Some(entry.th32ParentProcessID);
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        found
    }
}

fn get_parent_process_name() -> Option<String> {
    use windows::Win32::System::Diagnostics::ToolHelp::*;
    use windows::Win32::Foundation::CloseHandle;
    use std::mem::size_of;
    let pid = std::process::id();
    unsafe {
     let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
       let mut entry = PROCESSENTRY32W {
         dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == pid {
                    let parent_pid = entry.th32ParentProcessID;
                    let mut parent_entry = PROCESSENTRY32W {
                        dwSize: size_of::<PROCESSENTRY32W>() as u32,
                        ..Default::default()
                    };
                    if Process32FirstW(snapshot, &mut parent_entry).is_ok() {
                        loop {
                            if parent_entry.th32ProcessID == parent_pid {
                                let name = String::from_utf16_lossy(
                                    &parent_entry.szExeFile[..parent_entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(parent_entry.szExeFile.len())]
                                );
                              let _ = CloseHandle(snapshot);
                                return Some(name.trim().to_lowercase());
                            }
                            if Process32NextW(snapshot, &mut parent_entry).is_err() {
                                break;
                            }
                        }
                    }
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
       let _ = CloseHandle(snapshot);
    }
    None
}

fn is_launched_by_explorer() -> bool {
    if let Some(parent) = get_parent_process_name() {
        parent == "explorer.exe"
    } else {
        false
    }
}

fn main() -> io::Result<()> {
    let (ram_tx, ram_rx) = mpsc::channel();
    thread::spawn(move || {
    let con = WMIConnection::new(COMLibrary::new().expect("COM init failed")).expect("WMI init failed");
    let results: Vec<PhysicalMemory> = con.raw_query("SELECT Capacity, Speed FROM Win32_PhysicalMemory").expect("WMI query failed");
    ram_tx.send(results).unwrap();
});

    execute!(stdout(), Clear(ClearType::All)).unwrap();
    let _art_width = CONFIG.ascii_art.lines().map(|l| l.len()).max().unwrap_or(0) + 2;
    let mut _info_y = 1;
    let color = if is_valid_ansi_code(&CONFIG.color) {
        CONFIG.color.as_str()
    } else {
        "34"
    };
    let info_color = if is_valid_ansi_code(&CONFIG.info_color) {
        CONFIG.info_color.as_str()
    } else {
        "38;5;117"
    };
    let title_color = if is_valid_ansi_code(&CONFIG.title_color) {
        CONFIG.title_color.as_str()
    } else {
        "38;5;110"
    };
    let ascii_lines: Vec<_> = CONFIG.ascii_art.lines().collect();
    let max_art_line_len = ascii_lines.iter()
    .map(|line| {
        let v = strip_ansi_escapes::strip(line.as_bytes());
        String::from_utf8(v)
            .map(|s| s.chars().count())
            .unwrap_or_else(|_| line.chars().count())
    })
    .max()
    .unwrap_or(0);

    let art_width = max_art_line_len + 3;

    let mut y = 0;
    for line in CONFIG.ascii_art.lines() {
    let trimmed_line = line.trim_end();
    if !trimmed_line.is_empty() {
        execute!(
            io::stdout(),
            MoveTo(0, y as u16),
            Print(format!("\x1b[{}m{}\x1b[0m", color, trimmed_line))
        )?;
        y += 1;
    }
}

    let mut info_y = 0;
    if CONFIG.show_user {
    let username = get_username();
    let hostname = get_hostname();
    execute!(
        io::stdout(),
        MoveTo(art_width as u16, info_y),
        Print(format!("\x1b[{}m{}@{}\x1b[0m", info_color, username, hostname))
    )?;
    info_y += 1;

    execute!(
        io::stdout(),
        MoveTo(art_width as u16, info_y),
        Print(format!("\x1b[97m-------\x1b[0m"))
    )?;
    info_y += 1;
}

    if CONFIG.show_os {
    let edition = get_windows_edition()?;
    execute!(
    io::stdout(),
    MoveTo(art_width as u16, info_y),
    Print(format!("\x1b[{}mOS:\x1b[0m \x1b[{}m{}\x1b[0m", title_color, info_color, edition)),
    )?;
    info_y += 1;
}
    if CONFIG.show_uptime {
    let uptime_ms = unsafe { GetTickCount64() };
    let uptime_sec = uptime_ms / 1000;
    let uptime_min = uptime_sec / 60;
    let uptime_hr = uptime_min / 60;
    let uptime_days = uptime_hr / 24;
    let uptime_str = if uptime_days > 0 {

        format!(
    "\x1b[{}mUptime:\x1b[0m \x1b[{}m{}\x1b[0m \x1b[97mdays\x1b[0m \x1b[{}m{}\x1b[0m \x1b[97mhours\x1b[0m \x1b[{}m{}\x1b[0m \x1b[97mminutes\x1b[0m",
    title_color, info_color, uptime_days, info_color, uptime_hr % 24, info_color, uptime_min % 60
        )   
    } else {
        format!(
    "\x1b[{}mUptime:\x1b[0m \x1b[{}m{}\x1b[0m \x1b[97mhours\x1b[0m \x1b[{}m{}\x1b[0m \x1b[97mminutes\x1b[0m",
    title_color, info_color, uptime_hr % 24, info_color, uptime_min % 60
    )    };
    execute!(
        io::stdout(),
        MoveTo(art_width as u16, info_y),
        Print(uptime_str)
    )?;
    info_y += 1;
}

    if CONFIG.show_shell {
    let shell_str = match detect_shell() {
    Ok(shell) => format!("\x1b[{}mShell:\x1b[0m \x1b[{}m{}\x1b[0m", title_color, info_color, shell),
    Err(_) => "Shell: CMD".to_string(),
    };
    execute!(
        io::stdout(),
        MoveTo(art_width as u16, info_y),
        Print(shell_str)
    )?;
    info_y += 1;
}  

    if CONFIG.show_de {
    execute!(
        io::stdout(),
        MoveTo(art_width as u16, info_y),
        Print(format!("\x1b[{}mDE:\x1b[0m \x1b[{}mFluent\x1b[0m", title_color, info_color))
    )?;
    info_y += 1;
}

    if CONFIG.show_screen {
    let mut device_index = 0;
    while let Ok(device) = enum_display_device(device_index) {
        if device.StateFlags.0 & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP.0 != 0 {
            let device_name = PCWSTR(device.DeviceName.as_ptr());
            if let Some((width, height, refresh_rate)) = get_screen_resolution(device_name) {
                execute!(
                    io::stdout(),
                    MoveTo(art_width as u16, info_y),
                    Print(format!(
                        "\x1b[{}mScreen:\x1b[0m \x1b[{}m{}\x1b[0m\x1b[97mx\x1b[0m\x1b[{}m{}\x1b[0m \x1b[97m@\x1b[0m \x1b[{}m{}Hz\x1b[0m",
                        title_color, info_color, width, info_color, height, info_color, refresh_rate
                    ))
                )?;
                info_y += 1;
            }
        }
        device_index += 1;
    }}
    if CONFIG.show_motherboard {
    execute!(
        io::stdout(),
        MoveTo(art_width as u16, info_y),
        Print(format!(
            "\x1b[{}mMotherboard:\x1b[0m \x1b[{}m{}\x1b[0m",
            title_color, info_color, get_motherboard_name().unwrap_or_else(|| "Unknown".to_string())
        ))
    )?;
    info_y += 1;
}

    if CONFIG.show_cpu {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let cpus_key = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\CentralProcessor")?;
    let mut cpu_map: HashMap<String, usize> = HashMap::new();

    for cpu_index in cpus_key.enum_keys().flatten() {
        if let Ok(cpu_key) = cpus_key.open_subkey(&cpu_index) {
            if let Ok(cpu_name) = cpu_key.get_value::<String, _>("ProcessorNameString") {
                let name = cpu_name.trim().to_string();
                    *cpu_map.entry(name).or_insert(0) += 1;
        }
    }
}
    let cpu_str = cpu_map
    .iter()
    .map(|(name, count)| {
        if *count > 1 {
            format!("{} ({} threads)", name, count)
     } else {
            name.clone()
            }})
    .collect::<Vec<_>>()
    .join("\n     ");

    execute!(
    io::stdout(),
    MoveTo(art_width as u16, info_y),
    Print(format!("\x1b[{}mCpu:\x1b[0m \x1b[{}m{}\x1b[0m", title_color, info_color, cpu_str))
)?;
    info_y += cpu_map.len() as u16;
}

    if CONFIG.show_gpu {
    unsafe {
    let dxgi_factory: IDXGIFactory1 = CreateDXGIFactory1()?;
    let mut i = 0;
    let mut found = false;
    let mut results = Vec::new();
    while let Ok(adapter) = dxgi_factory.EnumAdapters1(i) {
        let desc = adapter.GetDesc1()?;
        let gpu_name: String = desc.Description
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
            .collect();
        if !gpu_name.contains("Microsoft Basic Render Driver") {
    let vram_gb = desc.DedicatedVideoMemory as f64 / (1024.0 * 1024.0 * 1024.0);
    let gpu_line = if CONFIG.show_vram_gb {
        format!(
            "\x1b[{}m{}\x1b[0m \x1b[{}m{}\x1b[0m \x1b[{}m{:.2}GB\x1b[0m",
            title_color,
            if i == 0 { "Gpu:" } else { "    " },
            info_color,
            gpu_name.trim(),
            info_color,
            vram_gb
        )
    } else {
        format!(
            "\x1b[{}m{}\x1b[0m \x1b[{}m{}\x1b[0m",
            title_color,
            if i == 0 { "Gpu:" } else { "    " },
            info_color,
            gpu_name.trim()
        )
    };
    results.push(gpu_line);
    found = true;
}
        i += 1;
    }
    if found {
        for line in results {
            execute!(
                io::stdout(),
                MoveTo(art_width as u16, info_y),
                Print(line)
            )?;
            info_y += 1;
        }
    } else {
        execute!(
            io::stdout(),
            MoveTo(art_width as u16, info_y),
            Print(format!("\x1b[{}mGpu:\x1b[0m", title_color))
        )?;
        info_y += 1;
    }}} 

    if CONFIG.show_ram {
    let results = ram_rx.recv().unwrap();
    let (total_bytes, max_speed, modules) = results.iter().fold((0u64, 0u32, 0u32), |(total, speed, count), mem| {
    let cap = mem.capacity.unwrap_or(0);
    let spd = mem.speed.unwrap_or(0);
    (total + cap, speed.max(spd), count + (cap > 0) as u32)
});

    let total_gb = total_bytes as f64 / 1_073_741_824.0;
    let used_gb = get_used_ram_gb().expect("RAM info failed") as f64;
    let percent = if total_gb > 0.0 { used_gb * 100.0 / total_gb } else { 0.0 };
    let size_per_module = if modules > 0 { total_gb / modules as f64 } else { 0.0 };

    let output = if modules > 0 && CONFIG.show_ram_ext_info {
    format!(
        "\x1b[{}mRam:\x1b[0m \x1b[{}m{:.2}\x1b[0m \x1b[97m/\x1b[0m \x1b[{}m{:.2}gb ({:.0}%) ({}x{:.0}gb, {} MHz)\x1b[0m",
        title_color, info_color, used_gb, info_color, total_gb, percent, modules, size_per_module, max_speed
    )
    } else {
    format!(
        "\x1b[{}mRam:\x1b[0m \x1b[{}m{:.2}\x1b[0m \x1b[97m/\x1b[0m \x1b[{}m{:.2}gb ({:.0}%)\x1b[0m",
        title_color, info_color, used_gb, info_color, total_gb, percent
    )
    };

    execute!(
    io::stdout(),
    MoveTo(art_width as u16, info_y),
    Print(output)

    )?;
    info_y += 1;
}

    if CONFIG.show_swap {
    execute!(
    io::stdout(),
    MoveTo(art_width as u16, info_y),
    Print(format!("\x1b[{}mSwap:\x1b[0m \x1b[{}m{:.2}gb\x1b[0m",title_color, info_color, get_swap_total_gb()))
)?;
    info_y += 1;
}

    if CONFIG.show_locale {
    let mut locale_name = [0u16; 85];
    unsafe {
    GetUserDefaultLocaleName(&mut locale_name);
}
    let locale = String::from_utf16_lossy(&locale_name)
    .trim_end_matches('\0')
    .to_string();

    execute!(
    io::stdout(),
    MoveTo(art_width as u16, info_y),
    Print(format!("\x1b[{}mLocale:\x1b[0m \x1b[{}m{}\x1b[0m", title_color, info_color, locale))
)?;
    info_y += 1;
}

    if CONFIG.show_disk {
    let mut buffer = [0u16; 512];
    let size = unsafe { GetLogicalDriveStringsW(Some(&mut buffer)) };

    let drives = buffer[..size as usize]
        .split(|&c| c == 0)
        .filter(|d| !d.is_empty())
        .map(String::from_utf16_lossy)
        .collect::<Vec<_>>();

    if CONFIG.show_disk {
    if drives.is_empty() {
        execute!(
            io::stdout(),
            MoveTo(art_width as u16, info_y),
            Print(format!("\x1b[{}mDisk:\x1b[0m \x1b[{}merror :(\x1b[0m", title_color, info_color))
        )?;
        info_y += 1;
    } else {
        let max_len = drives.iter().map(|d| d.len()).max().unwrap_or(3);
        for drive in drives.iter() {
            let (used, total) = get_drive_usage(drive);
            if total == 0 {
                continue;
            }
            let percent = (used as f64 / total as f64 * 100.0).round() as u64;
            let line = format!(
                "\x1b[{}mDisk:\x1b[0m \x1b[97m{:<width$}\x1b[0m \x1b[{}m{:>3}gb\x1b[0m \x1b[97m/\x1b[0m \x1b[{}m{:>3}gb ({}%)\x1b[0m",
                title_color,
                drive,
                info_color,
                used,
                info_color,
                total,
                percent,
                width = max_len
            );
            execute!(
                io::stdout(),
                MoveTo(art_width as u16, info_y),
                Print(line)
            )?;
            info_y += 1;
        }
    }
}
}
    if CONFIG.show_color_scheme {
    info_y += 1;
    let top_colors = [0, 91, 92, 93, 94, 95, 96, 97];
    let bottom_colors = [30, 31, 32, 33, 34, 35, 36, 37];

    let top_line: String = top_colors.iter()
    .map(|&c| if c == 0 { "   ".to_string() } else { format!("\x1b[{}m███\x1b[0m", c) })
    .collect();

    let bottom_line: String = bottom_colors.iter()
    .map(|&c| format!("\x1b[{}m███\x1b[0m", c))
    .collect();

    execute!(
    io::stdout(),
    MoveTo(art_width as u16, info_y),
    Print(&top_line),
    MoveTo(art_width as u16, info_y + 1),
    Print(&bottom_line)
)?;
    info_y += 2;
}
    let max_y = y.max(info_y);
    execute!(
    io::stdout(),
    MoveTo(0, max_y as u16 + 1)
)?;

    if is_launched_by_explorer() {
    println!("\nPress Enter to exit...");
    let _ = std::io::stdin().read_line(&mut String::new());
}

    Ok(())  
}