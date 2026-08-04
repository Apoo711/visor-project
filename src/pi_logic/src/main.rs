mod modules;


use modules::{arduino::ArduinoBridge, gemini::GeminiClient, input::capture_frame};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting V.I.S.O.R. Core Process...");

    let mut arduino = ArduinoBridge::new("/dev/ttyAMA0", 9600)?;

    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let ai_client = GeminiClient::new(api_key);

    loop {
        if let Ok(image_bytes) = capture_frame("/tmp/visor_frame.jpg") {

            match ai_client.analyze_image(&image_bytes).await {
                Ok(command) => {
                    println!("AI Result: {}", command);

                    if command.contains("OPEN") {
                        let _ = arduino.send_command(b'O');
                    } else if command.contains("CLOSE") {
                        let _ = arduino.send_command(b'C');
                    }
                }
                Err(e) => eprintln!("API Error: {}", e),
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
