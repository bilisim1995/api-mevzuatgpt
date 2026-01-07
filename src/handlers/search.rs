use actix_web::{web, HttpResponse, http::{StatusCode, header::HeaderValue}};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::search::{SearchResponse, SearchResult};
use regex::Regex;

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

    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");

    // Gelişmiş regex pattern oluştur (yakın eşleşmeler için)
    let regex_pattern = build_advanced_regex_pattern(search_query);
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

    // 1. Metadata'da arama yap (sadece pdf_adi ve aciklama)
    // Gelişmiş regex: her kelime için ayrı pattern (yakın eşleşme)
    let mongo_patterns = build_mongodb_regex_patterns(search_query);
    
    let mut match_filter = if mongo_patterns.is_empty() {
        // Tek kelime veya boş ise basit pattern
        doc! {
            "$or": [
                { "pdf_adi": doc! { "$regex": &regex_pattern, "$options": "i" } },
                { "aciklama": doc! { "$regex": &regex_pattern, "$options": "i" } }
            ]
        }
    } else if mongo_patterns.len() == 1 {
        // Tek kelime için basit pattern (daha hızlı)
        let pattern = &mongo_patterns[0];
        doc! {
            "$or": [
                { "pdf_adi": doc! { "$regex": pattern, "$options": "i" } },
                { "aciklama": doc! { "$regex": pattern, "$options": "i" } }
            ]
        }
    } else {
        // Çoklu kelime için: her kelime için ayrı $and koşulu
        let mut or_conditions = Vec::new();
        
        // pdf_adi için: tüm kelimeler geçmeli
        let mut pdf_adi_conditions = Vec::new();
        for pattern in &mongo_patterns {
            pdf_adi_conditions.push(doc! {
                "pdf_adi": doc! { "$regex": pattern, "$options": "i" }
            });
        }
        if !pdf_adi_conditions.is_empty() {
            or_conditions.push(doc! { "$and": pdf_adi_conditions });
        }
        
        // aciklama için: tüm kelimeler geçmeli
        let mut aciklama_conditions = Vec::new();
        for pattern in &mongo_patterns {
            aciklama_conditions.push(doc! {
                "aciklama": doc! { "$regex": pattern, "$options": "i" }
            });
        }
        if !aciklama_conditions.is_empty() {
            or_conditions.push(doc! { "$and": aciklama_conditions });
        }
        
        if or_conditions.is_empty() {
            doc! {
                "$or": [
                    { "pdf_adi": doc! { "$regex": &regex_pattern, "$options": "i" } },
                    { "aciklama": doc! { "$regex": &regex_pattern, "$options": "i" } }
                ]
            }
        } else {
            doc! { "$or": or_conditions }
        }
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

    // Metadata sonuçlarını al - $lookup ile kurum bilgilerini birleştir (N+1 query problemini çöz)
    let pipeline = vec![
        doc! { "$match": match_filter.clone() },
        doc! {
            "$addFields": {
                "kurum_id_object": {
                    "$cond": {
                        "if": { "$eq": [{ "$type": "$kurum_id" }, "string"] },
                        "then": { "$toObjectId": "$kurum_id" },
                        "else": "$kurum_id"
                    }
                }
            }
        },
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

    // Her metadata kaydı için detayları hesapla
    while let Ok(true) = cursor.advance().await {
        if let Ok(metadata_doc) = cursor.deserialize_current() {
            let id = metadata_doc
                .get_object_id("_id")
                .map(|oid| oid.to_hex())
                .unwrap_or_default();

            let pdf_adi = metadata_doc
                .get_str("pdf_adi")
                .unwrap_or("")
                .to_string();

            let url_slug = metadata_doc
                .get_str("url_slug")
                .unwrap_or("")
                .to_string();

            // Kurum adını $lookup ile gelen kurum_bilgisi'nden al
            let kurum_adi = metadata_doc
                .get_document("kurum_bilgisi")
                .ok()
                .and_then(|k| {
                    k.get_str("kurum_adi")
                        .or_else(|_| k.get_str("kurumAdi"))
                        .ok()
                })
                .unwrap_or("")
                .to_string();

            // Match type ve match count hesapla
            // MongoDB pattern'lerini kullanarak her kelime için ayrı kontrol
            let mongo_patterns = build_mongodb_regex_patterns(search_query);
            let mut match_types: Vec<String> = Vec::new();
            
            // Ağırlıklı puanlama için ayrı ayrı sayılar
            let mut title_count = 0u64;
            let mut content_count = 0u64;

            // Title match (pdf_adi) - tüm kelimeler geçmeli
            let pdf_adi_lower = pdf_adi.to_lowercase();
            let mut title_matches = true;
            
            if mongo_patterns.is_empty() {
                // Tek kelime veya basit pattern
                if regex_obj.is_match(&pdf_adi_lower) {
                    match_types.push("title".to_string());
                    title_count = regex_obj.find_iter(&pdf_adi_lower).count() as u64;
                }
            } else {
                // Çoklu kelime: her kelime için ayrı kontrol
                // MongoDB pattern formatı: ".*kelime.*" -> sadece "kelime" kısmını al
                for pattern in &mongo_patterns {
                    // Pattern'den kelimeyi çıkar: ".*kısa.*" -> "kısa"
                    let word = pattern.trim_start_matches(".*").trim_end_matches(".*");
                    if pdf_adi_lower.contains(&word.to_lowercase()) {
                        title_count += pdf_adi_lower.matches(&word.to_lowercase()).count() as u64;
                    } else {
                        title_matches = false;
                        break;
                    }
                }
                if title_matches && title_count > 0 {
                    match_types.push("title".to_string());
                }
            }

            // Content match (aciklama) - tüm kelimeler geçmeli
            let mut content_preview = String::new();
            if let Ok(aciklama) = metadata_doc.get_str("aciklama") {
                let aciklama_lower = aciklama.to_lowercase();
                let mut content_matches = true;
                
                if mongo_patterns.is_empty() {
                    // Tek kelime veya basit pattern
                    if regex_obj.is_match(&aciklama_lower) {
                        match_types.push("content".to_string());
                        content_count = regex_obj.find_iter(&aciklama_lower).count() as u64;
                        
                        // Content preview oluştur
                        if let Some(mat) = regex_obj.find(&aciklama_lower) {
                            let start = mat.start().saturating_sub(100);
                            let end = (mat.end() + 100).min(aciklama.len());
                            content_preview = format!("...{}...", &aciklama[start..end]);
                        } else {
                            content_preview = aciklama.chars().take(200).collect::<String>();
                            if aciklama.len() > 200 {
                                content_preview.push_str("...");
                            }
                        }
                    } else {
                        // Eşleşme yoksa da preview göster
                        content_preview = aciklama.chars().take(200).collect::<String>();
                        if aciklama.len() > 200 {
                            content_preview.push_str("...");
                        }
                    }
                } else {
                    // Çoklu kelime: her kelime için ayrı kontrol
                    // MongoDB pattern formatı: ".*kelime.*" -> sadece "kelime" kısmını al
                    for pattern in &mongo_patterns {
                        // Pattern'den kelimeyi çıkar: ".*kısa.*" -> "kısa"
                        let word = pattern.trim_start_matches(".*").trim_end_matches(".*");
                        if aciklama_lower.contains(&word.to_lowercase()) {
                            content_count += aciklama_lower.matches(&word.to_lowercase()).count() as u64;
                        } else {
                            content_matches = false;
                            break;
                        }
                    }
                    if content_matches && content_count > 0 {
                        match_types.push("content".to_string());
                        
                        // Content preview oluştur (ilk kelimeyi bul)
                        let first_word = mongo_patterns[0].trim_start_matches(".*").trim_end_matches(".*");
                        if let Some(pos) = aciklama_lower.find(&first_word.to_lowercase()) {
                            let start = pos.saturating_sub(100);
                            let end = (pos + first_word.len() + 100).min(aciklama.len());
                            content_preview = format!("...{}...", &aciklama[start..end]);
                        } else {
                            content_preview = aciklama.chars().take(200).collect::<String>();
                            if aciklama.len() > 200 {
                                content_preview.push_str("...");
                            }
                        }
                    } else {
                        // Eşleşme yoksa da preview göster
                        content_preview = aciklama.chars().take(200).collect::<String>();
                        if aciklama.len() > 200 {
                            content_preview.push_str("...");
                        }
                    }
                }
            }

            // Ağırlıklı puanlama ile relevance percentage hesapla
            // Title ağırlığı: 0.7, Content ağırlığı: 0.3
            const TITLE_WEIGHT: f64 = 0.7;
            const CONTENT_WEIGHT: f64 = 0.3;
            
            // Her alanın kendi base score'unu hesapla
            let title_base_score = if title_count > 0 {
                title_count as f64 / (title_count as f64 + 1.0)
            } else {
                0.0
            };
            
            let content_base_score = if content_count > 0 {
                content_count as f64 / (content_count as f64 + 1.0)
            } else {
                0.0
            };
            
            // Ağırlıklı ortalama
            let relevance_score = (title_base_score * TITLE_WEIGHT) + (content_base_score * CONTENT_WEIGHT);
            let relevance_percentage = (relevance_score * 100.0) as u64;
            
            // Toplam match count (hem title hem content için)
            let match_count = title_count + content_count;

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
                relevance_percentage,
                match_count,
                url_slug,
                belge_yayin_tarihi,
                etiketler,
                aciklama,
                belge_turu,
                belge_durumu,
            });
        }
    }

    // Relevance percentage'a göre sırala (yüksekten düşüğe)
    results.sort_by(|a, b| {
        b.relevance_percentage.cmp(&a.relevance_percentage)
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

// Gelişmiş regex pattern oluşturma fonksiyonu
// Türkçe karakter normalizasyonu ve partial matching için
fn build_advanced_regex_pattern(query: &str) -> String {
    // Türkçe karakterleri normalize et
    let normalized = normalize_turkish_chars(query);
    
    // Kelimelere ayır
    let words: Vec<&str> = normalized.split_whitespace().collect();
    
    if words.is_empty() {
        return format!(r"(?i){}", regex::escape(query));
    }
    
    // Her kelime için pattern oluştur
    let word_patterns: Vec<String> = words
        .iter()
        .filter(|w| w.len() >= 2) // Minimum 2 karakter
        .map(|word| {
            let escaped = regex::escape(word);
            // Partial matching: kelime başlangıcı veya içinde geçmesi
            // Türkçe karakter varyasyonlarını da kapsar
            format!(r"(?i).*?{}.*?", escaped)
        })
        .collect();
    
    if word_patterns.is_empty() {
        format!(r"(?i){}", regex::escape(query))
    } else {
        // Tüm kelimelerin geçmesi için AND mantığı (her kelime ayrı ayrı aranır)
        // MongoDB'de $and kullanacağız, bu yüzden her kelime için ayrı pattern
        word_patterns.join("")
    }
}

// Türkçe karakterleri normalize et (yakın eşleşme için)
fn normalize_turkish_chars(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'ı' | 'İ' | 'I' | 'i' => 'i',
            'ş' | 'Ş' => 's',
            'ğ' | 'Ğ' => 'g',
            'ü' | 'Ü' => 'u',
            'ö' | 'Ö' => 'o',
            'ç' | 'Ç' => 'c',
            _ => c.to_lowercase().next().unwrap_or(c),
        })
        .collect()
}

// Türkçe karakterli regex pattern oluştur
// Sadece Türkçe karakterleri destekler (normalize edilmiş versiyon yok)
fn build_turkish_char_pattern(word: &str) -> String {
    let word_lower = word.to_lowercase();
    // Sadece orijinal Türkçe karakterli versiyonu kullan
    regex::escape(&word_lower)
}


// MongoDB için gelişmiş regex pattern oluştur (her kelime için ayrı)
// Sadece Türkçe karakterleri destekler (normalize edilmiş versiyon yok)
fn build_mongodb_regex_patterns(query: &str) -> Vec<String> {
    let words: Vec<&str> = query.split_whitespace().collect();
    
    words
        .iter()
        .filter(|w| w.len() >= 2)
        .map(|word| {
            // Sadece Türkçe karakterli pattern (normalize edilmiş versiyon yok)
            let pattern = build_turkish_char_pattern(word);
            // MongoDB regex: başta, sonda veya ortada geçebilir
            format!(r".*{}.*", pattern)
        })
        .collect()
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(search));
}

