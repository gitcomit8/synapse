use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub firebase_credentials_path: String,
    pub r2_bucket_name: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL")?,
            jwt_secret: std::env::var("JWT_SECRET")?,
            firebase_credentials_path: std::env::var("FIREBASE_CREDENTIALS_PATH")
                .unwrap_or_else(|_| "firebase.json".to_string()),
            r2_bucket_name: std::env::var("R2_BUCKET_NAME")?,
            r2_access_key_id: std::env::var("R2_ACCESS_KEY_ID")?,
            r2_secret_access_key: std::env::var("R2_SECRET_ACCESS_KEY")?,
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
        })
    }
}
