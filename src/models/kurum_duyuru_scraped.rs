use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DuyuruItem {
    pub baslik: String,
    pub link: String,
    pub tarih: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KurumDuyuruScrapedResponse {
    pub success: bool,
    pub data: Vec<DuyuruItem>,
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

