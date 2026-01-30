use crate::core::traits::{ExamGenerationEngine, PersonalizationEngine, VectorAccessor};
use serde_json::Value;

// The Stable Manager
pub struct EducationManager {
    exam_engine: Box<dyn ExamGenerationEngine>,
    personalization_engine: Box<dyn PersonalizationEngine>,
    vector_accessor: Box<dyn VectorAccessor>,
}

impl EducationManager {
    // Dependency Injection via constructor
    pub fn new(
        exam_engine: Box<dyn ExamGenerationEngine>,
        personalization_engine: Box<dyn PersonalizationEngine>,
        vector_accessor: Box<dyn VectorAccessor>,
    ) -> Self {
        Self {
            exam_engine,
            personalization_engine,
            vector_accessor,
        }
    }

    // The workflow logic (Stable)
    pub async fn generate_personalized_exam(&self, user_id: &str) -> Result<Value, String> {
        // 1. Identify what the user needs (Personalization Engine)
        let weak_points = self
            .personalization_engine
            .determine_weak_points(user_id)
            .await?;

        // 2. Decide on a topic (Logic in Manager, or delegate to Engine)
        let topic = weak_points
            .first()
            .unwrap_or(&"general".to_string())
            .clone();

        // 3. (Optional) Find similar past questions (Vector Accessor)
        // let _examples = self.vector_accessor.find_similar_questions(&[], 3).await?;

        // 4. Generate new content (Exam Engine)
        let exam = self.exam_engine.generate_exam(&topic, "medium").await?;

        Ok(exam)
    }
}
