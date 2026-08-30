use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispenseItems {
    pub bandage: bool,
    pub alcohol_pad: bool,
    // pub gauze_pad: bool, // [DISABLED: 2-item dispensing only]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisorAnalysis {
    pub can_help: bool,
    pub reasoning: String,
    pub dispense: DispenseItems,
    pub video_search_query: Option<String>,
}

pub fn build_request_body(base64_string: &str) -> serde_json::Value {
    let prompt_text = "Analyze this image to evaluate the user's first-aid needs. \
        Available resources for dispensing: Bandage (Normal Size), Alcohol Pad. \
        Determine if the user has a minor condition that can be treated using ONLY the available supplies. \
        For each item in dispense, specify true to dispense or false to hold. \
        If the user requires immediate emergency care, severe medical attention, or if no first-aid help is needed, set can_help to false.";

    json!({
        "model": "gemini-3.7-flash",
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
                            "bandage": { "type": "boolean", "description": "true to dispense, false to hold." },
                            "alcohol_pad": { "type": "boolean", "description": "true to dispense, false to hold." }
                            // "gauze_pad": { "type": "boolean", "description": "true to dispense, false to hold." }
                        },
                        "required": ["bandage", "alcohol_pad"]
                    },
                    "video_search_query": {
                        "type": "string",
                        "description": "Short YouTube search query for treatment instructions, or null if can_help is false."
                    }
                },
                "required": ["can_help", "reasoning", "dispense"]
            }
        }
    })
}

pub fn extract_response_text(res: &serde_json::Value) -> Result<&str, String> {
    res["output"][0]["text"]
        .as_str()
        .or_else(|| res["candidates"][0]["content"]["parts"][0]["text"].as_str())
        .ok_or_else(|| format!("Unexpected API response format: {}", res))
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

    pub async fn analyze_image(
        &self,
        image_bytes: &[u8],
    ) -> Result<VisorAnalysis, Box<dyn std::error::Error>> {
        let base64_string = general_purpose::STANDARD.encode(image_bytes);
        let url = "https://generativelanguage.googleapis.com/v1beta/interactions";
        let body = build_request_body(&base64_string);

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

        let response_text = extract_response_text(&res)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let analysis: VisorAnalysis = serde_json::from_str(response_text)?;

        Ok(analysis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispense_items_boolean_deserialization() {
        // Note: 3rd item (gauze_pad) disabled for 2-item dispensing
        let json_data = r#"{
            "can_help": true,
            "reasoning": "Minor superficial cut requiring cleaning and bandage.",
            "dispense": {
                "bandage": true,
                "alcohol_pad": true
            },
            "video_search_query": "how to apply bandage to finger cut"
        }"#;

        let analysis: VisorAnalysis =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        assert!(analysis.can_help);
        assert_eq!(
            analysis.reasoning,
            "Minor superficial cut requiring cleaning and bandage."
        );
        assert!(analysis.dispense.bandage);
        assert!(analysis.dispense.alcohol_pad);
        // assert!(!analysis.dispense.gauze_pad); // [DISABLED: 3rd item]
        assert_eq!(
            analysis.video_search_query.as_deref(),
            Some("how to apply bandage to finger cut")
        );
    }

    #[test]
    fn test_cannot_help_emergency_deserialization() {
        // Note: 3rd item (gauze_pad) disabled for 2-item dispensing
        let json_data = r#"{
            "can_help": false,
            "reasoning": "Severe compound fracture detected. Seek emergency care immediately.",
            "dispense": {
                "bandage": false,
                "alcohol_pad": false
            },
            "video_search_query": null
        }"#;

        let analysis: VisorAnalysis =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        assert!(!analysis.can_help);
        assert_eq!(
            analysis.reasoning,
            "Severe compound fracture detected. Seek emergency care immediately."
        );
        assert!(!analysis.dispense.bandage);
        assert!(!analysis.dispense.alcohol_pad);
        // assert!(!analysis.dispense.gauze_pad); // [DISABLED: 3rd item]
        assert!(analysis.video_search_query.is_none());
    }

    #[test]
    fn test_omitted_video_search_query() {
        // Note: 3rd item (gauze_pad) disabled for 2-item dispensing
        let json_data = r#"{
            "can_help": true,
            "reasoning": "Minor abrasion, alcohol pad suggested.",
            "dispense": {
                "bandage": false,
                "alcohol_pad": true
            }
        }"#;

        let analysis: VisorAnalysis =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        assert!(analysis.can_help);
        // assert!(analysis.dispense.gauze_pad); // [DISABLED: 3rd item]
        assert!(analysis.dispense.alcohol_pad);
        assert!(!analysis.dispense.bandage);
        assert!(analysis.video_search_query.is_none());
    }

    #[test]
    fn test_malformed_json_failure() {
        let invalid_json = r#"{ "can_help": true, "reasoning": "Incomplete json" "#;
        let res: Result<VisorAnalysis, _> = serde_json::from_str(invalid_json);
        assert!(res.is_err());
    }

    #[test]
    fn test_missing_dispense_field_failure() {
        let missing_dispense = r#"{
            "can_help": true,
            "reasoning": "Missing dispense structure"
        }"#;
        let res: Result<VisorAnalysis, _> = serde_json::from_str(missing_dispense);
        assert!(res.is_err());
    }

    #[test]
    fn test_build_request_body_structure() {
        let test_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";
        let body = build_request_body(test_base64);

        assert_eq!(body["model"], "gemini-3.7-flash");
        assert_eq!(body["input"][0]["type"], "text");
        assert_eq!(body["input"][1]["type"], "image");
        assert_eq!(body["input"][1]["data"], test_base64);
        assert_eq!(body["input"][1]["mime_type"], "image/jpeg");
        assert_eq!(body["response_format"]["type"], "text");
        assert_eq!(body["response_format"]["mime_type"], "application/json");
    }

    #[test]
    fn test_extract_response_text_interactions_format() {
        let payload = json!({
            "output": [
                {
                    "text": "{\"can_help\": true, \"reasoning\": \"ok\", \"dispense\": {\"bandage\": true, \"alcohol_pad\": false}}"
                }
            ]
        });
        let text = extract_response_text(&payload).expect("Should extract output text");
        assert!(text.contains("\"can_help\": true"));
    }

    #[test]
    fn test_extract_response_text_candidates_format() {
        let payload = json!({
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "{\"can_help\": false, \"reasoning\": \"no\", \"dispense\": {\"bandage\": false, \"alcohol_pad\": false}}"
                            }
                        ]
                    }
                }
            ]
        });
        let text = extract_response_text(&payload).expect("Should extract candidates text");
        assert!(text.contains("\"can_help\": false"));
    }

    #[test]
    fn test_extract_response_text_invalid_format() {
        let payload = json!({ "error": "Invalid API Key" });
        let result = extract_response_text(&payload);
        assert!(result.is_err());
    }
}
