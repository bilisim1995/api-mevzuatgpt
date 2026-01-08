use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SitemapInstitution {
    pub kurum_adi: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SitemapDocument {
    pub id: String,
    pub url_slug: String,
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

