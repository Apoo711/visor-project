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

/// Formats a clean, privacy-enhanced YouTube embed URL configured for kiosk playback.
///
/// Configures parameters for autoplay, controls, JavaScript API integration, and fullscreen support.
///
/// # Arguments
/// * `video_id` - The alphanumeric YouTube video identifier string.
///
/// # Returns
/// * `String` - Fully qualified embed URL pointing to `youtube-nocookie.com`.
pub fn format_embed_url(video_id: &str) -> String {
    format!(
        "https://www.youtube-nocookie.com/embed/{}?autoplay=1&controls=1&enablejsapi=1&rel=0&fs=1",
        video_id
    )
}

/// Parses the first valid video search result from a YouTube Data API v3 search response.
///
/// Extracts the video ID, standard watch URL, and snippet title if available.
///
/// # Arguments
/// * `response` - Deserialized YouTube search response structure.
///
/// # Returns
/// * `Option<(String, String, String)>` - `Some((video_id, watch_url, title))` if a video is found, or `None` if the items list is empty or lacks a video ID.
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

/// Resolves the standby interface location into a valid browser URL.
///
/// If a local file exists at `standby_file_path`, canonicalizes it into a `file:///` URI.
/// Otherwise, returns an inline HTML data URI fallback displaying the standby interface.
///
/// # Arguments
/// * `standby_file_path` - Relative or absolute path to the local standby HTML asset.
///
/// # Returns
/// * `String` - Browser-navigable URL (either `file:///...` or `data:text/html,...`).
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

/// Client for interacting with the YouTube Data API v3 to search for first-aid instruction videos.
pub struct YouTubeClient {
    client: Client,
    api_key: String,
}

impl YouTubeClient {
    /// Creates a new `YouTubeClient` instance.
    ///
    /// # Arguments
    /// * `api_key` - Google Cloud YouTube Data API key.
    ///
    /// # Returns
    /// * `Self` - Initialized `YouTubeClient`.
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Queries the YouTube Data API for the most relevant embeddable instructional video matching the query.
    ///
    /// # Arguments
    /// * `query` - Search terms (e.g., "how to apply bandage to cut").
    ///
    /// # Returns
    /// * `Result<Option<(String, String, String)>, Box<dyn std::error::Error>>` - `Ok(Some((video_id, watch_url, title)))` on match,
    ///   `Ok(None)` if no video matched, or an `Err` on HTTP/API failure.
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

/// Controls the Chromium fullscreen kiosk display for showing standby screens and first-aid videos.
pub struct DisplayManager {
    _browser: Browser,
    page: Page,
    standby_url: String,
}

impl DisplayManager {
    /// Launches an automated Chromium headless/kiosk browser instance navigating to the standby screen.
    ///
    /// # Arguments
    /// * `standby_file_path` - Path to the local standby UI HTML file.
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn std::error::Error>>` - Managed browser display instance on success, or launch error on failure.
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

    /// Navigates the kiosk display back to the idle "VISOR Ready to Help" standby UI screen.
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok on successful page navigation.
    pub async fn show_standby(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Returning display to 'VISOR Ready to Help' standby screen...");
        self.page.goto(&self.standby_url).await?;
        Ok(())
    }

    /// Navigates the kiosk display to the given YouTube video, monitors playback until the video finishes, and returns to the standby screen.
    ///
    /// Polls the HTML5 `<video>` element status inside the browser context, waiting for either the
    /// `ended` event or a maximum timeout (5 minutes) before safely restoring the standby display.
    ///
    /// # Arguments
    /// * `video_id` - Alphanumeric YouTube video identifier to play.
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok on successful completion of playback and return to standby.
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
