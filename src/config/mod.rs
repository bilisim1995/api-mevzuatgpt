use mongodb::{Client, Database, options::{ClientOptions, IndexOptions}, IndexModel};
use mongodb::bson::doc;
use std::env;
use std::time::Duration;

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
        // MongoDB client options ile timeout ayarları
        let mut client_options = ClientOptions::parse(&config.mongodb_uri).await?;
        
        // Timeout ayarları (arama işlemleri uzun sürebilir)
        client_options.server_selection_timeout = Some(Duration::from_secs(60)); // 60 saniye
        client_options.connect_timeout = Some(Duration::from_secs(30)); // 30 saniye
        // socket_timeout private field, MongoDB driver'ı otomatik yönetiyor
        
        let client = Client::with_options(client_options)?;
        let db = client.database(&config.mongodb_db_name);

        log::info!("MongoDB bağlantısı başarıyla kuruldu: {}", config.mongodb_db_name);

        Ok(AppState { db })
    }

    // MongoDB index'lerini güvenli bir şekilde oluştur
    // Mevcut verilere zarar vermez, sadece performans için index oluşturur
    // Index zaten varsa hata vermez, sadece log yazar
    // Bu fonksiyon manuel olarak çağrılmalıdır (--create-indexes parametresi ile)
    pub async fn ensure_indexes(db: &Database) {
        log::info!("MongoDB index'leri kontrol ediliyor...");

        // metadata koleksiyonu index'leri
        let metadata_collection = db.collection::<mongodb::bson::Document>("metadata");
        
        // url_slug unique index
        if let Err(e) = metadata_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "url_slug": 1 })
                .options(IndexOptions::builder()
                    .unique(true)
                    .name("idx_url_slug_unique".to_string())
                    .build())
                .build(),
            None,
        ).await {
            // Index zaten varsa veya başka bir hata varsa sadece log yaz
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ url_slug unique index zaten mevcut");
            } else {
                log::warn!("url_slug index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ url_slug unique index oluşturuldu");
        }

        // kurum_id index
        if let Err(e) = metadata_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "kurum_id": 1 })
                .options(IndexOptions::builder()
                    .name("idx_kurum_id".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ kurum_id index zaten mevcut");
            } else {
                log::warn!("kurum_id index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ kurum_id index oluşturuldu");
        }

        // olusturulma_tarihi index
        if let Err(e) = metadata_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "olusturulma_tarihi": -1 })
                .options(IndexOptions::builder()
                    .name("idx_olusturulma_tarihi_desc".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ olusturulma_tarihi index zaten mevcut");
            } else {
                log::warn!("olusturulma_tarihi index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ olusturulma_tarihi index oluşturuldu");
        }

        // belge_turu index
        if let Err(e) = metadata_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "belge_turu": 1 })
                .options(IndexOptions::builder()
                    .name("idx_belge_turu".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ belge_turu index zaten mevcut");
            } else {
                log::warn!("belge_turu index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ belge_turu index oluşturuldu");
        }

        // Compound index: kurum_id + olusturulma_tarihi
        if let Err(e) = metadata_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "kurum_id": 1, "olusturulma_tarihi": -1 })
                .options(IndexOptions::builder()
                    .name("idx_kurum_tarih".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ kurum_tarih compound index zaten mevcut");
            } else {
                log::warn!("kurum_tarih compound index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ kurum_tarih compound index oluşturuldu");
        }

        // content koleksiyonu index'leri
        let content_collection = db.collection::<mongodb::bson::Document>("content");
        
        // metadata_id index
        if let Err(e) = content_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "metadata_id": 1 })
                .options(IndexOptions::builder()
                    .name("idx_content_metadata_id".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ content metadata_id index zaten mevcut");
            } else {
                log::warn!("content metadata_id index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ content metadata_id index oluşturuldu");
        }

        // kurumlar koleksiyonu index'leri
        let kurumlar_collection = db.collection::<mongodb::bson::Document>("kurumlar");
        
        // kurum_adi index (opsiyonel)
        if let Err(e) = kurumlar_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "kurum_adi": 1 })
                .options(IndexOptions::builder()
                    .name("idx_kurum_adi".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ kurum_adi index zaten mevcut");
            } else {
                log::warn!("kurum_adi index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ kurum_adi index oluşturuldu");
        }

        // kurum_duyuru koleksiyonu index'leri
        let kurum_duyuru_collection = db.collection::<mongodb::bson::Document>("kurum_duyuru");
        
        // kurum_id index
        if let Err(e) = kurum_duyuru_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "kurum_id": 1 })
                .options(IndexOptions::builder()
                    .name("idx_duyuru_kurum_id".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ kurum_duyuru kurum_id index zaten mevcut");
            } else {
                log::warn!("kurum_duyuru kurum_id index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ kurum_duyuru kurum_id index oluşturuldu");
        }

        // links koleksiyonu index'leri
        let links_collection = db.collection::<mongodb::bson::Document>("links");
        
        // kurum_id index
        if let Err(e) = links_collection.create_index(
            IndexModel::builder()
                .keys(doc! { "kurum_id": 1 })
                .options(IndexOptions::builder()
                    .name("idx_links_kurum_id".to_string())
                    .build())
                .build(),
            None,
        ).await {
            if e.to_string().contains("already exists") || e.to_string().contains("duplicate") {
                log::info!("✓ links kurum_id index zaten mevcut");
            } else {
                log::warn!("links kurum_id index oluşturulamadı: {}", e);
            }
        } else {
            log::info!("✓ links kurum_id index oluşturuldu");
        }

        log::info!("MongoDB index kontrolü tamamlandı");
    }
}

