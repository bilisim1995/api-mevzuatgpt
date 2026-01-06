use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::announcement::{AnnouncementResponse, AnnouncementsListResponse};

#[derive(serde::Deserialize)]
pub struct AnnouncementQuery {
    pub kurum_id: Option<String>,
}

pub async fn get_announcements(
    state: web::Data<AppState>,
    query: web::Query<AnnouncementQuery>,
) -> HttpResponse {
    let collection: Collection<MongoDocument> = state.db.collection("kurum_duyuru");

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
            return HttpResponse::InternalServerError().json(AnnouncementsListResponse {
                success: false,
                data: vec![],
                count: None,
                message: "Duyurular alınamadı".to_string(),
            });
        }
    };

    let mut announcements: Vec<AnnouncementResponse> = Vec::new();

    // Sonuçları işle
    while let Ok(true) = cursor.advance().await {
        if let Ok(doc_map) = cursor.deserialize_current() {
            let duyuru_linki = doc_map
                .get_str("duyuru_linki")
                .unwrap_or("")
                .to_string();

            // Baslik: Sadece koleksiyondan oku, yoksa null
            let baslik = doc_map
                .get_str("baslik")
                .ok()
                .map(|s| s.to_string());

            // Tarih için şu anki tarihi kullan (veya _id'den türet)
            let tarih = doc_map
                .get_object_id("_id")
                .map(|oid| {
                    // ObjectId'den timestamp çıkar
                    let bson_datetime = oid.timestamp();
                    let timestamp = bson_datetime.timestamp_millis() / 1000;
                    let date = chrono::DateTime::from_timestamp(timestamp, 0)
                        .unwrap_or_else(|| chrono::Utc::now());
                    date.format("%Y-%m-%d").to_string()
                })
                .unwrap_or_else(|_| chrono::Utc::now().format("%Y-%m-%d").to_string());

            announcements.push(AnnouncementResponse {
                baslik,
                link: duyuru_linki,
                tarih,
            });
        }
    }

    // Toplam sayıyı al
    let count = match collection.count_documents(count_filter, None).await {
        Ok(count) => Some(count),
        Err(_) => None,
    };

    HttpResponse::Ok().json(AnnouncementsListResponse {
        success: true,
        data: announcements,
        count,
        message: "İşlem başarılı".to_string(),
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_announcements));
}

