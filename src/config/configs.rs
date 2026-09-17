use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub database_name: String,
    pub jwt_secret: String,
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub openrouter_api_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let database_name = env::var("DATABASE_NAME")
            .unwrap_or_else(|_| "flowframe".to_string());
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_super_secret_key_12345".to_string());
        let gemini_api_key = env::var("GEMINI_API_KEY")
            .or_else(|_| env::var("GEMINI_API"))
            .unwrap_or_default();
        let gemini_model = env::var("GEMINI_MODEL")
            .unwrap_or_else(|_| "gemini-flash-latest".to_string());
        let openrouter_api_key = env::var("OPENROUTER_API_KEY").unwrap_or_default();

        Self {
            database_url,
            database_name,
            jwt_secret,
            gemini_api_key,
            gemini_model,
            openrouter_api_key,
        }
    }
}
