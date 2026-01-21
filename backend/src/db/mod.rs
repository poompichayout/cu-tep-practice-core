use sqlx::postgres::{PgPoolOptions, PgPool};
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub async fn init_db() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to create pool.")
}
