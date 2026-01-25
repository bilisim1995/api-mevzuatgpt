use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchResult {
    pub id: String,
    pub pdf_adi: String,
    pub kurum_adi: String,
    pub match_type: String,
    pub content_preview: String,
    pub relevance_percentage: u64,
    pub match_count: u64,
    pub url_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belge_yayin_tarihi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etiketler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aciklama: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belge_turu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belge_durumu: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchResponse {
    pub success: bool,
    pub data: Vec<SearchResult>,
    pub count: u64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchResultV2 {
    pub id: String,
    pub pdf_adi: String,
    pub kurum_adi: String,
    pub match_type: String,
    pub content_preview: String,
    pub relevance_percentage: u64,
    pub match_count: u64,
    pub url_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belge_yayin_tarihi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etiketler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belge_turu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belge_durumu: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SearchResponseV2 {
    pub success: bool,
    pub data: Vec<SearchResultV2>,
    pub count: u64,
    pub message: String,
}
