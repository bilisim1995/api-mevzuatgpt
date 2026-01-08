use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SitemapInstitution {
    pub kurum_adi: String,
    pub count: i32,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SitemapDocument {
    pub url_slug: String,
    pub pdf_adi: String,
    pub kurum_adi: String,
    pub belge_yayin_tarihi: String,
    pub olusturulma_tarihi: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SitemapInstitutionsResponse {
    pub success: bool,
    pub data: Vec<SitemapInstitution>,
    pub count: usize,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SitemapDocumentsResponse {
    pub success: bool,
    pub data: Vec<SitemapDocument>,
    pub count: usize,
    pub message: String,
}

