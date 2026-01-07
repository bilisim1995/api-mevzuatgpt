use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::statistics::{StatisticsResponse, StatisticsData, BelgeTuruCount};

pub async fn get_statistics(state: web::Data<AppState>) -> HttpResponse {
    let kurum_collection: Collection<MongoDocument> = state.db.collection("kurumlar");
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");

    // Toplam kurum sayısı
    let total_kurumlar = match kurum_collection.count_documents(doc! {}, None).await {
        Ok(count) => count,
        Err(e) => {
            log::error!("Kurum sayısı alınamadı: {}", e);
            return HttpResponse::InternalServerError().json(StatisticsResponse {
                success: false,
                data: None,
                message: None,
                error: Some("Failed to count institutions".to_string()),
            });
        }
    };

    // Toplam belge sayısı
    let total_belgeler = match metadata_collection.count_documents(doc! {}, None).await {
        Ok(count) => count,
        Err(e) => {
            log::error!("Belge sayısı alınamadı: {}", e);
            return HttpResponse::InternalServerError().json(StatisticsResponse {
                success: false,
                data: None,
                message: None,
                error: Some("Failed to count documents".to_string()),
            });
        }
    };

    // Belge türü istatistikleri - Aggregation pipeline
    let belge_turu_pipeline = vec![
        doc! {
            "$group": {
                "_id": {
                    "$cond": {
                        "if": { "$or": [
                            { "$eq": ["$belge_turu", null] },
                            { "$eq": ["$belge_turu", ""] }
                        ]},
                        "then": "Belirtilmemiş",
                        "else": "$belge_turu"
                    }
                },
                "count": { "$sum": 1 }
            }
        },
        doc! {
            "$project": {
                "belge_turu": "$_id",
                "count": 1,
                "_id": 0
            }
        },
        doc! {
            "$sort": { "count": -1 }
        },
    ];

    let mut belge_turu_istatistik: Vec<BelgeTuruCount> = Vec::new();

    match metadata_collection.aggregate(belge_turu_pipeline, None).await {
        Ok(mut cursor) => {
            while let Ok(true) = cursor.advance().await {
                if let Ok(doc_map) = cursor.deserialize_current() {
                    let belge_turu = doc_map
                        .get_str("belge_turu")
                        .unwrap_or("Belirtilmemiş")
                        .to_string();

                    // Count değerini farklı tiplerden oku
                    let count = if let Ok(c) = doc_map.get_i64("count") {
                        c as u64
                    } else if let Ok(c) = doc_map.get_i32("count") {
                        c as u64
                    } else {
                        // BSON value'dan direkt oku
                        doc_map.get("count")
                            .and_then(|v| v.as_i64())
                            .or_else(|| doc_map.get("count").and_then(|v| v.as_i32().map(|i| i as i64)))
                            .unwrap_or(0) as u64
                    };

                    belge_turu_istatistik.push(BelgeTuruCount {
                        belge_turu,
                        count,
                    });
                }
            }
        }
        Err(e) => {
            log::error!("Belge türü istatistikleri alınamadı: {}", e);
            return HttpResponse::InternalServerError().json(StatisticsResponse {
                success: false,
                data: None,
                message: None,
                error: Some("Failed to aggregate document types".to_string()),
            });
        }
    }

    HttpResponse::Ok().json(StatisticsResponse {
        success: true,
        data: Some(StatisticsData {
            total_kurumlar,
            total_belgeler,
            belge_turu_istatistik,
        }),
        message: Some("Statistics fetched successfully".to_string()),
        error: None,
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_statistics));
}

