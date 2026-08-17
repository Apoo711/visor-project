mod modules;

use log::{error, info, warn};
use modules::{
    arduino::ArduinoBridge,
    audio::WakeWordDetector,
    gemini::GeminiClient,
    input::capture_frame,
    youtube::{DisplayManager, YouTubeClient},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("=========================================");
    info!("   Starting V.I.S.O.R. Core System       ");
    info!("   Visual Inspection & Smart Relief      ");
    info!("=========================================");

    let arduino_port = std::env::var("ARDUINO_PORT").unwrap_or_else(|_| "/dev/ttyAMA0".to_string());
    let arduino_baud: u32 = std::env::var("ARDUINO_BAUD")
        .ok()
        .and_then(|b| b.parse().ok())
        .unwrap_or(9600);

    let mut arduino = ArduinoBridge::new(&arduino_port, arduino_baud)?;

    let gemini_api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let ai_client = GeminiClient::new(gemini_api_key);

    let yt_api_key = std::env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY not set");
    let yt_client = YouTubeClient::new(yt_api_key);

    let standby_path = "assets/standby.html";
    let display_manager = DisplayManager::new(standby_path).await?;

    let wake_detector = WakeWordDetector::new()?;

    info!("V.I.S.O.R. is fully initialized and awaiting voice triggers.");

    loop {
        info!(">>> Awaiting wake word: 'VISOR help'...");
        wake_detector.wait_for_wake_word().await;

        info!(">>> Wake word detected! Initiating visual diagnosis...");

        match capture_frame("/tmp/visor_frame.jpg") {
            Ok(image_bytes) => {
                info!("Sending snapshot to Gemini 3.7 Flash API...");
                match ai_client.analyze_image(&image_bytes).await {
                    Ok(analysis) => {
                        info!("=== AI Assessment Received ===");
                        info!("Can Help: {}", analysis.can_help);
                        info!("Reasoning: {}", analysis.reasoning);
                        info!(
                            "Dispense Plan -> Bandage: {}, Alcohol Pad: {}, Gauze Pad: {}",
                            analysis.dispense.bandage,
                            analysis.dispense.alcohol_pad,
                            analysis.dispense.gauze_pad
                        );

                        if analysis.can_help {
                            if let Err(e) = arduino.send_dispense(
                                analysis.dispense.bandage,
                                analysis.dispense.alcohol_pad,
                                analysis.dispense.gauze_pad,
                            ) {
                                error!("Serial communication error to Arduino: {}", e);
                            } else if let Ok(ack) = arduino.read_response() {
                                if !ack.is_empty() {
                                    info!("Arduino Acknowledgment: {}", ack);
                                }
                            }

                            if let Some(query) = &analysis.video_search_query {
                                info!("Searching YouTube for query: '{}'", query);
                                match yt_client.fetch_top_video(query).await {
                                    Ok(Some((video_id, watch_url, title))) => {
                                        info!(
                                            "Instructional Video Found: '{}' ({})",
                                            title, watch_url
                                        );
                                        if let Err(e) = display_manager
                                            .play_video_and_return_to_standby(&video_id)
                                            .await
                                        {
                                            error!("Failed to display video in kiosk: {}", e);
                                            let _ = display_manager.show_standby().await;
                                        }
                                    }
                                    Ok(None) => {
                                        warn!("No instructional video found for query: '{}'", query)
                                    }
                                    Err(e) => error!("YouTube API query error: {}", e),
                                }
                            }
                        } else {
                            warn!(
                                "Condition cannot be treated with available supplies or requires emergency care."
                            );
                            let _ = arduino.send_hold();
                            if let Ok(ack) = arduino.read_response() {
                                if !ack.is_empty() {
                                    info!("Arduino Acknowledgment: {}", ack);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Gemini API analysis error: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Camera snapshot failed: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
