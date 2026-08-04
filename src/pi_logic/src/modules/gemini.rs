use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispenseItems {
    pub bandage: u8,
    pub alcohol_pad: u8,
    pub gauze_pad: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisorAnalysis {
    pub can_help: bool,
    pub reasoning: String,
    pub dispense: DispenseItems,
    pub video_search_query: Option<String>,
}

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

    pub async fn analyze_image(&self, image_bytes: &[u8]) -> Result<VisorAnalysis, Box<dyn std::error::Error>> {
        let base64_string = general_purpose::STANDARD.encode(image_bytes);
        let url = "https://generativelanguage.googleapis.com/v1beta/interactions";

        let prompt_text = "Analyze this image to evaluate the user's first-aid needs. \
            Available resources for dispensing: Bandage (Normal Size), Alcohol Pad, Gauze Pad. \
            Determine if the user has a minor condition that can be treated using ONLY the available supplies. \
            If the user requires immediate emergency care, severe medical attention, or if no first-aid help is needed, set can_help to false.";

        let body = json!({
            "model": "gemini-3.6-flash",
            "input": [
                {
                    "type": "text",
                    "text": prompt_text
                },
                {
                    "type": "image",
                    "data": base64_string,
                    "mime_type": "image/jpeg"
                }
            ],
            "response_format": {
                "type": "text",
                "mime_type": "application/json",
                "schema": {
                    "type": "object",
                    "properties": {
                        "can_help": {
                            "type": "boolean",
                            "description": "True if the condition is minor and solvable with the available supplies."
                        },
                        "reasoning": {
                            "type": "string",
                            "description": "Brief summary explanation of the medical assessment."
                        },
                        "dispense": {
                            "type": "object",
                            "properties": {
                                "bandage": { "type": "integer", "description": "1 to dispense, 0 to hold." },
                                "alcohol_pad": { "type": "integer", "description": "1 to dispense, 0 to hold." },
                                "gauze_pad": { "type": "integer", "description": "1 to dispense, 0 to hold." }
                            },
                            "required": ["bandage", "alcohol_pad", "gauze_pad"]
                        },
                        "video_search_query": {
                            "type": "string",
                            "description": "Short YouTube search query for treatment instructions, or null if can_help is false."
                        }
                    },
                    "required": ["can_help", "reasoning", "dispense"]
                }
            }
        });

        let res = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let response_text = res["output"][0]["text"]
            .as_str()
            .or_else(|| res["candidates"][0]["content"]["parts"][0]["text"].as_str())
            .ok_or_else(|| format!("Unexpected API response format: {}", res))?;

        let analysis: VisorAnalysis = serde_json::from_str(response_text)?;

        Ok(analysis)
    }
}