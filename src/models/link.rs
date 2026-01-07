use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KurumLink {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub kurum_id: String,
    pub baslik: String,
    pub aciklama: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LinkResponse {
    pub id: String,
    pub baslik: String,
    pub aciklama: String,
    pub url: String,
    pub kurum_id: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LinksListResponse {
    pub success: bool,
    pub data: Vec<LinkResponse>,
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

