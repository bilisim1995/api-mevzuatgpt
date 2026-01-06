use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub pdf_adi: String,
    pub kurum_id: String,
    pub belge_turu: String,
    pub belge_durumu: String,
    pub belge_yayin_tarihi: String,
    pub yururluluk_tarihi: Option<String>,
    pub etiketler: String,
    pub anahtar_kelimeler: String,
    pub aciklama: String,
    pub url_slug: String,
    pub status: String,
    pub sayfa_sayisi: i32,
    pub dosya_boyutu_mb: f64,
    pub yukleme_tarihi: Option<String>,
    pub olusturulma_tarihi: String,
    pub pdf_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    pub id: String,
    pub kurum_id: String,
    pub kurum_adi: String,
    pub kurum_logo: String,
    pub kurum_aciklama: String,
    pub pdf_adi: String,
    pub etiketler: String,
    pub belge_yayin_tarihi: String,
    pub belge_durumu: String,
    pub aciklama: String,
    pub url_slug: String,
    pub belge_turu: String,
    pub anahtar_kelimeler: String,
    pub status: String,
    pub sayfa_sayisi: i32,
    pub dosya_boyutu_mb: f64,
    pub pdf_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentsListResponse {
    pub success: bool,
    pub data: Vec<DocumentResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    pub message: String,
}

