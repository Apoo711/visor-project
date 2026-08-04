use reqwest::Client;
use serde_json::json;

pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn analyze_image(&self, image_bytes: &[u8]) -> Result<String, reqwest::Error> {
        let base64_image = base64::encode(image_bytes);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            self.api_key
        );

        let body = json!({
            "contents": [{
                "parts": [
                    { "text": "Analyze this image for first aid needs. Reply with short command code: OPEN, CLOSE, or HOLD." },
                    {
                        "inline_data": {
                            "mime_type": "image/jpeg",
                            "data": base64_image
                        }
                    }
                ]
            }]
        });

        let res = self.client.post(&url)
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Extract returned text
        let response_text = res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("HOLD")
            .to_string();

        Ok(response_text)
    }
}