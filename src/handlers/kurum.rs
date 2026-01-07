use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::Document as MongoDocument};
use futures::stream::TryStreamExt;
use crate::config::AppState;
use crate::models::kurum::{KurumResponse, InstitutionsListResponse};

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

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_kurumlar));
}

