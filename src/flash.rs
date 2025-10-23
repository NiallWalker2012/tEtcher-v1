use std::fs::{
    File,
    OpenOptions
};
use crossterm::{
    execute,
    terminal::{
        self,
        ClearType
    },
    cursor,
    style::{
        Color,
        Stylize
    },
    event::{
        self,
        KeyCode,
        Event
    },
};
use std::io::{
    Read,
    Write,
    Result,
    stdout
};
use std::time::Instant;
use crate::verify;

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
    
    let mut stdout = stdout();

    let verify: Vec<&str> = vec!["Yes", "No"];
    let mut selected = 0;

    loop {
        // Adjust selection if needed
        if selected >= verify.len() {
            selected = verify.len().saturating_sub(1);
        }

        // Draw UI
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::FromCursorDown)
        )?;
        println!("{}", "Please navigate to the file you wish to flash".with(Color::Blue));

        for (i, item) in verify.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16))?;
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;

            if i == selected {
                print!("{}", item.on_white().black());
            } else {
                print!("{}", item);
            }
        }
        stdout.flush()?;

        if let Event::Key(event) = event::read()? {
            match event.code {
                //Move selected item up when Up-arrow is pressed
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                //Move selected item down when down-arrow is pressed
                KeyCode::Down => {
                    if selected < verify.len().saturating_sub(1) { // .len() and .saturating_sub(1) checks that the selected item is not greater than menu items
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    match selected {
                        0 => verify::verify(&iso, &device),
                        1 => break,
                        _ => break,
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}


fn flash_iso(iso_path: &str, device_path: &str) -> Result<()> {
    println!("\x1B[H\x1B[2J");

    let bs: usize = 4; //Variable of bs is to adjust the block size (in MB)
    let buffer_size: usize = bs * 1024 * 1024;

    let mut iso_file = File::open(iso_path)?; //Opens ISO file for reading
    let mut device_file = OpenOptions::new()
        .write(true)
        .open(device_path)?;

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
        print!("\rProgress: {:>6.2}%", percent_written);
        Write::flush(&mut stdout())?;
    }

    device_file.flush()?;
    let elapsed = time.elapsed();
    println!("\n Flash completed in {:.2?} seconds", elapsed);
    
    Ok(())
}
