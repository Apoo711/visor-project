mod modules;

use modules::{
    arduino::ArduinoBridge, gemini::GeminiClient, input::capture_frame, youtube::YouTubeClient,
};

use log::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

    info!("Starting V.I.S.O.R. Core Process...");

    let mut arduino = ArduinoBridge::new("/dev/ttyAMA0", 9600)?;

    let gemini_api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let ai_client = GeminiClient::new(gemini_api_key);

    let yt_api_key = std::env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY not set");
    let yt_client = YouTubeClient::new(yt_api_key);

    loop {
        if let Ok(image_bytes) = capture_frame("/tmp/visor_frame.jpg") {
            match ai_client.analyze_image(&image_bytes).await {
                Ok(analysis) => {
                    debug!("=== AI Assessment ===");
                    debug!("Can Help: {}", analysis.can_help);
                    debug!("Reasoning: {}", analysis.reasoning);
                    debug!(
                        "Dispense Signals -> Bandage: {}, Alcohol Pad: {}, Gauze Pad: {}",
                        analysis.dispense.bandage,
                        analysis.dispense.alcohol_pad,
                        analysis.dispense.gauze_pad
                    );

                    if analysis.can_help {
                        let cmd_str = format!(
                            "{}{}{}\n",
                            analysis.dispense.bandage,
                            analysis.dispense.alcohol_pad,
                            analysis.dispense.gauze_pad
                        );
                        debug!(
                            "Transmitting binary dispense command to Arduino: {}",
                            cmd_str.trim()
                        );
                        if let Err(e) = arduino.send_bytes(cmd_str.as_bytes()) {
                            error!("Serial communication error: {}", e);
                        }

                        if let Some(query) = &analysis.video_search_query {
                            info!("Searching YouTube for instructional video: '{}'", query);
                            match yt_client.fetch_top_video(query).await {
                                Ok(Some((watch_url, title))) => {
                                    debug!(
                                        "Instructional Video Found: '{}' -> {}",
                                        title, watch_url
                                    );
                                    if let Err(e) = yt_client.display_video(&watch_url).await {
                                        error!("Failed to display video: {}", e);
                                    }
                                }
                                Ok(None) => debug!(
                                    "No instructional video found for query: '{}'",
                                    query
                                ),
                                Err(e) => debug!("YouTube API error: {}", e),
                            }
                        }
                    } else {
                        debug!(
                            "Condition cannot be treated with available supplies or requires emergency care."
                        );
                        let _ = arduino.send_bytes(b"000\n");
                    }
                }
                Err(e) => error!("API Analysis Error: {}", e),
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
