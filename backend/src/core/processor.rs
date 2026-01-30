use crate::core::gemini_client::GeminiClient;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
struct ExtractedQuestion {
    topic: String,
    difficulty: String,
    content: serde_json::Value, // Flexible JSON content
    text_for_embedding: String, // Text used to generate the vector
}

#[derive(Deserialize, Debug)]
struct ExtractionResponse {
    questions: Vec<ExtractedQuestion>,
}

pub async fn process_material(
    material_id: Uuid,
    content: String,
    gemini: GeminiClient,
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Processing material {}", material_id);

    // 1. Extract Questions using Gemini
    let prompt = format!(
        "Analyze the following text and extract practice questions for CU-TEP. \
        Return a JSON object with a key 'questions', which is a list of objects. \
        Each object must have: 'topic' (reading, listening, error_id), 'difficulty' (easy, medium, hard), \
        'content' (the actual question structure), and 'text_for_embedding' (a summary or the question text itself). \
        \n\n TEXT: {}", 
        content
    );

    let json_response = match gemini.generate_json(&prompt).await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Gemini generation failed: {}", e);
            return Err(e);
        }
    };

    // Clean markdown code blocks if present (Gemini sometimes adds ```json ... ```)
    let clean_json = json_response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```");

    let extracted: ExtractionResponse = serde_json::from_str(clean_json)?;

    // 2. Save Questions and Generate Embeddings
    for q in extracted.questions {
        // Insert Question
        let q_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO questions (id, raw_material_id, topic, content, difficulty_level) VALUES ($1, $2, $3, $4, $5)",
            q_id, material_id, q.topic, q.content, q.difficulty
        )
        .execute(&pool)
        .await?;

        // Generate Embedding
        let embedding_values = match gemini.generate_embedding(&q.text_for_embedding).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to generate embedding for question {}: {}", q_id, e);
                // Continue to next question or returning error?
                // For now, let's log and continue, or fill with zeros?
                // Better to fail this question's embedding but keep the question?
                // Let's return error to fail the batch for now for safety.
                return Err(e);
            }
        };

        let embedding = pgvector::Vector::from(embedding_values.clone());

        // Insert Embedding
        sqlx::query!(
            "INSERT INTO embeddings (question_id, chunk_text, embedding) VALUES ($1, $2, $3)",
            q_id,
            q.text_for_embedding,
            embedding as pgvector::Vector
        )
        .execute(&pool)
        .await?;
    }

    // 3. Mark processed
    sqlx::query!(
        "UPDATE raw_materials SET processed = TRUE WHERE id = $1",
        material_id
    )
    .execute(&pool)
    .await?;

    println!("Finished processing material {}", material_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;
    use sqlx::Row;

    #[tokio::test]
    async fn test_process_material_with_mock_gemini() {
        // 1. Setup Config & Client
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let config = Config {
            database_url: database_url.clone(),
            gemini_api_key: "dummy_key".to_string(),
            mock_gemini: true,
        };
        let gemini = GeminiClient::new(&config);

        // 2. Setup DB Pool
        let pool = PgPool::connect(&config.database_url)
            .await
            .expect("Failed to connect to DB");

        // 3. Insert Test Data (Raw Material)
        let raw_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO raw_materials (id, url, content, source_type) VALUES ($1, $2, $3, $4)",
            raw_id,
            "http://test.com/unit-test",
            "Unit Test Content for Processor Verification",
            "unit-test"
        )
        .execute(&pool)
        .await
        .expect("Failed to insert raw material");

        // 4. Run Processor
        let result = process_material(
            raw_id,
            "Unit Test Content".to_string(),
            gemini,
            pool.clone(),
        )
        .await;

        // 5. Verify Result
        assert!(result.is_ok(), "Processor failed: {:?}", result.err());

        // 6. Verify Database State
        // Check Question
        let questions_count: i64 =
            sqlx::query("SELECT count(*) FROM questions WHERE raw_material_id = $1")
                .bind(raw_id)
                .fetch_one(&pool)
                .await
                .expect("Failed to fetch question count")
                .get(0);
        assert!(questions_count > 0, "No questions were generated");

        // Check Embedding (we just check if any embedding exists for the questions linked to this raw material)
        // Since we don't know the question ID easily without querying, we join or verify count.
        let embeddings_count: i64 = sqlx::query(
            "SELECT count(*) FROM embeddings e JOIN questions q ON e.question_id = q.id WHERE q.raw_material_id = $1"
        )
        .bind(raw_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch embedding count")
        .get(0);

        assert!(embeddings_count > 0, "No embeddings were generated");

        // Cleanup (Optional, but good for local dev)
        // Ideally we use a transaction that rolls back, but for this simple test suite we might just leave it or delete.
        sqlx::query!("DELETE FROM embeddings USING questions WHERE embeddings.question_id = questions.id AND questions.raw_material_id = $1", raw_id)
            .execute(&pool).await.ok();
        sqlx::query!("DELETE FROM questions WHERE raw_material_id = $1", raw_id)
            .execute(&pool)
            .await
            .ok();
        sqlx::query!("DELETE FROM raw_materials WHERE id = $1", raw_id)
            .execute(&pool)
            .await
            .ok();
    }
}
