use actix_web::{web, HttpResponse, http::StatusCode};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::autocomplete::{AutocompleteResponse, AutocompleteSuggestion};

#[derive(serde::Deserialize)]
pub struct AutocompleteQuery {
    pub q: String,
    pub limit: Option<u64>,
    pub kurum_id: String, // Zorunlu
}

pub async fn get_autocomplete(
    state: web::Data<AppState>,
    query: web::Query<AutocompleteQuery>,
) -> HttpResponse {
    // q parametresi minimum 2 karakter olmalı
    if query.q.len() < 2 {
        return HttpResponse::build(StatusCode::BAD_REQUEST).json(AutocompleteResponse {
            success: false,
            suggestions: vec![],
            message: "Arama sorgusu en az 2 karakter olmalıdır".to_string(),
        });
    }

    // kurum_id zorunlu kontrolü
    if query.kurum_id.is_empty() {
        return HttpResponse::build(StatusCode::BAD_REQUEST).json(AutocompleteResponse {
            success: false,
            suggestions: vec![],
            message: "kurum_id parametresi zorunludur".to_string(),
        });
    }

    let limit = query.limit.unwrap_or(10).min(50); // Maksimum 50
    let search_query = query.q.trim();

    let mut suggestions: Vec<AutocompleteSuggestion> = Vec::new();

    // 1. Title (pdf_adi) önerileri
    let title_suggestions = get_title_suggestions(
        &state,
        search_query,
        limit,
        &query.kurum_id,
    ).await;
    suggestions.extend(title_suggestions);

    // 2. Keyword (anahtar_kelimeler) önerileri
    let keyword_suggestions = get_keyword_suggestions(
        &state,
        search_query,
        limit,
        &query.kurum_id,
    ).await;
    suggestions.extend(keyword_suggestions);

    // 3. Tag (etiketler) önerileri
    let tag_suggestions = get_tag_suggestions(
        &state,
        search_query,
        limit,
        &query.kurum_id,
    ).await;
    suggestions.extend(tag_suggestions);

    // 4. Content (icerik) önerileri
    let content_suggestions = get_content_suggestions(
        &state,
        search_query,
        limit,
        &query.kurum_id,
    ).await;
    suggestions.extend(content_suggestions);

    // Toplam limit'e göre sırala ve kes
    suggestions.sort_by(|a, b| b.count.cmp(&a.count));
    suggestions.truncate(limit as usize);

    HttpResponse::Ok().json(AutocompleteResponse {
        success: true,
        suggestions,
        message: "İşlem başarılı".to_string(),
    })
}

async fn get_title_suggestions(
    state: &AppState,
    search_query: &str,
    limit: u64,
    kurum_id: &str,
) -> Vec<AutocompleteSuggestion> {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");
    
    let match_filter = doc! {
        "pdf_adi": doc! {
            "$regex": format!(r"(?i){}", regex::escape(search_query)),
            "$options": "i"
        },
        "kurum_id": kurum_id
    };

    let pipeline = vec![
        doc! { "$match": match_filter },
        doc! {
            "$group": {
                "_id": "$pdf_adi",
                "count": { "$sum": 1 }
            }
        },
        doc! { "$sort": { "count": -1 } },
        doc! { "$limit": limit as i64 },
        doc! {
            "$project": {
                "text": "$_id",
                "count": 1,
                "_id": 0
            }
        },
    ];

    let mut suggestions = Vec::new();
    if let Ok(mut cursor) = metadata_collection.aggregate(pipeline, None).await {
        while let Ok(true) = cursor.advance().await {
            if let Ok(doc_map) = cursor.deserialize_current() {
                if let (Ok(text), Ok(count)) = (doc_map.get_str("text"), doc_map.get_i64("count")) {
                    suggestions.push(AutocompleteSuggestion {
                        text: text.to_string(),
                        count: count as u64,
                        r#type: "title".to_string(),
                    });
                }
            }
        }
    }
    suggestions
}

