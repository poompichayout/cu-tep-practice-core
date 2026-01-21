use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use crate::core::config::Config;
use crate::db::init_db;

mod api;
mod core;
mod db;

// Export AppState so submodules can use it
pub use db::AppState;

#[tokio::main]
async fn main() {
    // 1. Init Config
    let config = Config::init();

    // 2. Init DB
    // We assume DATABASE_URL is set in .env or environment
    let pool = init_db().await;
    
    // Auto-migrate
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    // 3. App State
    let app_state = AppState { db: pool };

    // 4. Router
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/internal/ingest", post(api::ingest::ingest_handler))
        .with_state(app_state);

    // 5. Run Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
