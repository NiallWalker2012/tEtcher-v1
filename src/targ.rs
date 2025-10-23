#![allow(dead_code)]

use std::fs;
use std::io::{self, Result, Write, stdout};
use std::path::PathBuf;
use crossterm::terminal::disable_raw_mode;
use crossterm::{
    execute,
    cursor,
    style::Stylize,
    terminal::{self, ClearType},
    event::{self, KeyCode, Event},
};
use std::process::Command;
use crate::flash;

/// targ.rs will list all external drives and their model names to select from
/// 
/// For example, on windows it might list: "\\.\PHYSICALDRIVE1 - SanDisk Ultra USB 64G"
/// 
/// If there are no external devices found, it will print "No removeable drives found
///                                                        Please plug in a USB and restart the program"


/// Unified structure for displaying drives
#[derive(Debug, Clone)]
struct DriveInfo {
    path: String,
    model: Option<String>,
}

/// Windows: list removable drives with model names
fn list_flashable_drives_windows() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    // PowerShell query gets both DeviceID and Model
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-CimInstance Win32_DiskDrive | Where-Object { $_.MediaType -eq 'Removable Media' -or $_.InterfaceType -eq 'USB' } | Select-Object DeviceID, Model",
        ])
        .output()
        .expect("failed to run PowerShell command");

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        // Skip headers or blank lines
        if line.trim().is_empty() || line.contains("DeviceID") {
            continue;
        }

        // PowerShell columns can be space-padded; split roughly in half
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let device_id = parts[0].trim().to_string();
            let model = parts[1..].join(" ");
            drives.push(DriveInfo {
                path: device_id,
                model: Some(model),
            });
        } else if parts.len() == 1 && parts[0].starts_with("\\\\.\\") {
            drives.push(DriveInfo {
                path: parts[0].trim().to_string(),
                model: None,
            });
        }
    }

    drives
}

/// macOS: list external drives with model names
fn list_flashable_drives_macos() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    // List all disks
    let output = Command::new("diskutil")
        .arg("list")
        .output()
        .expect("failed to run diskutil");

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.contains("external, physical") {
            if let Some(disk_name) = line.split_whitespace().next() {
                let path = format!("/dev/{}", disk_name);

                // Query model using `diskutil info`
                let info_output = Command::new("diskutil")
                    .args(["info", &path])
                    .output()
                    .unwrap();

                let info_text = String::from_utf8_lossy(&info_output.stdout);
                let mut model = None;

                for infoline in info_text.lines() {
                    if infoline.contains("Device / Media Name:") {
                        model = Some(
                            infoline
                                .split(':')
                                .nth(1)
                                .unwrap_or("")
                                .trim()
                                .to_string(),
                        );
                        break;
                    }
                }

                drives.push(DriveInfo { path, model });
            }
        }
    }

    drives
}

/// Linux: list removable drives with model names
fn list_flashable_drives_linux() -> Result<Vec<DriveInfo>> {
    let mut drives = Vec::new();

    for entry in fs::read_dir("/sys/block")? {
        let entry = entry?;
        let dev_name = entry.file_name();
        let dev_str = dev_name.to_string_lossy();
        let removable_path = format!("/sys/block/{}/removable", dev_str);

        if let Ok(contents) = fs::read_to_string(&removable_path) {
            if contents.trim() == "1" {
                let model_path = format!("/sys/block/{}/device/model", dev_str);
                let model = fs::read_to_string(&model_path)
                    .ok()
                    .map(|s| s.trim().to_string());
                let dev_path = format!("/dev/{}", dev_str);
                if fs::metadata(&dev_path).is_ok() {
                    drives.push(DriveInfo {
                        path: dev_path,
                        model,
                    });
                }
            }
        }
    }
    Ok(drives)
}

/// Menu UI for selecting which drive to flash to
pub fn menu(file_in: &PathBuf) -> Result<()> {
    print!("\x1B[H\x1B[2J");
    io::stdout().flush()?;

    let mut extselected = 0;
    let mut stdout = stdout();

    let extdevs: Vec<DriveInfo>;

    #[cfg(target_os = "windows")]
    {
        extdevs = list_flashable_drives_windows();
        if extdevs.is_empty() {
            println!("No removable drives detected.");
            println!("Insert a USB drive and restart the program.");
            return Ok(());
        }
    }

    #[cfg(target_os = "macos")]
    {
        extdevs = list_flashable_drives_macos();
        if extdevs.is_empty() {
            println!("No removable drives detected.");
            println!("Insert a USB drive and restart the program.");
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    {
        extdevs = list_flashable_drives_linux()?;
        if extdevs.is_empty() {
            println!("No removable drives detected.");
            println!("Insert a USB drive and restart the program.");
            return Ok(());
        }
    }

    loop {
        println!("External devices found:");

        for (i, item) in extdevs.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 2) as u16))?;
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;

            let label = if let Some(model) = &item.model {
                format!("{} — {}", item.path, model)
            } else {
                item.path.clone()
            };

            if i == extselected {
                print!("  {}", label.on_white().black());
            } else {
                print!("  {}", label);
            }
        }

        stdout.flush()?;

        if let Event::Key(ev) = event::read()? {
            match ev.code {
                KeyCode::Up => {
                    if extselected > 0 {
                        extselected -= 1;
                    }
                }
                KeyCode::Down => {
                    if extselected < extdevs.len() - 1 {
                        extselected += 1;
                    }
                }
                KeyCode::Enter => {
                    let mut confselected: usize = 0;
                    let conf: Vec<&str> = vec!["Yes", "No"];
                    let selected_device = extdevs[extselected].clone();

                    loop {
                        print!("\x1B[H\x1B[2J");
                        println!(
                            "Do you want to flash {} to {}?",
                            file_in.to_string_lossy(),
                            selected_device.path
                        );
                        for (i, confitem) in conf.iter().enumerate() {
                            execute!(stdout, cursor::MoveTo(0, (i + 2) as u16))?;
                            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
                            if i == confselected {
                                print!("{}", confitem.on_white().black());
                            } else {
                                print!("{}", confitem);
                            }
                        }

                        stdout.flush()?;

                        use std::process::exit;

                        if let Event::Key(confev) = event::read()? {
                            match confev.code {
                                KeyCode::Up => {
                                    if confselected > 0 {
                                        confselected -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if confselected < conf.len() - 1 {
                                        confselected += 1;
                                    }
                                }
                                KeyCode::Enter => match confselected {
                                    0 => {
                                        execute!(
                                            stdout,
                                            terminal::Clear(ClearType::All),
                                            cursor::MoveTo(0, 0)
                                        )?;
                                        flash::menu(&file_in.to_string_lossy(), &selected_device.path)?;
                                        disable_raw_mode()?;
                                        execute!(stdout, cursor::Show)?;
                                        std::process::exit(0);

                                    }
                                    1 => exit(0),
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
