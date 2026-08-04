use std::time::Duration;

use arduino::ArduinoBridge;
use gemini::GeminiClient;
use input::capture_frame;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting V.I.S.O.R. Core Process...");

    // 1. Initialize Serial link to Arduino
    let mut arduino = ArduinoBridge::new("/dev/ttyAMA0", 9600)?;

    // 2. Initialize API Client
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let ai_client = GeminiClient::new(api_key);

    loop {
        // Step 1: Capture frame
        if let Ok(image_bytes) = capture_frame("/tmp/visor_frame.jpg") {
            // Step 2: Request evaluation from Gemini
            match ai_client.analyze_image(&image_bytes).await {
                Ok(command) => {
                    println!("AI Result: {}", command);

                    // Step 3: Trigger Arduino servos based on response
                    if command.contains("OPEN") {
                        let _ = arduino.send_command(b'O');
                    } else if command.contains("CLOSE") {
                        let _ = arduino.send_command(b'C');
                    }
                }
                Err(e) => eprintln!("API Error: {}", e),
            }
        }

        // Delay loop execution to prevent spamming
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
