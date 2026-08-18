use std::{fs, path::Path, process::Command};

use log::{debug, error, info};

/// Ensures that the parent directory of a specified target file path exists.
///
/// If the parent directory does not exist, this function creates all required
/// intermediate directories in the path.
///
/// # Arguments
/// * `file_path` - The destination file path whose parent directory should be verified or created.
///
/// # Returns
/// * `std::io::Result<()>` - `Ok(())` if the directory exists or was successfully created,
///   or an `Err` containing the I/O failure.
pub fn ensure_parent_dir(file_path: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Captures a single image frame from the Raspberry Pi camera using the `rpicam-still` CLI.
///
/// The function invokes the camera hardware, saves the snapshot to the provided output path,
/// and reads the resulting JPEG bytes into memory for further processing.
///
/// # Arguments
/// * `output_path` - The file path where the captured frame should be written on disk.
///
/// # Returns
/// * `Result<Vec<u8>, std::io::Error>` - `Ok(Vec<u8>)` containing the captured raw image bytes,
///   or an `Err` if invoking the camera or reading the file fails.
pub fn capture_frame(output_path: &str) -> Result<Vec<u8>, std::io::Error> {
    info!(
        "Capturing camera snapshot via rpicam-still to '{}'...",
        output_path
    );

    ensure_parent_dir(output_path)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ensure_parent_dir_creates_directories() {
        let temp_dir = env::temp_dir().join("visor_test_dir_creation");
        let target_file = temp_dir.join("nested").join("frame.jpg");
        let target_str = target_file.to_str().unwrap();

        assert!(ensure_parent_dir(target_str).is_ok());
        assert!(target_file.parent().unwrap().exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ensure_parent_dir_handles_flat_filename() {
        assert!(ensure_parent_dir("simple_frame.jpg").is_ok());
    }
}
