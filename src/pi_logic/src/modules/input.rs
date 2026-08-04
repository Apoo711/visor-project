use std::{fs, process::Command};

pub fn capture_frame(output_path: &str) -> Result<Vec<u8>, std::io::Error> {
    Command::new("rpicam-still")
        .args(["-o", output_path, "--immediate", "-t", "1"])
        .status()?;

    let bytes = fs::read(output_path)?;
    Ok(bytes)
}
