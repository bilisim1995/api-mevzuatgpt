use actix_web::{web, HttpResponse, http::StatusCode};
use mongodb::{Collection, bson::{doc, oid::ObjectId, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::link::{LinkResponse, LinksListResponse};

#[derive(serde::Deserialize)]
pub struct LinkQuery {
    pub kurum_id: String,
}

pub async fn get_links(
    state: web::Data<AppState>,
    query: web::Query<LinkQuery>,
) -> HttpResponse {
    // kurum_id zorunlu - zaten String olarak tanımlı, boş kontrolü yap
    if query.kurum_id.is_empty() {
        return HttpResponse::build(StatusCode::BAD_REQUEST).json(LinksListResponse {
            success: false,
            data: vec![],
            count: 0,
            message: None,
            error: Some("kurum_id parameter is required".to_string()),
        });
    }

    // kurum_id'nin geçerli ObjectID formatında olup olmadığını kontrol et
    let _kurum_object_id = match ObjectId::parse_str(&query.kurum_id) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::build(StatusCode::BAD_REQUEST).json(LinksListResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Invalid kurum_id format".to_string()),
            });
        }
    };

    // links koleksiyonundan çek
    let collection: Collection<MongoDocument> = state.db.collection("links");

    // Match stage için filter oluştur - hem ObjectID hem string formatını dene
    let match_filter = doc! {
        "$or": [
            { "kurum_id": _kurum_object_id },
            { "kurum_id": query.kurum_id.clone() }
        ]
    };

    // Count için filter'ı klonla
    let count_filter = match_filter.clone();

    // Aggregation pipeline oluştur
    let pipeline = vec![
        doc! { "$match": match_filter },
    ];

    // Aggregation çalıştır
    let mut cursor = match collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("MongoDB aggregation hatası: {}", e);
            return HttpResponse::InternalServerError().json(LinksListResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Failed to fetch links".to_string()),
            });
        }
    };

    let mut links: Vec<LinkResponse> = Vec::new();

    // Sonuçları işle
    while let Ok(true) = cursor.advance().await {
        if let Ok(doc_map) = cursor.deserialize_current() {
            // ID
            let id = doc_map
                .get_object_id("_id")
                .map(|oid| oid.to_hex())
                .unwrap_or_default();

            // Kurum ID - ObjectID olarak oku ve hex'e çevir
            let kurum_id = doc_map
                .get_object_id("kurum_id")
                .map(|oid| oid.to_hex())
                .or_else(|_| doc_map.get_str("kurum_id").map(|s| s.to_string()))
                .unwrap_or_else(|_| query.kurum_id.clone());

            // Baslik
            let baslik = doc_map
                .get_str("baslik")
                .unwrap_or("")
                .to_string();

            // Aciklama
            let aciklama = doc_map
                .get_str("aciklama")
                .unwrap_or("")
                .to_string();

            // URL
            let url = doc_map
                .get_str("url")
                .unwrap_or("")
                .to_string();

            // Created at: Önce koleksiyondan kontrol et, yoksa _id'den türet
            let created_at = match doc_map.get_str("created_at") {
                Ok(created_at_str) => created_at_str.to_string(),
                Err(_) => {
                    // Koleksiyonda yoksa, _id'den timestamp çıkar
                    match doc_map.get_object_id("_id") {
                        Ok(oid) => {
                            let bson_datetime = oid.timestamp();
                            let timestamp = bson_datetime.timestamp_millis() / 1000;
                            chrono::DateTime::from_timestamp(timestamp, 0)
                                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                                .unwrap_or_else(|| {
                                    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
                                })
                        }
                        Err(_) => chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    }
                }
            };

            links.push(LinkResponse {
                id,
                baslik,
                aciklama,
                url,
                kurum_id,
                created_at,
            });
        } else {
            log::error!("Link deserialize hatası");
            return HttpResponse::InternalServerError().json(LinksListResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Failed to decode links".to_string()),
            });
        }
    }

    // Toplam sayıyı al
    let count = match collection.count_documents(count_filter, None).await {
        Ok(count) => count,
        Err(_) => links.len() as u64, // Fallback olarak mevcut liste uzunluğu
    };

    HttpResponse::Ok().json(LinksListResponse {
        success: true,
        data: links,
        count,
        message: Some("Kurum linkleri başarıyla çekildi".to_string()),
        error: None,
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_links));
}

