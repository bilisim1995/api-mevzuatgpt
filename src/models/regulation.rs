use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RecentRegulationResponse {
    pub pdf_adi: String,
    pub kurum_adi: String,
    pub aciklama: String,
    pub olusturulma_tarihi: String,
    pub belge_turu: String,
    pub url_slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RecentRegulationsListResponse {
    pub success: bool,
    pub data: Vec<RecentRegulationResponse>,
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

