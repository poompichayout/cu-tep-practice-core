use crate::core::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
    role: String,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    response_mime_type: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerateContentResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize, Debug)]
struct Candidate {
    content: Option<ContentResponse>,
}

#[derive(Deserialize, Debug)]
struct ContentResponse {
    parts: Option<Vec<PartResponse>>,
}

#[derive(Deserialize, Debug)]
struct PartResponse {
    text: String,
}

impl GeminiClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            api_key: config.gemini_api_key.clone(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
        }
    }

    pub async fn generate_text(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/gemini-1.5-pro:generateContent?key={}",
            self.base_url, self.api_key
        );

        let request_body = GenerateContentRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: None,
            },
        };

        let res = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("Gemini API Error: {}", error_text).into());
        }

        let response_body: GenerateContentResponse = res
            .json()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if let Some(candidates) = response_body.candidates {
            if let Some(first) = candidates.first() {
                if let Some(content) = &first.content {
                    if let Some(parts) = &content.parts {
                        if let Some(first_part) = parts.first() {
                            return Ok(first_part.text.clone());
                        }
                    }
                }
            }
        }

        Err("No content generated".into())
    }

    pub async fn generate_json(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/gemini-1.5-pro:generateContent?key={}",
            self.base_url, self.api_key
        );

        let request_body = GenerateContentRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: format!("{} \n Respond in JSON format.", prompt),
                }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: Some("application/json".to_string()),
            },
        };

        let res = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // ... (Similar parsing logic, concise for MVP) ...
        let response_body: GenerateContentResponse = res
            .json()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if let Some(candidates) = &response_body.candidates {
            if let Some(first) = candidates.first() {
                if let Some(content) = &first.content {
                    if let Some(parts) = &content.parts {
                        if let Some(first_part) = parts.first() {
                            return Ok(first_part.text.clone());
                        }
                    }
                }
            }
        }

        eprintln!(
            "Gemini JSON Generation Failed. Raw Response: {:?}",
            response_body
        );
        Err("No JSON content generated".into())
    }

    pub async fn generate_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/text-embedding-004:embedContent?key={}",
            self.base_url, self.api_key
        );

        let request_body = EmbedContentRequest {
            content: Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: text.to_string(),
                }],
            },
        };

        let res = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(format!("Gemini API Error (Embedding): {}", error_text).into());
        }

        let response_body: EmbedContentResponse = res
            .json()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if let Some(embedding) = response_body.embedding {
            return Ok(embedding.values);
        }

        Err("No embedding generated".into())
    }
}

#[derive(Serialize)]
struct EmbedContentRequest {
    content: Content,
}

#[derive(Deserialize, Debug)]
struct EmbedContentResponse {
    embedding: Option<ContentEmbedding>,
}

#[derive(Deserialize, Debug)]
struct ContentEmbedding {
    values: Vec<f32>,
}
