use std::fs::File;
use std::io::{Read, Result, stdout, Write};
use std::process::Command;
use sha2::{Sha256, Digest};
use crossterm::style::Stylize;

/// Verifies that the ISO image was written correctly to a device by comparing SHA-256 hashes.
///
/// - Reads both files in 4MB chunks up to the size of the ISO.
/// - Returns `Ok(true)` if the hashes match, `Ok(false)` otherwise.
pub fn verify(iso_path: &str, device_path: &str) -> Result<bool> {
    const BS: usize = 4 * 1024 * 1024; // 4 MB buffer

    // Sync writes before verifying to ensure the device is up to date
    let _ = Command::new("sync").status();

    let iso_size = File::open(iso_path)?.metadata()?.len();
    let mut iso_file = File::open(iso_path)?;
    let mut dev_file = File::open(device_path)?;

    let mut iso_hash = Sha256::new();
    let mut dev_hash = Sha256::new();

    let mut buffer = vec![0u8; BS];
    let mut bytes_read: u64 = 0;

    println!("{}", "Verifying flashed image...".blue().bold());
    let mut stdout = stdout();

    while bytes_read < iso_size {
        let to_read = std::cmp::min(BS as u64, iso_size - bytes_read) as usize;

        // Read chunk from ISO and device
        let iso_bytes = iso_file.read(&mut buffer[..to_read])?;
        let dev_bytes = dev_file.read(&mut buffer[..to_read])?;

        if iso_bytes == 0 || dev_bytes == 0 {
            break;
        }

        // Update hashes
        iso_hash.update(&buffer[..iso_bytes]);
        dev_hash.update(&buffer[..dev_bytes]);

        bytes_read += iso_bytes as u64;

        // Print progress
        let percent = (bytes_read as f64 / iso_size as f64) * 100.0;
        print!("\rProgress: {:>6.2}%", percent);
        stdout.flush()?;
    }

    println!("\nCalculating hashes...");

    let iso_digest = iso_hash.finalize();
    let dev_digest = dev_hash.finalize();

    if iso_digest == dev_digest {
        println!("{}", "✅ Verification successful — hashes match!".green().bold());
        Ok(true)
    } else {
        println!("{}", "❌ Verification failed — hashes differ.".red().bold());
        Ok(false)
    }
}
