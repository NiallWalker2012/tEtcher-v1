use std::fs;
use std::io::{self, Result, Write};
use std::path::PathBuf;
use crossterm::event;

#[cfg(target_os = "windows")]
fn list_external_devices() -> Result<Vec<PathBuf>> {
    let mut devices = Vec::new();
    for drive in 'A'..='Z' {
        let path = format!("{}:\\", drive);
        if fs::metadata(&path).is_ok() {
            devices.push(PathBuf::from(path));
        }
    }
    Ok(devices)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn list_external_devices() -> Result<Vec<PathBuf>> {
    #[cfg(target_os = "linux")]
    let mount_point = "/media";

    #[cfg(target_os = "macos")]
    let mount_point = "/Volumes";

    let mut devices = Vec::new();
    for entry in fs::read_dir(mount_point)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            devices.push(path);
        }
    }
    Ok(devices)
}

pub fn menu(_file_in: &PathBuf) -> Result<()> {
    print!("\x1B[H\x1B[2J");
    io::stdout().flush()?;

    let extdevs = list_external_devices()?; // now this is Vec<PathBuf>
    println!("External devices found:");
    for dev in &extdevs {
        println!("- {}", dev.display());
    }

    println!("Press any button to continue...");
    let _ = event::read();
    Ok(())
}
