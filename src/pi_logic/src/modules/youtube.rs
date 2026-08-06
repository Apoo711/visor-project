use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

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
    ) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
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
                return Ok(Some((watch_url, title)));
            }
        }

        Ok(None)
    }

    pub async fn display_video(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let target_url = if input.starts_with("http://") || input.starts_with("https://") {
            input.to_string()
        } else {
            format!("https://www.youtube.com/watch?v={}", input)
        };

        println!("Launching browser to play video: {}", target_url);

        let (mut browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .arg("--autoplay-policy=no-user-gesture-required")
                .arg("--no-sandbox")
                .build()?,
        )
        .await?;

        let handle = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let page = browser.new_page(&target_url).await?;

        println!("Waiting for video playback to complete...");
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;

            let js_check = r#"
                (function() {
                    const video = document.querySelector('video');
                    if (!video) return 'loading';
                    if (video.ended) return 'ended';
                    return 'playing';
                })()
            "#;

            match page.evaluate(js_check).await {
                Ok(value) => {
                    if let Some(status) = value.value().and_then(|v| v.as_str()) {
                        if status == "ended" {
                            println!("Video playback completed.");
                            break;
                        }
                    }
                }
                Err(_) => {
                    
                }
            }
        }

        println!("Waiting 15 seconds before closing the browser window...");
        tokio::time::sleep(Duration::from_secs(15)).await;

        println!("Closing browser...");
        browser.close().await?;
        let _ = handle.await;

        Ok(())
    }
}

