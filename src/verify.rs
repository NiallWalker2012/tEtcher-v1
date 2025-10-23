use std::io::{
    Read,
    Result
};
use std::fs::{
    File
};

/// This function will read the the contents of the ISO file and the bootable external drive
/// 
/// It takes the arguments, ISO Path and Device Path from main.rs and targ.rs
/// 
/// Verifying is completely optional but recommended to prevent corrupted OS installations 
/// 
/// Like flash.rs, verify.rs automatically uses a block size of 4M, which can be altered through the 'bs' variable

pub fn verify(iso_path: &str, device_path: &str) -> Result<bool> {
    const BS: usize = 4;
    //Automatic buffer size is blocksize in megabytes
    const BUFFER_SIZE: usize = BS * 1024 * 1024;

    //Opens the ISO file and Device file for reading
    let mut iso_file = File::open(iso_path)?;
    let mut device_file = File::open(device_path)?;

    //Sets the buffer (how much the program will read at once) for the ISO and external device
    let mut iso_buffer = vec![0u8; BUFFER_SIZE];
    let mut device_buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let iso_contents = iso_file.read(&mut iso_buffer)?;
        let device_contents = device_file.read(&mut device_buffer)?;

        if iso_contents == 0 && device_contents == 0 {
            break;
        }

        if iso_contents != device_contents || iso_buffer[..iso_contents] != device_buffer[..device_contents] {
            return Ok(false)
        }
    }
    Ok(true)

}