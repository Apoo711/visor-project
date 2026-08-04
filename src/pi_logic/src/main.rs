use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    // 1. Define port settings matching the Arduino configuration
    let port_path = "/dev/ttyACM0"; 
    let baud_rate = 9600;

    println!("Opening serial port: {} at {} baud...", port_path, baud_rate);

    // 2. Open the serial stream
    let mut port = tokio_serial::new(port_path, baud_rate)
        .timeout(Duration::from_secs(2))
        .open_native_async()?;

    // 3. Send a command to turn the Arduino LED ON ('1')
    println!("Sending command: '1' (Turn LED ON)");
    port.write_all(b"1").await?;

    // 4. Read the text response back from the Arduino
    let mut buffer = vec![0; 32];
    match port.read(&mut buffer).await {
        Ok(bytes_read) => {
            let response = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Arduino response: {}", response.trim());
        }
        Err(e) => eprintln!("Failed to read response: {}", e),
    }

    Ok(())
}
