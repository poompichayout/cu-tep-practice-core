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
