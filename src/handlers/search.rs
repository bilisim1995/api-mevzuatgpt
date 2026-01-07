use actix_web::{web, HttpResponse, http::{StatusCode, header::HeaderValue}};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::search::{SearchResponse, SearchResult};
use regex::Regex;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub kurum_id: Option<String>,
}

pub async fn search(
    state: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> HttpResponse {
    // q parametresi boş olamaz
    if query.q.trim().is_empty() {
        return HttpResponse::build(StatusCode::BAD_REQUEST).json(SearchResponse {
            success: false,
            data: vec![],
            count: 0,
            message: "Arama sorgusu boş olamaz".to_string(),
        });
    }

    let limit = query.limit.unwrap_or(10000);
    let offset = query.offset.unwrap_or(0);
    let search_query = query.q.trim();
    let search_query_lower = search_query.to_lowercase();

    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");
    let content_collection: Collection<MongoDocument> = state.db.collection("content");
    let kurum_collection: Collection<MongoDocument> = state.db.collection("kurumlar");

    // Regex pattern oluştur (case-insensitive)
    let regex_pattern = format!(r"(?i){}", regex::escape(search_query));
    let regex_obj = match Regex::new(&regex_pattern) {
        Ok(re) => re,
        Err(_) => {
            return HttpResponse::InternalServerError().json(SearchResponse {
                success: false,
                data: vec![],
                count: 0,
                message: "Geçersiz arama sorgusu".to_string(),
            });
        }
    };

    // 1. Metadata'da arama yap (pdf_adi, anahtar_kelimeler, etiketler, aciklama)
    let mut match_filter = doc! {
        "$or": [
            { "pdf_adi": doc! { "$regex": &regex_pattern, "$options": "i" } },
            { "anahtar_kelimeler": doc! { "$regex": &regex_pattern, "$options": "i" } },
            { "etiketler": doc! { "$regex": &regex_pattern, "$options": "i" } },
            { "aciklama": doc! { "$regex": &regex_pattern, "$options": "i" } }
        ]
    };

    if let Some(kurum_id) = &query.kurum_id {
        match_filter.insert("kurum_id", kurum_id);
    }

    // Toplam sayıyı hesapla
    let total_count = match metadata_collection.count_documents(match_filter.clone(), None).await {
        Ok(count) => count,
        Err(e) => {
            log::error!("MongoDB count hatası: {}", e);
            return HttpResponse::InternalServerError().json(SearchResponse {
                success: false,
                data: vec![],
                count: 0,
                message: "Arama yapılamadı".to_string(),
            });
        }
    };

    // Metadata sonuçlarını al
    let pipeline = vec![
        doc! { "$match": match_filter.clone() },
        doc! { "$sort": { "olusturulma_tarihi": -1 } },
        doc! { "$skip": offset as i64 },
        doc! { "$limit": limit as i64 },
    ];

    let mut cursor = match metadata_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("MongoDB aggregation hatası: {}", e);
            return HttpResponse::InternalServerError().json(SearchResponse {
                success: false,
                data: vec![],
                count: 0,
                message: "Arama yapılamadı".to_string(),
            });
        }
    };

    let mut results: Vec<SearchResult> = Vec::new();
    let mut url_slug_to_metadata: HashMap<String, MongoDocument> = HashMap::new();

    // Metadata sonuçlarını topla
    while let Ok(true) = cursor.advance().await {
        if let Ok(doc_map) = cursor.deserialize_current() {
            if let Ok(url_slug) = doc_map.get_str("url_slug") {
                url_slug_to_metadata.insert(url_slug.to_string(), doc_map.clone());
            }
        }
    }

    // Her metadata kaydı için detayları hesapla
    for (url_slug, metadata_doc) in url_slug_to_metadata.iter() {
        let id = metadata_doc
            .get_object_id("_id")
            .map(|oid| oid.to_hex())
            .unwrap_or_default();

        let pdf_adi = metadata_doc
            .get_str("pdf_adi")
            .unwrap_or("")
            .to_string();

        let kurum_id = metadata_doc
            .get_str("kurum_id")
            .unwrap_or("")
            .to_string();

        // Kurum adını bul
        let kurum_adi = if let Ok(kurum_oid) = mongodb::bson::oid::ObjectId::parse_str(&kurum_id) {
            if let Ok(Some(kurum_doc)) = kurum_collection.find_one(
                doc! { "_id": kurum_oid },
                None
            ).await {
                kurum_doc.get_str("kurum_adi").unwrap_or("").to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // Match type ve match count hesapla
        let mut match_types: Vec<String> = Vec::new();
        let mut match_count = 0u64;

        // Title match
        if regex_obj.is_match(&pdf_adi.to_lowercase()) {
            match_types.push("title".to_string());
            match_count += regex_obj.find_iter(&pdf_adi.to_lowercase()).count() as u64;
        }

        // Keyword match
        if let Ok(keywords) = metadata_doc.get_str("anahtar_kelimeler") {
            if regex_obj.is_match(&keywords.to_lowercase()) {
                match_types.push("keyword".to_string());
                match_count += regex_obj.find_iter(&keywords.to_lowercase()).count() as u64;
            }
        }

        // Tag match
        if let Ok(etiketler) = metadata_doc.get_str("etiketler") {
            if regex_obj.is_match(&etiketler.to_lowercase()) {
                match_types.push("tag".to_string());
                match_count += regex_obj.find_iter(&etiketler.to_lowercase()).count() as u64;
            }
        }

        // Content match - content koleksiyonunda ara
        let mut content_preview = String::new();
        let content_match_filter = doc! {
            "url_slug": url_slug,
            "icerik": doc! { "$regex": &regex_pattern, "$options": "i" }
        };

        if let Ok(Some(content_doc)) = content_collection.find_one(content_match_filter, None).await {
            if let Ok(icerik) = content_doc.get_str("icerik") {
                match_types.push("content".to_string());
                match_count += regex_obj.find_iter(&icerik.to_lowercase()).count() as u64;
                
                // Content preview oluştur (arama teriminin geçtiği kısmı al)
                let icerik_lower = icerik.to_lowercase();
                if let Some(mat) = regex_obj.find(&icerik_lower) {
                    let start = mat.start().saturating_sub(100);
                    let end = (mat.end() + 100).min(icerik.len());
                    let preview = &icerik[start..end];
                    content_preview = format!("...{}...", preview);
                } else {
                    content_preview = icerik.chars().take(200).collect::<String>();
                    if icerik.len() > 200 {
                        content_preview.push_str("...");
                    }
                }
            }
        }

        // Eğer content preview yoksa, aciklama'dan al
        if content_preview.is_empty() {
            if let Ok(aciklama) = metadata_doc.get_str("aciklama") {
                if regex_obj.is_match(&aciklama.to_lowercase()) {
                    let start = aciklama.to_lowercase().find(&search_query_lower)
                        .unwrap_or(0)
                        .saturating_sub(50);
                    let end = (start + 200).min(aciklama.len());
                    content_preview = format!("...{}...", &aciklama[start..end]);
                } else {
                    content_preview = aciklama.chars().take(200).collect::<String>();
                    if aciklama.len() > 200 {
                        content_preview.push_str("...");
                    }
                }
            }
        }

        // Relevance score hesapla (0-1 arası)
        // Match count ve match type sayısına göre
        let match_type_count = match_types.len() as f64;
        let base_score = (match_count as f64) / (match_count as f64 + 1.0);
        let type_bonus = match_type_count * 0.2; // Her match type için 0.2 bonus
        let relevance_score = (base_score + type_bonus).min(1.0);
        let relevance_percentage = (relevance_score * 100.0) as u64;

        let belge_yayin_tarihi = metadata_doc
            .get_str("belge_yayin_tarihi")
            .ok()
            .map(|s| s.to_string());

        let etiketler = metadata_doc
            .get_str("etiketler")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let aciklama = metadata_doc
            .get_str("aciklama")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let belge_turu = metadata_doc
            .get_str("belge_turu")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let belge_durumu = metadata_doc
            .get_str("belge_durumu")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        results.push(SearchResult {
            id,
            pdf_adi,
            kurum_adi,
            match_type: match_types.join(","),
            content_preview,
            relevance_score,
            relevance_percentage,
            match_count,
            url_slug: url_slug.clone(),
            belge_yayin_tarihi,
            etiketler,
            aciklama,
            belge_turu,
            belge_durumu,
        });
    }

    // Relevance score'a göre sırala (yüksekten düşüğe)
    results.sort_by(|a, b| {
        b.relevance_score.partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Response oluştur
    let mut response = HttpResponse::Ok().json(SearchResponse {
        success: true,
        data: results,
        count: total_count,
        message: "İşlem başarılı".to_string(),
    });

    // X-Total-Count header'ını ekle
    if let Ok(header_value) = HeaderValue::from_str(&total_count.to_string()) {
        response.headers_mut().insert(
            actix_web::http::header::HeaderName::from_static("x-total-count"),
            header_value,
        );
    }

    response
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(search));
}

