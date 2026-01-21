use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Deserialize)]
pub struct IngestRequest {
    pub url: String,
    pub raw_content: String,
    pub source_type: String,
}

pub async fn ingest_handler(
    State(state): State<AppState>,
    Json(payload): Json<IngestRequest>,
) -> impl IntoResponse {
    // 1. Save Raw Material
    let result = sqlx::query!(
        "INSERT INTO raw_materials (url, content, source_type) VALUES ($1, $2, $3) RETURNING id",
        payload.url,
        payload.raw_content,
        payload.source_type
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(record) => {
            let state_clone = state.clone();
            let gemini = crate::core::gemini_client::GeminiClient::new(&crate::core::config::Config::init()); // Logic improvement needed for creating client from state
            
            // Spawn background task
            tokio::spawn(async move {
                let _ = crate::core::processor::process_material(record.id, payload.raw_content, gemini, state_clone.db).await;
            });

            (StatusCode::CREATED, Json(serde_json::json!({ "id": record.id, "status": "queued" })))
        },
        Err(e) => {
            eprintln!("Failed to ingest: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Database error" })))
        }
    }
}
