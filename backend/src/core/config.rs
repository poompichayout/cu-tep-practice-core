use dotenvy::dotenv;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub gemini_api_key: String,
    pub mock_gemini: bool,
}

impl Config {
    pub fn init() -> Config {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let gemini_api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        let mock_gemini = env::var("MOCK_GEMINI").unwrap_or_else(|_| "false".to_string()) == "true";

        Config {
            database_url,
            gemini_api_key,
            mock_gemini,
        }
    }
}
