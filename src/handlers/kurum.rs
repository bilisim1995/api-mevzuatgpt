use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::Document as MongoDocument};
use futures::stream::TryStreamExt;
use crate::config::AppState;
use crate::models::kurum::{KurumResponse, InstitutionsListResponse, KurumBySlugResponse, KurumBySlugData};

// Helper function to create slug from institution name
fn create_kurum_slug(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace('ç', "c")
        .replace('ğ', "g")
        .replace('ı', "i")
        .replace('ö', "o")
        .replace('ş', "s")
        .replace('ü', "u")
}

pub async fn get_kurumlar(state: web::Data<AppState>) -> HttpResponse {
    let kurum_collection: Collection<MongoDocument> = state.db.collection("kurumlar");

    // Tüm kurumları al
    let kurum_docs = match kurum_collection.find(None, None).await {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<_>>().await {
                Ok(docs) => docs,
                Err(e) => {
                    log::error!("Kurum deserialize hatası: {}", e);
                    return HttpResponse::InternalServerError().json(InstitutionsListResponse {
                        success: false,
                        data: vec![],
                        count: None,
                        message: "Kurumlar listesi alınamadı".to_string(),
                    });
                }
            }
        }
        Err(e) => {
            log::error!("MongoDB sorgu hatası: {}", e);
            return HttpResponse::InternalServerError().json(InstitutionsListResponse {
                success: false,
                data: vec![],
                count: None,
                message: "Kurumlar listesi alınamadı".to_string(),
            });
        }
    };

    // Kurumları response formatına dönüştür
    let mut kurumlar: Vec<KurumResponse> = Vec::new();

    for doc_map in kurum_docs {
        let kurum_id = doc_map
            .get_object_id("_id")
            .map(|oid| oid.to_hex())
            .unwrap_or_default();

        let kurum_adi = doc_map
            .get_str("kurum_adi")
            .or_else(|_| doc_map.get_str("kurumAdi"))
            .unwrap_or("")
            .to_string();

        let kurum_logo = doc_map
            .get_str("kurum_logo")
            .or_else(|_| doc_map.get_str("kurumLogo"))
            .unwrap_or("")
            .to_string();

        let kurum_aciklama = doc_map
            .get_str("aciklama")
            .or_else(|_| doc_map.get_str("kurumAciklama"))
            .unwrap_or("")
            .to_string();

        let detsis = doc_map
            .get_str("detsis")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        kurumlar.push(KurumResponse {
            kurum_id,
            kurum_adi,
            kurum_logo,
            kurum_aciklama,
            detsis,
        });
    }

    let total_count = kurumlar.len() as u64;

    HttpResponse::Ok().json(InstitutionsListResponse {
        success: true,
        data: kurumlar,
        count: Some(total_count),
        message: "İşlem başarılı".to_string(),
    })
}

pub async fn get_kurum_by_slug(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let slug = path.into_inner();
    let kurum_collection: Collection<MongoDocument> = state.db.collection("kurumlar");

    // Tüm kurumları al ve slug ile eşleştir
    let kurum_docs = match kurum_collection.find(None, None).await {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<_>>().await {
                Ok(docs) => docs,
                Err(e) => {
                    log::error!("Kurum deserialize hatası: {}", e);
                    return HttpResponse::InternalServerError().json(KurumBySlugResponse {
                        success: false,
                        data: None,
                        message: "Kurum sorgulanamadı".to_string(),
                    });
                }
            }
        }
        Err(e) => {
            log::error!("MongoDB sorgu hatası: {}", e);
            return HttpResponse::InternalServerError().json(KurumBySlugResponse {
                success: false,
                data: None,
                message: "Kurum sorgulanamadı".to_string(),
            });
        }
    };

    // Slug ile eşleşen kurumu bul
    for doc_map in kurum_docs {
        let kurum_adi = doc_map
            .get_str("kurum_adi")
            .or_else(|_| doc_map.get_str("kurumAdi"))
            .unwrap_or("")
            .to_string();

        if kurum_adi.is_empty() {
            continue;
        }

        let kurum_slug = create_kurum_slug(&kurum_adi);

        if kurum_slug == slug {
            // Eşleşme bulundu, ID'yi döndür
            let kurum_id = doc_map
                .get_object_id("_id")
                .map(|oid| oid.to_hex())
                .unwrap_or_default();

            return HttpResponse::Ok().json(KurumBySlugResponse {
                success: true,
                data: Some(KurumBySlugData { kurum_id }),
                message: "Başarılı".to_string(),
            });
        }
    }

    // Kurum bulunamadı
    HttpResponse::NotFound().json(KurumBySlugResponse {
        success: false,
        data: None,
        message: "Kurum bulunamadı".to_string(),
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_kurumlar))
        .route("/slug/{slug}", web::get().to(get_kurum_by_slug));
}

