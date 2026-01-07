use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::document::{DocumentResponse, DocumentsListResponse};
use crate::models::document_filters::{DocumentFiltersResponse, DocumentFiltersData};
use std::collections::HashSet;
use regex;

#[derive(serde::Deserialize)]
pub struct DocumentQuery {
    pub kurum_id: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub belge_turu: Option<String>,
    pub etiketler: Option<String>,
}

pub async fn get_documents(
    state: web::Data<AppState>,
    query: web::Query<DocumentQuery>,
) -> HttpResponse {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");

    // Query parametrelerini al
    let limit = query.limit.unwrap_or(10000);
    let offset = query.offset.unwrap_or(0);
    let sort_by = query.sort_by.as_deref().unwrap_or("olusturulma_tarihi");
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");
    let sort_value = if sort_order == "asc" { 1 } else { -1 };

    // Match stage için filter oluştur
    let mut match_filter = doc! {};
    
    if let Some(kurum_id) = &query.kurum_id {
        match_filter.insert("kurum_id", kurum_id);
    }
    
    // belge_turu filtresi
    if let Some(belge_turu) = &query.belge_turu {
        if !belge_turu.is_empty() {
            match_filter.insert("belge_turu", belge_turu);
        }
    }
    
    // etiketler filtresi (virgülle ayrılmış string içinde arama)
    if let Some(etiketler) = &query.etiketler {
        if !etiketler.is_empty() {
            // Etiketler virgülle ayrılmış string içinde belirli bir etiketin geçip geçmediğini kontrol et
            // Regex ile etiketin virgülle ayrılmış listede geçip geçmediğini kontrol et
            let regex_pattern = format!(r"(^|,\s*){}(,|$)", regex::escape(etiketler));
            match_filter.insert("etiketler", doc! {
                "$regex": regex_pattern,
                "$options": "i" // case insensitive
            });
        }
    }

    // Count için filter'ı klonla (pipeline'da move edilecek)
    let count_filter = match_filter.clone();

    // Aggregation pipeline oluştur
    let mut pipeline = vec![
        doc! { "$match": match_filter },
        doc! {
            "$addFields": {
                "kurum_id_object": {
                    "$toObjectId": "$kurum_id"
                }
            }
        },
        doc! { "$sort": { sort_by: sort_value } },
    ];
    
    // Offset varsa $skip ekle
    if offset > 0 {
        pipeline.push(doc! { "$skip": offset as i64 });
    }
    
    // Limit ekle
    pipeline.push(doc! { "$limit": limit as i64 });
    
    // Lookup ve unwind ekle
    pipeline.extend(vec![
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
    ]);

    // Aggregation çalıştır
    let mut cursor = match metadata_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("MongoDB aggregation hatası: {}", e);
            return HttpResponse::InternalServerError().json(DocumentsListResponse {
                success: false,
                data: vec![],
                count: None,
                message: "Belgeler alınamadı".to_string(),
            });
        }
    };

    let mut documents: Vec<DocumentResponse> = Vec::new();

    // Sonuçları işle
    while let Ok(true) = cursor.advance().await {
        if let Ok(doc_map) = cursor.deserialize_current() {
            // Sadece istenen alanları al
            let url_slug = doc_map
                .get_str("url_slug")
                .unwrap_or("")
                .to_string();

            let pdf_adi = doc_map
                .get_str("pdf_adi")
                .unwrap_or("")
                .to_string();

            let aciklama = doc_map
                .get_str("aciklama")
                .unwrap_or("")
                .to_string();

            let belge_yayin_tarihi = doc_map
                .get_str("belge_yayin_tarihi")
                .unwrap_or("")
                .to_string();

            let belge_turu = doc_map
                .get_str("belge_turu")
                .unwrap_or("")
                .to_string();

            let belge_durumu = doc_map
                .get_str("belge_durumu")
                .unwrap_or("")
                .to_string();

            let etiketler = doc_map
                .get_str("etiketler")
                .unwrap_or("")
                .to_string();

            let anahtar_kelimeler = doc_map
                .get_str("anahtar_kelimeler")
                .unwrap_or("")
                .to_string();

            let pdf_url = doc_map
                .get_str("pdf_url")
                .unwrap_or("")
                .to_string();

            documents.push(DocumentResponse {
                url_slug,
                pdf_adi,
                aciklama,
                belge_yayin_tarihi,
                belge_turu,
                belge_durumu,
                etiketler,
                anahtar_kelimeler,
                pdf_url,
            });
        }
    }

    // Toplam sayıyı al (pagination için)
    let count = match metadata_collection.count_documents(count_filter, None).await {
        Ok(count) => Some(count),
        Err(_) => None,
    };

    HttpResponse::Ok().json(DocumentsListResponse {
        success: true,
        data: documents,
        count,
        message: "İşlem başarılı".to_string(),
    })
}

