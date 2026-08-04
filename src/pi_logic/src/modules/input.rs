use std::process::Command;
use std::fs;

pub fn capture_frame(output_path: &str) -> Result<Vec<u8>, std::io::Error> {
    // Calls native Pi camera stack to output a temporary JPEG
    Command::new("rpicam-still")
        .args(["-o", output_path, "--immediate", "-t", "1"])
        .status()?;

    let bytes = fs::read(output_path)?;
    Ok(bytes)
}