async fn get_keyword_suggestions(
    state: &AppState,
    search_query: &str,
    limit: u64,
    kurum_id: &str,
) -> Vec<AutocompleteSuggestion> {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");
    
    let match_filter = doc! {
        "anahtar_kelimeler": doc! {
            "$regex": format!(r"(?i){}", regex::escape(search_query)),
            "$options": "i"
        },
        "kurum_id": kurum_id
    };

    let pipeline = vec![
        doc! { "$match": match_filter },
        doc! {
            "$project": {
                "anahtar_kelimeler": 1
            }
        },
    ];

    let mut keyword_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut keyword_count: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    if let Ok(mut cursor) = metadata_collection.aggregate(pipeline, None).await {
        while let Ok(true) = cursor.advance().await {
            if let Ok(doc_map) = cursor.deserialize_current() {
                if let Ok(keywords_str) = doc_map.get_str("anahtar_kelimeler") {
                    for keyword in keywords_str.split(',') {
                        let trimmed = keyword.trim().to_lowercase();
                        if trimmed.contains(&search_query.to_lowercase()) && trimmed.len() >= 2 {
                            keyword_set.insert(trimmed.clone());
                            *keyword_count.entry(trimmed.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let mut suggestions: Vec<AutocompleteSuggestion> = keyword_count
        .into_iter()
        .map(|(text, count)| AutocompleteSuggestion {
            text,
            count,
            r#type: "keyword".to_string(),
        })
        .collect();
    
    suggestions.sort_by(|a, b| b.count.cmp(&a.count));
    suggestions.truncate(limit as usize);
    suggestions
}

async fn get_tag_suggestions(
    state: &AppState,
    search_query: &str,
    limit: u64,
    kurum_id: &str,
) -> Vec<AutocompleteSuggestion> {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");
    
    let match_filter = doc! {
        "etiketler": doc! {
            "$regex": format!(r"(?i){}", regex::escape(search_query)),
            "$options": "i"
        },
        "kurum_id": kurum_id
    };

    let pipeline = vec![
        doc! { "$match": match_filter },
        doc! {
            "$project": {
                "etiketler": 1
            }
        },
    ];

    let mut tag_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tag_count: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

    if let Ok(mut cursor) = metadata_collection.aggregate(pipeline, None).await {
        while let Ok(true) = cursor.advance().await {
            if let Ok(doc_map) = cursor.deserialize_current() {
                if let Ok(etiketler_str) = doc_map.get_str("etiketler") {
                    for etiket in etiketler_str.split(',') {
                        let trimmed = etiket.trim().to_lowercase();
                        if trimmed.contains(&search_query.to_lowercase()) && trimmed.len() >= 2 {
                            tag_set.insert(trimmed.clone());
                            *tag_count.entry(trimmed.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let mut suggestions: Vec<AutocompleteSuggestion> = tag_count
        .into_iter()
        .map(|(text, count)| AutocompleteSuggestion {
            text,
            count,
            r#type: "tag".to_string(),
        })
        .collect();
    
    suggestions.sort_by(|a, b| b.count.cmp(&a.count));
    suggestions.truncate(limit as usize);
    suggestions
}

async fn get_content_suggestions(
    state: &AppState,
    search_query: &str,
    limit: u64,
    kurum_id: &str,
) -> Vec<AutocompleteSuggestion> {
    let content_collection: Collection<MongoDocument> = state.db.collection("content");
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");
    
    // Content koleksiyonunda icerik alanında arama yap
    // content koleksiyonunda kurum_id yoksa, metadata ile join yaparak kurum_id'ye göre filtrele
    let match_filter = doc! {
        "icerik": doc! {
            "$regex": format!(r"(?i){}", regex::escape(search_query)),
            "$options": "i"
        }
    };

    // Önce metadata'da bu kurum_id'ye ait belgelerin url_slug'larını bul
    let metadata_pipeline = vec![
        doc! {
            "$match": {
                "kurum_id": kurum_id
            }
        },
        doc! {
            "$project": {
                "url_slug": 1
            }
        }
    ];

    let mut url_slugs: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(mut cursor) = metadata_collection.aggregate(metadata_pipeline, None).await {
        while let Ok(true) = cursor.advance().await {
            if let Ok(doc_map) = cursor.deserialize_current() {
                if let Ok(url_slug) = doc_map.get_str("url_slug") {
                    url_slugs.insert(url_slug.to_string());
                }
            }
        }
    }

    if url_slugs.is_empty() {
        return Vec::new();
    }

    // Content koleksiyonunda bu url_slug'lara sahip ve icerik alanında arama sorgusu geçen kayıtları bul
    let url_slug_vec: Vec<&String> = url_slugs.iter().collect();
    let content_pipeline = vec![
        doc! {
            "$match": {
                "$and": [
                    match_filter.clone(),
                    doc! {
                        "url_slug": {
                            "$in": &url_slug_vec
                        }
                    }
                ]
            }
        },
        doc! {
            "$group": {
                "_id": "$url_slug",
                "count": { "$sum": 1 }
            }
        },
        doc! { "$sort": { "count": -1 } },
        doc! { "$limit": limit as i64 },
        doc! {
            "$project": {
                "url_slug": "$_id",
                "count": 1,
                "_id": 0
            }
        },
    ];

    let mut suggestions = Vec::new();
    if let Ok(mut cursor) = content_collection.aggregate(content_pipeline, None).await {
        while let Ok(true) = cursor.advance().await {
            if let Ok(doc_map) = cursor.deserialize_current() {
                // url_slug'dan pdf_adi'yi bul
                if let Ok(url_slug) = doc_map.get_str("url_slug") {
                    let pdf_adi_filter = doc! {
                        "url_slug": url_slug,
                        "kurum_id": kurum_id
                    };
                    
                    if let Ok(Some(metadata_doc)) = metadata_collection.find_one(pdf_adi_filter, None).await {
                        if let Ok(pdf_adi) = metadata_doc.get_str("pdf_adi") {
                            let count = doc_map.get_i64("count").unwrap_or(1) as u64;
                            suggestions.push(AutocompleteSuggestion {
                                text: pdf_adi.to_string(),
                                count,
                                r#type: "content".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    suggestions
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_autocomplete));
}

