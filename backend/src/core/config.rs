use std::env;
use dotenvy::dotenv;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub gemini_api_key: String,
}

impl Config {
    pub fn init() -> Config {
        dotenv().ok();
        
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let gemini_api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");

        Config {
            database_url,
            gemini_api_key,
        }
    }
}
