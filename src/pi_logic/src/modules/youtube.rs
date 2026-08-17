use std::{path::Path, time::Duration};

use chromiumoxide::{
    Page,
    browser::{Browser, BrowserConfig},
};
use futures_util::StreamExt;
use log::{debug, info, warn};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct YouTubeSearchResponse {
    items: Vec<YouTubeItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeItem {
    id: VideoId,
    snippet: Snippet,
}

#[derive(Debug, Deserialize)]
struct VideoId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Snippet {
    title: String,
}

pub struct YouTubeClient {
    client: Client,
    api_key: String,
}

impl YouTubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn fetch_top_video(
        &self,
        query: &str,
    ) -> Result<Option<(String, String, String)>, Box<dyn std::error::Error>> {
        let url = "https://www.googleapis.com/youtube/v3/search";

        let res: YouTubeSearchResponse = self
            .client
            .get(url)
            .query(&[
                ("part", "snippet"),
                ("type", "video"),
                ("videoEmbeddable", "true"),
                ("maxResults", "1"),
                ("q", query),
                ("key", &self.api_key),
            ])
            .send()
            .await?
            .json()
            .await?;

        if let Some(first_item) = res.items.first() {
            if let Some(video_id) = &first_item.id.video_id {
                let watch_url = format!("https://www.youtube.com/watch?v={}", video_id);
                let title = first_item.snippet.title.clone();
                return Ok(Some((video_id.clone(), watch_url, title)));
            }
        }

        Ok(None)
    }
}

pub struct DisplayManager {
    _browser: Browser,
    page: Page,
    standby_url: String,
}

impl DisplayManager {
    pub async fn new(standby_file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let standby_url = if Path::new(standby_file_path).exists() {
            let abs_path = std::fs::canonicalize(standby_file_path)?;
            let path_str = abs_path.to_string_lossy().replace('\\', "/");
            let clean_path = path_str
                .trim_start_matches("//?/")
                .trim_start_matches(r"\\?\");
            format!("file:///{}", clean_path.trim_start_matches('/'))
        } else {
            warn!(
                "Standby file '{}' not found. Falling back to data URI.",
                standby_file_path
            );
            "data:text/html,<html><body style='background:%23080c14;color:%23fff;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;'><h1 style='font-size:3rem;'>VISOR: Ready to Help</h1></body></html>".to_string()
        };

        info!(
            "Launching Chromium Kiosk Display pointing to: {}",
            standby_url
        );

        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .arg("--kiosk")
                .arg("--autoplay-policy=no-user-gesture-required")
                .arg("--no-sandbox")
                .arg("--disable-infobars")
                .arg("--check-for-update-interval=31536000")
                .arg("--disable-session-crashed-bubble")
                .build()?,
        )
        .await?;

        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let page = browser.new_page(&standby_url).await?;
        info!("Chromium Kiosk ready and displaying standby interface.");

        Ok(Self {
            _browser: browser,
            page,
            standby_url,
        })
    }

    pub async fn show_standby(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Returning display to 'VISOR Ready to Help' standby screen...");
        self.page.goto(&self.standby_url).await?;
        Ok(())
    }

    pub async fn play_video_and_return_to_standby(
        &self,
        video_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let embed_url = format!(
            "https://www.youtube-nocookie.com/embed/{}?autoplay=1&controls=1&enablejsapi=1&rel=0&fs=1",
            video_id
        );

        info!("Navigating kiosk display to video: {}", embed_url);
        self.page.goto(&embed_url).await?;

        debug!("Waiting for video playback to complete...");
        let check_interval = Duration::from_secs(2);
        let max_duration = Duration::from_secs(300);
        let start_time = tokio::time::Instant::now();

        loop {
            tokio::time::sleep(check_interval).await;

            if start_time.elapsed() > max_duration {
                warn!("Max video playback duration reached. Returning to standby.");
                break;
            }

            let js_check = r#"
                (function() {
                    const video = document.querySelector('video');
                    if (!video) return 'loading';
                    if (video.ended) return 'ended';
                    return 'playing';
                })()
            "#;

            match self.page.evaluate(js_check).await {
                Ok(value) => {
                    if let Some(status) = value.value().and_then(|v| v.as_str()) {
                        debug!("Current video status: {}", status);
                        if status == "ended" {
                            info!("Video playback has completed.");
                            break;
                        }
                    }
                }
                Err(e) => {
                    debug!("Error evaluating video status: {}", e);
                }
            }
        }

        info!("Video finished. Waiting 5 seconds before returning to standby screen...");
        tokio::time::sleep(Duration::from_secs(5)).await;

        self.show_standby().await?;

        Ok(())
    }
}
