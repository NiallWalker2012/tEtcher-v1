use std::fs;
use std::io::{self, Result, Write};
use std::path::PathBuf;
use crossterm::{
    event,
};

#[cfg(target_os = "windows")]
fn list_external_devices() -> Result<()> {
    for drive in 'A'..='Z' {
        let path = format!("{}:\\", drive);
        if fs::metadata(&path).is_ok() {
            println!("Found drive: {}", path);
        }
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn list_external_devices() -> Result<()> {
    #[cfg(target_os = "linux")]
    let mount_point = "/media";

    #[cfg(target_os = "macos")]
    let mount_point = "/Volumes";

    println!("External devices:");
    for entry in fs::read_dir(mount_point)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            println!("- {}", path.display());
        }
    }
    Ok(())
}

pub fn menu(_file_in: &PathBuf) -> Result<()> {
    print!("\x1B[H\x1B[2J");
    io::stdout().flush()?;

    let _ = list_external_devices();
    println!("Press any button to continue...");
    let _ = event::read();
    Ok(())
}
