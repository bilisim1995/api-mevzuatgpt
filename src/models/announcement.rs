use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KurumDuyuru {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub kurum_id: String,
    pub duyuru_linki: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baslik: Option<String>, // Opsiyonel: Eğer koleksiyonda varsa kullanılır
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baslik: Option<String>, // Koleksiyondan okunur, yoksa null
    pub link: String,
    pub tarih: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementsListResponse {
    pub success: bool,
    pub data: Vec<AnnouncementResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    pub message: String,
}

