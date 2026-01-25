use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Bson, Document as MongoDocument}};
use serde::{Deserialize, Serialize};
use crate::config::AppState;

#[derive(Deserialize)]
pub struct RecentRegulationsQueryV2 {
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct RecentRegulationYargitayResponse {
    #[serde(rename = "_id")]
    pub id: String,
    pub pdf_adi: String,
    pub kurum_id: String,
    pub belge_turu: String,
    pub belge_durumu: String,
    pub url_slug: String,
    pub status: String,
    pub sayfa_sayisi: i32,
    pub dosya_boyutu_mb: f64,
    pub olusturulma_tarihi: String,
    pub pdf_url: String,
    pub daire: String,
    #[serde(rename = "esasNo")]
    pub esas_no: String,
    #[serde(rename = "kararNo")]
    pub karar_no: String,
    #[serde(rename = "kararTarihi")]
    pub karar_tarihi: String,
    pub etiketler: String,
    pub icerik: String,
    pub icerik_text: String,
}

#[derive(Debug, Serialize)]
pub struct RecentRegulationsYargitayListResponse {
    pub success: bool,
    pub data: Vec<RecentRegulationYargitayResponse>,
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn get_string_field(doc: &MongoDocument, field: &str) -> String {
    doc.get_str(field).unwrap_or("").to_string()
}

fn get_i32_field(doc: &MongoDocument, field: &str) -> i32 {
    if let Ok(value) = doc.get_i32(field) {
        return value;
    }
    if let Ok(value) = doc.get_i64(field) {
        return value as i32;
    }
    if let Ok(value) = doc.get_str(field) {
        return value.parse::<i32>().unwrap_or(0);
    }
    0
}

fn get_f64_field(doc: &MongoDocument, field: &str) -> f64 {
    if let Ok(value) = doc.get_f64(field) {
        return value;
    }
    if let Ok(value) = doc.get_i32(field) {
        return value as f64;
    }
    if let Ok(value) = doc.get_i64(field) {
        return value as f64;
    }
    if let Ok(value) = doc.get_str(field) {
        return value.parse::<f64>().unwrap_or(0.0);
    }
    0.0
}

fn get_date_string(doc: &MongoDocument, field: &str) -> String {
    match doc.get(field) {
        Some(Bson::DateTime(dt)) => dt.try_to_rfc3339_string().unwrap_or_default(),
        Some(Bson::String(value)) => value.to_string(),
        Some(Bson::Int64(value)) => value.to_string(),
        Some(Bson::Int32(value)) => value.to_string(),
        _ => String::new(),
    }
}

pub async fn get_recent_regulations_yargitay(
    state: web::Data<AppState>,
    query: web::Query<RecentRegulationsQueryV2>,
) -> HttpResponse {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("yargitay");

    let limit = query.limit.unwrap_or(50).min(1000);
    let pipeline = vec![
        doc! { "$sort": { "olusturulma_tarihi": -1 } },
        doc! { "$limit": limit as i64 },
    ];

    let mut cursor = match metadata_collection.aggregate(pipeline, None).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("MongoDB aggregation hatası: {}", e);
            return HttpResponse::InternalServerError().json(RecentRegulationsYargitayListResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Yargıtay mevzuatları alınamadı".to_string()),
            });
        }
    };

    let mut regulations: Vec<RecentRegulationYargitayResponse> = Vec::new();

    while let Ok(true) = cursor.advance().await {
        if let Ok(doc_map) = cursor.deserialize_current() {
            let id = doc_map
                .get_object_id("_id")
                .map(|oid| oid.to_hex())
                .unwrap_or_default();

            regulations.push(RecentRegulationYargitayResponse {
                id,
                pdf_adi: get_string_field(&doc_map, "pdf_adi"),
                kurum_id: get_string_field(&doc_map, "kurum_id"),
                belge_turu: get_string_field(&doc_map, "belge_turu"),
                belge_durumu: get_string_field(&doc_map, "belge_durumu"),
                url_slug: get_string_field(&doc_map, "url_slug"),
                status: get_string_field(&doc_map, "status"),
                sayfa_sayisi: get_i32_field(&doc_map, "sayfa_sayisi"),
                dosya_boyutu_mb: get_f64_field(&doc_map, "dosya_boyutu_mb"),
                olusturulma_tarihi: get_date_string(&doc_map, "olusturulma_tarihi"),
                pdf_url: get_string_field(&doc_map, "pdf_url"),
                daire: get_string_field(&doc_map, "daire"),
                esas_no: get_string_field(&doc_map, "esasNo"),
                karar_no: get_string_field(&doc_map, "kararNo"),
                karar_tarihi: get_string_field(&doc_map, "kararTarihi"),
                etiketler: get_string_field(&doc_map, "etiketler"),
                icerik: get_string_field(&doc_map, "icerik"),
                icerik_text: get_string_field(&doc_map, "icerik_text"),
            });
        }
    }

    let count = regulations.len() as u64;

    HttpResponse::Ok().json(RecentRegulationsYargitayListResponse {
        success: true,
        data: regulations,
        count,
        message: Some("İşlem başarılı".to_string()),
        error: None,
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/recent", web::get().to(get_recent_regulations_yargitay));
}
