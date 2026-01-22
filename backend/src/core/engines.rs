use async_trait::async_trait;
use serde_json::Value;
use crate::core::traits::{ExamGenerationEngine, PersonalizationEngine};
use crate::core::gemini_client::GeminiClient;

// --- Exam Generation Engine ---

pub struct GeminiExamEngine {
    client: GeminiClient,
}

impl GeminiExamEngine {
    pub fn new(client: GeminiClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ExamGenerationEngine for GeminiExamEngine {
    async fn generate_exam(&self, topic: &str, difficulty: &str) -> Result<Value, String> {
        let prompt = format!(
            "Generate a {} difficulty exam question for topic: {}. Return as JSON.", 
            difficulty, topic
        );
        match self.client.generate_json(&prompt).await {
            Ok(json_str) => {
                 // naive parsing for MVP
                 serde_json::from_str(&json_str).map_err(|e| e.to_string())
            },
            Err(e) => Err(e.to_string())
        }
    }
}

// --- Personalization Engine ---

pub struct RandomPersonalizationEngine;

#[async_trait]
impl PersonalizationEngine for RandomPersonalizationEngine {
    async fn determine_weak_points(&self, _user_id: &str) -> Result<Vec<String>, String> {
        // MVP: Returns random topics or static list
        Ok(vec!["reading_comprehension".to_string(), "error_identification".to_string()])
    }
}
