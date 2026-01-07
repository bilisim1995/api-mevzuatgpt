use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kurum {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub kurum_adi: String,
    pub kurum_logo: String,
    pub aciklama: String,
    pub olusturulma_tarihi: String,
    pub detsis: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KurumResponse {
    pub kurum_id: String,
    pub kurum_adi: String,
    pub kurum_logo: String,
    pub kurum_aciklama: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detsis: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstitutionsListResponse {
    pub success: bool,
    pub data: Vec<KurumResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    pub message: String,
}

