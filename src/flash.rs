use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Result, stdout};
use std::process::exit;
use std::time::Instant;
use crossterm::terminal::disable_raw_mode;
use crossterm::{
    execute,
    terminal::{self, ClearType},
    cursor,
    style::{Stylize},
    event::{self, Event, KeyCode},
};

use crate::verify;

pub fn menu(iso: &str, device: &str) -> Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();

    let warn = vec!["Yes", "No"];
    let mut selected = 0;

    loop {
        execute!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::FromCursorDown))?;
        println!("{}", "Do you wish to flash the ISO?".blue().bold());

        for (i, item) in warn.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16))?;
            if i == selected {
                print!("{}", item.on_white().black());
            } else {
                print!("{}", item);
            }
        }

        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => if selected > 0 { selected -= 1; },
                KeyCode::Down => if selected < warn.len() - 1 { selected += 1; },
                KeyCode::Enter => {
                    match selected {
                        0 => {
                            flash_iso(iso, device)?;
                            verify_menu(iso, device)?;
                        }
                        1 => break,
                        _ => {}
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

fn verify_menu(iso: &str, device: &str) -> Result<()> {
    let mut stdout = stdout();
    let verify_opts = vec!["Yes", "No"];
    let mut verselected = 0;

    loop {
        execute!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::FromCursorDown))?;
        println!("{}", "Do you wish to verify the ISO?".blue().bold());

        for (i, item) in verify_opts.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16))?;
            if i == verselected {
                print!("{}", item.on_white().black());
            } else {
                print!("{}", item);
            }
        }

        stdout.flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => if verselected > 0 { verselected -= 1; },
                KeyCode::Down => if verselected < verify_opts.len() - 1 { verselected += 1; },
                KeyCode::Enter => {
                    if verselected == 0 {
                        println!("\x1B[H\x1B[2J");
                        println!("Verifying...");
                        let is_verified = verify::verify(iso, device)?;
                        if is_verified {
                            println!("Verification succeeded!");
                        } else {
                            println!("Verification failed!");
                        }
                    }
                    disable_raw_mode()?;
                    execute!(stdout, cursor::Show)?;
                    exit(0);
                }
                _ => {}
            }
        }
    }
}

fn flash_iso(iso_path: &str, device_path: &str) -> Result<()> {
    println!("\x1B[H\x1B[2J");

    let bs: usize = 4 * 1024 * 1024; // 4 MB buffer
    let mut iso_file = File::open(iso_path)?;
    let mut device_file = OpenOptions::new().write(true).open(device_path)?;

    let iso_size = iso_file.metadata()?.len();
    let mut written: u64 = 0;
    let mut buffer = vec![0u8; bs];
    let start = Instant::now();

    println!("Flashing {} → {}", iso_path, device_path);

    loop {
        let bytes_read = iso_file.read(&mut buffer)?;
        if bytes_read == 0 { break; }

        device_file.write_all(&buffer[..bytes_read])?;
        written += bytes_read as u64;

        let percent = (written as f64 / iso_size as f64) * 100.0;
        print!("\rProgress: {:>6.2}%", percent);
        stdout().flush()?;
    }

    device_file.flush()?;
    println!("\nCompleted in {:.2?}", start.elapsed());
    return Ok(())
}
