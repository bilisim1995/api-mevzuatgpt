use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DocumentFiltersResponse {
    pub success: bool,
    pub data: DocumentFiltersData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DocumentFiltersData {
    pub belge_turu: Vec<String>,
    pub etiketler: Vec<String>,
}

