use std::fs::File;
use std::io::{
    Read,
    Write,
    Result,
    stdout
};
use std::time::Instant;


/// Flashes an ISO image to a raw device, showing percentage progress.
///
/// # Arguments
/// * `iso_path` - Path to the ISO file (e.g., "linux.iso").
/// * `device_path` - Path to the target device (e.g., "/dev/sdb").
///
/// # Notes
/// - Overwrites the device completely. Use with caution!
/// - Root/Administrator permissions are required

pub fn menu(iso: &str, device: &str) -> Result<()> {
    let _ = flash_iso(iso, device);
    Ok(())
}


fn flash_iso(iso_path: &str, device_path: &str) -> Result<()> {
    let bs: usize = 4; //Variable of bs is to adjust the block size (in MB)
    let buffer_size: usize = bs * 1024 * 1024;

    let mut iso_file = File::open(iso_path)?; //Opens ISO file for reading
    let mut device_file = File::create(device_path)?;

    //Get Size of the ISO file
    let iso_size = iso_file.metadata()?.len();
    let mut written: u64 = 0;

    let mut buffer = vec![0u8; buffer_size];
    let time = Instant::now();

    println!("Flashing {} to {}", iso_path, device_path);

    loop {
        let bytes_read = iso_file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        device_file.write_all(&buffer[..bytes_read])?;
        written += bytes_read as u64;

        //Print percentage of file written
        let percent_written = (written as f64 / iso_size as f64) * 100.0;
        print!("\rProgress: {}", percent_written);
        Write::flush(&mut stdout())?;
    }

    device_file.flush()?;
    let elapsed = time.elapsed();
    println!("\n Flash completed in {:.2?} seconds", elapsed);

    Ok(())
}