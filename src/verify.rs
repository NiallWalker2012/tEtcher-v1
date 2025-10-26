use std::fs::{File, OpenOptions};
use std::io::{Read, Result, stdout, Write};
use std::process::Command;
use sha2::{Sha256, Digest};
use crossterm::style::Stylize;

//
// --- Cross-Platform Device Open Helper ---
//

// On UNIX (Linux, macOS)
#[cfg(unix)]
fn open_device(path: &str) -> Result<File> {
    // macOS note: prefer /dev/rdiskX (raw) over /dev/diskX for speed
    OpenOptions::new().read(true).open(path)
}

// On Windows
#[cfg(windows)]
fn open_device(path: &str) -> Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    use winapi::um::winbase::FILE_FLAG_NO_BUFFERING;

    OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_NO_BUFFERING)
        .open(path)
}

//
// --- Cross-Platform Sync Helper ---
//

// Linux & macOS both have `sync`
#[cfg(unix)]
fn flush_system() {
    let _ = Command::new("sync").status();
}

// Windows: no `sync` command — rely on flushing and closing handles
#[cfg(windows)]
fn flush_system() {
    // Do nothing; Windows flushes on file close
}

//
// --- Verify Function ---
//

/// Verifies that the ISO image was written correctly to a device by comparing SHA-256 hashes.
/// Works on Linux, macOS, and Windows.
///
/// * Reads exactly `iso_size` bytes from the device.
/// * Prints progress and returns `Ok(true)` if hashes match.
pub fn verify(iso_path: &str, device_path: &str) -> Result<bool> {
    const BS: usize = 4 * 1024 * 1024; // 4 MB buffer

    // Flush any pending write buffers to disk
    flush_system();

    // Get ISO file size to know how many bytes to read from the device
    let iso_size = File::open(iso_path)?.metadata()?.len();

    let mut iso_file = File::open(iso_path)?;
    let mut dev_file = open_device(device_path)?;

    let mut iso_hash = Sha256::new();
    let mut dev_hash = Sha256::new();
    let mut buffer = vec![0u8; BS];

    let mut bytes_read: u64 = 0;
    let mut stdout = stdout();

    println!("{}", "Verifying flashed image...".blue().bold());

    while bytes_read < iso_size {
        let to_read = std::cmp::min(BS as u64, iso_size - bytes_read) as usize;

        // Read ISO and device chunks
        let iso_bytes = iso_file.read(&mut buffer[..to_read])?;
        let dev_bytes = dev_file.read(&mut buffer[..to_read])?;

        if iso_bytes == 0 || dev_bytes == 0 {
            break;
        }

        iso_hash.update(&buffer[..iso_bytes]);
        dev_hash.update(&buffer[..dev_bytes]);

        bytes_read += iso_bytes as u64;

        // Print progress percentage
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
