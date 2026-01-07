mod config;
mod handlers;
mod models;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use config::{AppConfig, AppState};
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Environment variables yükle
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // CLI argümanlarını kontrol et
    let args: Vec<String> = std::env::args().collect();
    let create_indexes = args.iter().any(|arg| arg == "--create-indexes" || arg == "-i");

    // Config yükle
    let config = AppConfig::from_env().expect("Config yüklenemedi");
    
    // MongoDB bağlantısı
    let app_state = AppState::new(&config)
        .await
        .expect("MongoDB bağlantısı kurulamadı");

    // Eğer --create-indexes parametresi verildiyse index'leri oluştur ve çık
    if create_indexes {
        log::info!("Index oluşturma modu aktif - MongoDB index'leri oluşturuluyor...");
        AppState::ensure_indexes(&app_state.db).await;
        log::info!("Index oluşturma işlemi tamamlandı. Uygulama kapatılıyor...");
        return Ok(());
    }

    log::info!("Server başlatılıyor: {}:{}", config.host, config.port);

    let app_state = web::Data::new(app_state);

    // HTTP Server başlat
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .configure(routes::configure_routes)
    })
    .bind((config.host.as_str(), config.port))?
    .run()
    .await
}

