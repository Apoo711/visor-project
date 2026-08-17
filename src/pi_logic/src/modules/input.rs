use std::{fs, path::Path, process::Command};

use log::{debug, error, info};

pub fn capture_frame(output_path: &str) -> Result<Vec<u8>, std::io::Error> {
    info!(
        "Capturing camera snapshot via rpicam-still to '{}'...",
        output_path
    );

    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }

    let status = Command::new("rpicam-still")
        .args(["-o", output_path, "--immediate", "-t", "1", "-n"])
        .status()
        .map_err(|e| {
            error!(
                "Failed to invoke 'rpicam-still'. Ensure the Raspberry Pi camera is enabled: {}",
                e
            );
            e
        })?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "rpicam-still exited with non-zero status: {:?}",
                status.code()
            ),
        ));
    }

    let bytes = fs::read(output_path)?;
    debug!("Captured {} bytes from camera snapshot.", bytes.len());
    Ok(bytes)
}
