#![allow(dead_code)]

use std::fs;
use std::io::{self, Result, Write, stdout};
use std::path::PathBuf;
use crossterm::terminal::disable_raw_mode;
use crossterm::{
    execute,
    cursor,
    style::Stylize,
    terminal::{
        self,
        ClearType,
    },
    event::{
        self,
        KeyCode,
        Event
    },
};
use std::process::Command;
use crate::flash;

fn list_flashable_drives_windows() -> Vec<String> {
    let mut drives = Vec::new();

    let output = Command::new("powershell")
    .args(["-Command", "Get-CimInstance Win32_DiskDrive | Where-Object { $_.MediaType -eq 'Removable Media' } | Select-Object -ExpandProperty DeviceID"])
    .output()
    .expect("failed to run PowerShell command");

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\\\\.\\") {
            drives.push(trimmed.to_string());
        }
    }

    drives
}

fn list_flashable_drives_macos() -> Vec<String> {
    let mut drives = Vec::new();

    let output = Command::new("diskutil")
        .arg("list")
        .output()
        .expect("failed to run diskutil");

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.contains("external, physical") {
            if let Some(disk_name) = line.split_whitespace().next() {
                drives.push(disk_name.to_string());
            }
        }
    }

    drives
}

fn list_flashable_drives_linux() -> Result<Vec<String>> {
    let mut drives = Vec::new();


    for entry in fs::read_dir("/sys/block")? {
        let entry = entry?;
        let dev_name = entry.file_name();
        let path = format!("/sys/block/{}/removable", dev_name.to_string_lossy());
        if let Ok(contents) = fs::read_to_string(&path) {
            if contents.trim() == "1" {
                // This is a removable device (like a USB)
                drives.push(format!("/dev/{}", dev_name.to_string_lossy()));
            }
        }
    }
    Ok(drives)
}


pub fn menu(file_in: &PathBuf) -> Result<()> {
    print!("\x1B[H\x1B[2J");
    io::stdout().flush()?;

    let mut extselected = 0;
    let mut stdout = stdout();

    let extdevs: Vec<String>;

    #[cfg(target_os = "windows")] {
        extdevs = list_flashable_drives_windows();

        if extdevs.is_empty() {
            println!("No removable drives detected.");
            println!("Insert a USB drive and restart the program.");
            return Ok(());
        }
    }

    #[cfg(target_os = "macos")] {
        extdevs = list_flashable_drives_macos();

        if extdevs.is_empty() {
        println!("No removable drives detected.");
        println!("Insert a USB drive and restart the program.");
        return Ok(());
    }
}
   
    #[cfg(target_os = "linux")] {
    extdevs = list_flashable_drives_linux()?;

    if extdevs.is_empty() {
        println!("No removable drives detected.");
        println!("Insert a USB drive and restart the program.");
        return Ok(());
    }

    let verify = ["yes", "no"];
}


    
    loop {
        println!("External devices found:");
        
        for (i, item) in extdevs.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 2) as u16))?;
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
            if i == extselected {
                print!("  {}", item.clone().on_white().black());
            } else {
                print!("  {}", item);
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
                        println!("Do you want to flash {} to {}?", file_in.to_string_lossy(), selected_device);
                        for (i, confitem) in conf.iter().enumerate() {
                            execute!(stdout, cursor::MoveTo(0, (i + 2) as u16))?;
                            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;
                            if i == confselected.try_into().unwrap() {
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
                                KeyCode::Enter => {
                                    match confselected {
                                        0 => {
                                             // Clear menu UI
                                            execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0,0))?;
                                            
                                            // Run flashing directly here so the menu doesn’t redraw until done
                                            flash::menu(&file_in.to_string_lossy(), &selected_device)?;

                                            disable_raw_mode()?;
                                            execute!(stdout, cursor::Show)?;
                                            exit(0); // Exits program once finished
                                        }
                                        1 => {
                                            exit(0);
                                        }
                                        _ => {}
                                    }
                                }
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
