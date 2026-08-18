use std::{fs, path::Path, process::Command};

use log::{debug, error, info};

pub fn ensure_parent_dir(file_path: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

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
