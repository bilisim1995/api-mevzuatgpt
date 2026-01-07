use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::regulation::{RecentRegulationResponse, RecentRegulationsListResponse};

#[derive(serde::Deserialize)]
pub struct RecentRegulationsQuery {
    pub limit: Option<u64>,
}

pub async fn get_recent_regulations(
    state: web::Data<AppState>,
    query: web::Query<RecentRegulationsQuery>,
) -> HttpResponse {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");

    // Query parametrelerini al
    let limit = query.limit.unwrap_or(50).min(1000); // Maksimum 1000
    // Her zaman olusturulma_tarihi'ne göre desc sıralama yap
    let sort_by = "olusturulma_tarihi";
    let sort_value = -1; // desc (en yeni önce)

    // Aggregation pipeline oluştur
    let pipeline = vec![
        doc! {
            "$addFields": {
                "kurum_id_object": {
                    "$toObjectId": "$kurum_id"
                }
            }
        },
        doc! { "$sort": { sort_by: sort_value } },
        doc! { "$limit": limit as i64 },
        doc! {
            "$lookup": {
                "from": "kurumlar",
                "localField": "kurum_id_object",
                "foreignField": "_id",
                "as": "kurum_bilgisi"
            }
        },
        doc! {
            "$unwind": {
                "path": "$kurum_bilgisi",
                "preserveNullAndEmptyArrays": true
            }
        },
    ];

    // Aggregation çalıştır
    let mut cursor = match metadata_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("MongoDB aggregation hatası: {}", e);
            return HttpResponse::InternalServerError().json(RecentRegulationsListResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Mevzuatlar alınamadı".to_string()),
            });
        }
    };

    let mut regulations: Vec<RecentRegulationResponse> = Vec::new();

    // Sonuçları işle
    while let Ok(true) = cursor.advance().await {
        if let Ok(doc_map) = cursor.deserialize_current() {
            // Kurum adını al
            let kurum_adi = doc_map
                .get_document("kurum_bilgisi")
                .ok()
                .and_then(|k| k.get_str("kurum_adi").ok())
                .unwrap_or("")
                .to_string();

            // Diğer alanları al
            let pdf_adi = doc_map
                .get_str("pdf_adi")
                .unwrap_or("")
                .to_string();

            let aciklama = doc_map
                .get_str("aciklama")
                .unwrap_or("")
                .to_string();

            let olusturulma_tarihi = doc_map
                .get_str("olusturulma_tarihi")
                .unwrap_or("")
                .to_string();

            let belge_turu = doc_map
                .get_str("belge_turu")
                .unwrap_or("")
                .to_string();

            let url_slug = doc_map
                .get_str("url_slug")
                .unwrap_or("")
                .to_string();

            regulations.push(RecentRegulationResponse {
                pdf_adi,
                kurum_adi,
                aciklama,
                olusturulma_tarihi,
                belge_turu,
                url_slug,
            });
        }
    }

    let count = regulations.len() as u64;

    HttpResponse::Ok().json(RecentRegulationsListResponse {
        success: true,
        data: regulations,
        count,
        message: Some("İşlem başarılı".to_string()),
        error: None,
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/recent", web::get().to(get_recent_regulations));
}

