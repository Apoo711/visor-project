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
pub struct YouTubeSearchResponse {
    pub items: Vec<YouTubeItem>,
}

#[derive(Debug, Deserialize)]
pub struct YouTubeItem {
    pub id: VideoId,
    pub snippet: Snippet,
}

#[derive(Debug, Deserialize)]
pub struct VideoId {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Snippet {
    pub title: String,
}

pub fn format_embed_url(video_id: &str) -> String {
    format!(
        "https://www.youtube-nocookie.com/embed/{}?autoplay=1&controls=1&enablejsapi=1&rel=0&fs=1",
        video_id
    )
}

pub fn parse_youtube_search_response(
    response: YouTubeSearchResponse,
) -> Option<(String, String, String)> {
    if let Some(first_item) = response.items.into_iter().next() {
        if let Some(video_id) = first_item.id.video_id {
            let watch_url = format!("https://www.youtube.com/watch?v={}", video_id);
            let title = first_item.snippet.title;
            return Some((video_id, watch_url, title));
        }
    }
    None
}

pub fn resolve_standby_url(standby_file_path: &str) -> String {
    if Path::new(standby_file_path).exists() {
        if let Ok(abs_path) = std::fs::canonicalize(standby_file_path) {
            let path_str = abs_path.to_string_lossy().replace('\\', "/");
            let clean_path = path_str
                .trim_start_matches("//?/")
                .trim_start_matches(r"\\?\");
            return format!("file:///{}", clean_path.trim_start_matches('/'));
        }
    }
    "data:text/html,<html><body style='background:%23080c14;color:%23fff;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;'><h1 style='font-size:3rem;'>VISOR: Ready to Help</h1></body></html>".to_string()
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

        Ok(parse_youtube_search_response(res))
    }
}

pub struct DisplayManager {
    _browser: Browser,
    page: Page,
    standby_url: String,
}

impl DisplayManager {
    pub async fn new(standby_file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let standby_url = resolve_standby_url(standby_file_path);

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
        let embed_url = format_embed_url(video_id);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_embed_url() {
        let video_id = "dQw4w9WgXcQ";
        let url = format_embed_url(video_id);
        assert_eq!(
            url,
            "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?autoplay=1&controls=1&enablejsapi=1&rel=0&fs=1"
        );
    }

    #[test]
    fn test_parse_youtube_search_response_valid() {
        let json_data = r#"{
            "items": [
                {
                    "id": { "videoId": "abc123XYZ" },
                    "snippet": { "title": "How to Apply a Bandage" }
                }
            ]
        }"#;

        let res: YouTubeSearchResponse =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        let parsed = parse_youtube_search_response(res);
        assert!(parsed.is_some());
        let (id, watch, title) = parsed.unwrap();
        assert_eq!(id, "abc123XYZ");
        assert_eq!(watch, "https://www.youtube.com/watch?v=abc123XYZ");
        assert_eq!(title, "How to Apply a Bandage");
    }

    #[test]
    fn test_parse_youtube_search_response_empty_items() {
        let json_data = r#"{ "items": [] }"#;
        let res: YouTubeSearchResponse =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        let parsed = parse_youtube_search_response(res);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_youtube_search_response_missing_video_id() {
        let json_data = r#"{
            "items": [
                {
                    "id": { "videoId": null },
                    "snippet": { "title": "Channel or Playlist" }
                }
            ]
        }"#;
        let res: YouTubeSearchResponse =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        let parsed = parse_youtube_search_response(res);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_resolve_standby_url_fallback() {
        let url = resolve_standby_url("non_existent_file_path_123.html");
        assert!(url.starts_with("data:text/html,"));
        assert!(url.contains("VISOR: Ready to Help"));
    }
}
