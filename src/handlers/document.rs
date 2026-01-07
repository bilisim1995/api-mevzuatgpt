use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::document::{DocumentResponse, DocumentsListResponse};

#[derive(serde::Deserialize)]
pub struct DocumentQuery {
    pub kurum_id: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
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

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_documents));
}

