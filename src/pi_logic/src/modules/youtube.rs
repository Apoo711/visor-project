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
}
