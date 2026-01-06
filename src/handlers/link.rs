use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::link::{LinkResponse, LinksListResponse};

#[derive(serde::Deserialize)]
pub struct LinkQuery {
    pub kurum_id: Option<String>,
}

pub async fn get_links(
    state: web::Data<AppState>,
    query: web::Query<LinkQuery>,
) -> HttpResponse {
    // Koleksiyon adını dene: önce "kurum_link", sonra "links"
    let collection: Collection<MongoDocument> = state.db.collection("kurum_link");

    // Match stage için filter oluştur
    let mut match_filter = doc! {};
    
    if let Some(kurum_id) = &query.kurum_id {
        match_filter.insert("kurum_id", kurum_id);
    }

    // Count için filter'ı klonla
    let count_filter = match_filter.clone();

    // Aggregation pipeline oluştur
    let pipeline = vec![
        doc! { "$match": match_filter },
        doc! { "$sort": { "_id": -1 } }, // En yeni önce
    ];

    // Aggregation çalıştır
    let mut cursor = match collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("MongoDB aggregation hatası: {}", e);
            return HttpResponse::InternalServerError().json(LinksListResponse {
                success: false,
                data: vec![],
                count: None,
                message: "Linkler alınamadı".to_string(),
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

            // Kurum ID
            let kurum_id = doc_map
                .get_str("kurum_id")
                .unwrap_or("")
                .to_string();

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
        }
    }

    // Toplam sayıyı al
    let count = match collection.count_documents(count_filter, None).await {
        Ok(count) => Some(count),
        Err(_) => None,
    };

    HttpResponse::Ok().json(LinksListResponse {
        success: true,
        data: links,
        count,
        message: "İşlem başarılı".to_string(),
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_links));
}