pub async fn get_document_filters(
    state: web::Data<AppState>,
    query: web::Query<DocumentQuery>,
) -> HttpResponse {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");
    
    // Match stage için filter oluştur (sadece kurum_id filtresi)
    let mut match_filter = doc! {};
    
    if let Some(kurum_id) = &query.kurum_id {
        match_filter.insert("kurum_id", kurum_id);
    }
    
    // belge_turu unique değerlerini al
    let belge_turu_pipeline = vec![
        doc! { "$match": match_filter.clone() },
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
                }
            }
        },
        doc! {
            "$project": {
                "belge_turu": "$_id",
                "_id": 0
            }
        },
        doc! {
            "$sort": { "belge_turu": 1 }
        },
    ];
    
    let mut belge_turu_list: Vec<String> = Vec::new();
    match metadata_collection.aggregate(belge_turu_pipeline, None).await {
        Ok(mut cursor) => {
            while let Ok(true) = cursor.advance().await {
                if let Ok(doc_map) = cursor.deserialize_current() {
                    if let Ok(belge_turu) = doc_map.get_str("belge_turu") {
                        belge_turu_list.push(belge_turu.to_string());
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Belge türü listesi alınamadı: {}", e);
            return HttpResponse::InternalServerError().json(DocumentFiltersResponse {
                success: false,
                data: DocumentFiltersData {
                    belge_turu: vec![],
                    etiketler: vec![],
                },
                message: None,
                error: Some("Belge türü listesi alınamadı".to_string()),
            });
        }
    }
    
    // etiketler unique değerlerini al
    let etiketler_pipeline = vec![
        doc! { "$match": match_filter },
        doc! {
            "$project": {
                "etiketler": 1
            }
        },
    ];
    
    let mut etiketler_set: HashSet<String> = HashSet::new();
    match metadata_collection.aggregate(etiketler_pipeline, None).await {
        Ok(mut cursor) => {
            while let Ok(true) = cursor.advance().await {
                if let Ok(doc_map) = cursor.deserialize_current() {
                    if let Ok(etiketler_str) = doc_map.get_str("etiketler") {
                        // Virgülle ayrılmış etiketleri parse et
                        for etiket in etiketler_str.split(',') {
                            let trimmed = etiket.trim();
                            if !trimmed.is_empty() {
                                etiketler_set.insert(trimmed.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Etiket listesi alınamadı: {}", e);
            return HttpResponse::InternalServerError().json(DocumentFiltersResponse {
                success: false,
                data: DocumentFiltersData {
                    belge_turu: belge_turu_list,
                    etiketler: vec![],
                },
                message: None,
                error: Some("Etiket listesi alınamadı".to_string()),
            });
        }
    }
    
    let mut etiketler_list: Vec<String> = etiketler_set.into_iter().collect();
    etiketler_list.sort();
    
    HttpResponse::Ok().json(DocumentFiltersResponse {
        success: true,
        data: DocumentFiltersData {
            belge_turu: belge_turu_list,
            etiketler: etiketler_list,
        },
        message: Some("Filtre listeleri başarıyla alındı".to_string()),
        error: None,
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_documents))
        .route("/filters", web::get().to(get_document_filters));
}

