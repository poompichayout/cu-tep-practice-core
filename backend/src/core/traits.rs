use async_trait::async_trait;
use serde_json::Value;

// Domain Models
#[derive(Debug, Clone)]
pub struct ExamRequest {
    pub user_id: String,
    // Add other fields as needed
}

// Volatile: How exams are generated changes (e.g. Prompt tuning, different models)
#[async_trait]
pub trait ExamGenerationEngine: Send + Sync {
    async fn generate_exam(&self, topic: &str, difficulty: &str) -> Result<Value, String>;
}

// Volatile: How we personalize changes (e.g. Simple Random vs ML model)
#[async_trait]
pub trait PersonalizationEngine: Send + Sync {
    async fn determine_weak_points(&self, user_id: &str) -> Result<Vec<String>, String>;
}

// Stable/Accessor: Wrapper around Vector DB details
#[async_trait]
pub trait VectorAccessor: Send + Sync {
    async fn find_similar_questions(&self, vector: &[f32], limit: i64) -> Result<Vec<Value>, String>;
}
