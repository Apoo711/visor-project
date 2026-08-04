mod modules;

use modules::{
    arduino::ArduinoBridge, gemini::GeminiClient, input::capture_frame, youtube::YouTubeClient,
};

async fn openLink(link: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening video {} ...", link);

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting V.I.S.O.R. Core Process...");

    let mut arduino = ArduinoBridge::new("/dev/ttyAMA0", 9600)?;

    let gemini_api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let ai_client = GeminiClient::new(gemini_api_key);

    let yt_api_key = std::env::var("YOUTUBE_API_KEY")
        .or_else(|_| std::env::var("GEMINI_API_KEY"))
        .ok();
    let yt_client = yt_api_key.map(YouTubeClient::new);

    loop {
        if let Ok(image_bytes) = capture_frame("/tmp/visor_frame.jpg") {
            match ai_client.analyze_image(&image_bytes).await {
                Ok(analysis) => {
                    println!("=== AI Assessment ===");
                    println!("Can Help: {}", analysis.can_help);
                    println!("Reasoning: {}", analysis.reasoning);
                    println!(
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
                        println!(
                            "Transmitting binary dispense command to Arduino: {}",
                            cmd_str.trim()
                        );
                        if let Err(e) = arduino.send_bytes(cmd_str.as_bytes()) {
                            eprintln!("Serial communication error: {}", e);
                        }

                        if let Some(query) = &analysis.video_search_query {
                            if let Some(yt) = &yt_client {
                                println!("Searching YouTube for instructional video: '{}'", query);
                                match yt.fetch_top_video(query).await {
                                    Ok(Some((watch_url, title))) => {
                                        println!(
                                            "Instructional Video Found: '{}' -> {}",
                                            title, watch_url
                                        );
                                    }
                                    Ok(None) => println!(
                                        "No instructional video found for query: '{}'",
                                        query
                                    ),
                                    Err(e) => eprintln!("YouTube API error: {}", e),
                                }
                            }
                        }
                    } else {
                        println!(
                            "Condition cannot be treated with available supplies or requires emergency care."
                        );
                        let _ = arduino.send_bytes(b"000\n");
                    }
                }
                Err(e) => eprintln!("API Analysis Error: {}", e),
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
