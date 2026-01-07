use mongodb::{Client, Database};
use std::env;

pub struct AppConfig {
    pub mongodb_uri: String,
    pub mongodb_db_name: String,
    pub port: u16,
    pub host: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        // Önce MONGODB_CONNECTION_STRING'i kontrol et, yoksa MONGODB_URI'yi dene
        let mongodb_uri = env::var("MONGODB_CONNECTION_STRING")
            .or_else(|_| env::var("MONGODB_URI"))
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        // Önce MONGODB_DATABASE'i kontrol et, yoksa MONGODB_DB_NAME'i dene
        let mongodb_db_name = env::var("MONGODB_DATABASE")
            .or_else(|_| env::var("MONGODB_DB_NAME"))
            .unwrap_or_else(|_| "mevzuatgpt".to_string());

        Ok(AppConfig {
            mongodb_uri,
            mongodb_db_name,
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            host: env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
        })
    }
}

pub struct AppState {
    pub db: Database,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::with_uri_str(&config.mongodb_uri).await?;
        let db = client.database(&config.mongodb_db_name);

        log::info!("MongoDB bağlantısı başarıyla kuruldu: {}", config.mongodb_db_name);

        Ok(AppState { db })
    }
}

