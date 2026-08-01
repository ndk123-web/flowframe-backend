use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub database_name: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let database_name = env::var("DATABASE_NAME")
            .unwrap_or_else(|_| "flowframe".to_string());
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_super_secret_key_12345".to_string());

        Self {
            database_url,
            database_name,
            jwt_secret,
        }
    }
}

