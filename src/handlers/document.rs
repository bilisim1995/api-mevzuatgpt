use actix_web::{web, HttpResponse, http::StatusCode};
use mongodb::{Collection, bson::{doc, oid::ObjectId, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::document::{
    DocumentResponse, DocumentsListResponse,
    DocumentDetailResponse, DocumentDetailData, DocumentMetadata, DocumentContent
};
use crate::models::document_filters::{DocumentFiltersResponse, DocumentFiltersData};
use std::collections::HashSet;
use regex;
use chrono::Utc;
use futures::{future, FutureExt};

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
    let limit = query.limit.unwrap_or(10000).min(10000); // Maksimum 10000
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

pub async fn get_document_by_slug(
    state: web::Data<AppState>,
    slug: web::Path<String>,
) -> HttpResponse {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");
    let content_collection: Collection<MongoDocument> = state.db.collection("content");
    let kurum_collection: Collection<MongoDocument> = state.db.collection("kurumlar");

    // Metadata'yı url_slug ile bul
    let metadata_doc = match metadata_collection
        .find_one(doc! { "url_slug": slug.as_str() }, None)
        .await
    {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            return HttpResponse::build(StatusCode::NOT_FOUND).json(DocumentDetailResponse {
                success: false,
                data: DocumentDetailData {
                    metadata: DocumentMetadata {
                        id: String::new(),
                        kurum_id: String::new(),
                        kurum_adi: String::new(),
                        kurum_logo: String::new(),
                        kurum_aciklama: String::new(),
                        pdf_adi: String::new(),
                        etiketler: String::new(),
                        belge_yayin_tarihi: String::new(),
                        belge_durumu: String::new(),
                        aciklama: String::new(),
                        url_slug: String::new(),
                        belge_turu: String::new(),
                        anahtar_kelimeler: String::new(),
                        status: String::new(),
                        sayfa_sayisi: 0,
                        dosya_boyutu_mb: 0.0,
                        pdf_url: String::new(),
                    },
                    content: DocumentContent {
                        id: String::new(),
                        metadata_id: String::new(),
                        icerik: String::new(),
                        olusturulma_tarihi: String::new(),
                    },
                    kurum_adi: String::new(),
                    kurum_logo: String::new(),
                    kurum_aciklama: String::new(),
                },
                message: "Belge bulunamadı".to_string(),
            });
        }
        Err(e) => {
            log::error!("MongoDB metadata sorgu hatası: {}", e);
            return HttpResponse::InternalServerError().json(DocumentDetailResponse {
                success: false,
                data: DocumentDetailData {
                    metadata: DocumentMetadata {
                        id: String::new(),
                        kurum_id: String::new(),
                        kurum_adi: String::new(),
                        kurum_logo: String::new(),
                        kurum_aciklama: String::new(),
                        pdf_adi: String::new(),
                        etiketler: String::new(),
                        belge_yayin_tarihi: String::new(),
                        belge_durumu: String::new(),
                        aciklama: String::new(),
                        url_slug: String::new(),
                        belge_turu: String::new(),
                        anahtar_kelimeler: String::new(),
                        status: String::new(),
                        sayfa_sayisi: 0,
                        dosya_boyutu_mb: 0.0,
                        pdf_url: String::new(),
                    },
                    content: DocumentContent {
                        id: String::new(),
                        metadata_id: String::new(),
                        icerik: String::new(),
                        olusturulma_tarihi: String::new(),
                    },
                    kurum_adi: String::new(),
                    kurum_logo: String::new(),
                    kurum_aciklama: String::new(),
                },
                message: "Belge alınamadı".to_string(),
            });
        }
    };

    // Metadata'dan bilgileri çıkar
    let metadata_id = metadata_doc
        .get_object_id("_id")
        .map(|oid| oid.to_hex())
        .unwrap_or_default();

    let kurum_id = metadata_doc
        .get_str("kurum_id")
        .unwrap_or("")
        .to_string();

    let pdf_adi = metadata_doc
        .get_str("pdf_adi")
        .unwrap_or("")
        .to_string();

    let belge_yayin_tarihi = metadata_doc
        .get_str("belge_yayin_tarihi")
        .unwrap_or("")
        .to_string();

    let etiketler = metadata_doc
        .get_str("etiketler")
        .unwrap_or("")
        .to_string();

    let aciklama = metadata_doc
        .get_str("aciklama")
        .unwrap_or("")
        .to_string();

    let belge_turu = metadata_doc
        .get_str("belge_turu")
        .unwrap_or("")
        .to_string();

    let belge_durumu = metadata_doc
        .get_str("belge_durumu")
        .unwrap_or("")
        .to_string();

    let url_slug = metadata_doc
        .get_str("url_slug")
        .unwrap_or("")
        .to_string();

    let anahtar_kelimeler = metadata_doc
        .get_str("anahtar_kelimeler")
        .unwrap_or("")
        .to_string();

    let status = metadata_doc
        .get_str("status")
        .unwrap_or("")
        .to_string();

    let sayfa_sayisi = metadata_doc
        .get_i32("sayfa_sayisi")
        .unwrap_or(0);

    let dosya_boyutu_mb = metadata_doc
        .get_f64("dosya_boyutu_mb")
        .unwrap_or(0.0);

    let pdf_url = metadata_doc
        .get_str("pdf_url")
        .unwrap_or("")
        .to_string();

    // Kurum ve Content sorgularını paralel çalıştır (performans iyileştirmesi)
    let kurum_oid_result = ObjectId::parse_str(&kurum_id);
    let metadata_oid = metadata_doc.get_object_id("_id").ok();
    
    // Paralel sorguları hazırla
    let kurum_future = if let Ok(kurum_oid) = kurum_oid_result {
        kurum_collection.find_one(doc! { "_id": kurum_oid }, None).boxed()
    } else {
        future::ready(Ok(None::<MongoDocument>)).boxed()
    };
    
    let content_future = if let Some(oid) = metadata_oid {
        // Önce ObjectId ile dene
        content_collection.find_one(doc! { "metadata_id": oid }, None).boxed()
    } else {
        // String ile dene
        content_collection.find_one(doc! { "metadata_id": &metadata_id }, None).boxed()
    };
    
    // Paralel çalıştır
    let (kurum_result, content_result) = future::join(kurum_future, content_future).await;
    
    // Kurum bilgilerini işle
    let (kurum_adi, kurum_logo, kurum_aciklama) = match kurum_result {
        Ok(Some(kurum_doc)) => {
            let kurum_adi = kurum_doc
                .get_str("kurum_adi")
                .or_else(|_| kurum_doc.get_str("kurumAdi"))
                .unwrap_or("")
                .to_string();
            let kurum_logo = kurum_doc
                .get_str("kurum_logo")
                .or_else(|_| kurum_doc.get_str("kurumLogo"))
                .unwrap_or("")
                .to_string();
            let kurum_aciklama = kurum_doc
                .get_str("aciklama")
                .or_else(|_| kurum_doc.get_str("kurumAciklama"))
                .unwrap_or("")
                .to_string();
            (kurum_adi, kurum_logo, kurum_aciklama)
        }
        _ => (String::new(), String::new(), String::new()),
    };
    
    // Content'i işle
    let (content_id, icerik, content_olusturulma_tarihi) = match content_result {
        Ok(Some(content_doc)) => {
            let content_id = content_doc
                .get_object_id("_id")
                .map(|oid| oid.to_hex())
                .unwrap_or_default();
            
            // icerik alanını farklı isimlerle deneyelim
            let icerik = content_doc
                .get_str("icerik")
                .or_else(|_| content_doc.get_str("content"))
                .or_else(|_| content_doc.get_str("text"))
                .unwrap_or("")
                .to_string();
            
            // olusturulma_tarihi'ni ISO formatına çevir
            let olusturulma_tarihi = match content_doc
                .get_str("olusturulma_tarihi")
                .or_else(|_| content_doc.get_str("created_at"))
            {
                Ok(s) => {
                    if s.contains('T') || s.contains('Z') {
                        s.to_string()
                    } else {
                        format!("{}T00:00:00Z", s)
                    }
                }
                Err(_) => {
                    match metadata_doc.get_str("olusturulma_tarihi") {
                        Ok(s) => {
                            if s.contains('T') || s.contains('Z') {
                                s.to_string()
                            } else {
                                format!("{}T00:00:00Z", s)
                            }
                        }
                        Err(_) => String::from(Utc::now().to_rfc3339()),
                    }
                }
            };
            
            (content_id, icerik, olusturulma_tarihi)
        }
        _ => {
            // Content bulunamadıysa string ile de dene (fallback)
            match content_collection
                .find_one(doc! { "metadata_id": &metadata_id }, None)
                .await
            {
                Ok(Some(content_doc)) => {
                    let content_id = content_doc
                        .get_object_id("_id")
                        .map(|oid| oid.to_hex())
                        .unwrap_or_default();
                    
                    let icerik = content_doc
                        .get_str("icerik")
                        .or_else(|_| content_doc.get_str("content"))
                        .or_else(|_| content_doc.get_str("text"))
                        .unwrap_or("")
                        .to_string();
                    
                    let olusturulma_tarihi = match content_doc
                        .get_str("olusturulma_tarihi")
                        .or_else(|_| content_doc.get_str("created_at"))
                    {
                        Ok(s) => {
                            if s.contains('T') || s.contains('Z') {
                                s.to_string()
                            } else {
                                format!("{}T00:00:00Z", s)
                            }
                        }
                        Err(_) => {
                            match metadata_doc.get_str("olusturulma_tarihi") {
                                Ok(s) => {
                                    if s.contains('T') || s.contains('Z') {
                                        s.to_string()
                                    } else {
                                        format!("{}T00:00:00Z", s)
                                    }
                                }
                                Err(_) => String::from(Utc::now().to_rfc3339()),
                            }
                        }
                    };
                    
                    (content_id, icerik, olusturulma_tarihi)
                }
                _ => {
                    log::warn!("Content bulunamadı - metadata_id: {}", metadata_id);
                    (String::new(), String::new(), String::new())
                }
            }
        }
    };

    HttpResponse::Ok().json(DocumentDetailResponse {
        success: true,
        data: DocumentDetailData {
            metadata: DocumentMetadata {
                id: metadata_id.clone(),
                kurum_id: kurum_id.clone(),
                kurum_adi: kurum_adi.clone(),
                kurum_logo: kurum_logo.clone(),
                kurum_aciklama: kurum_aciklama.clone(),
                pdf_adi,
                etiketler,
                belge_yayin_tarihi,
                belge_durumu,
                aciklama,
                url_slug,
                belge_turu,
                anahtar_kelimeler,
                status,
                sayfa_sayisi,
                dosya_boyutu_mb,
                pdf_url,
            },
            content: DocumentContent {
                id: content_id,
                metadata_id,
                icerik,
                olusturulma_tarihi: content_olusturulma_tarihi,
            },
            kurum_adi,
            kurum_logo,
            kurum_aciklama,
        },
        message: "İşlem başarılı".to_string(),
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_documents))
        .route("/filters", web::get().to(get_document_filters))
        .route("/{slug}", web::get().to(get_document_by_slug));
}

