use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AutocompleteSuggestion {
    pub text: String,
    pub count: u64,
    pub r#type: String, // "title", "keyword", "tag", "institution"
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AutocompleteResponse {
    pub success: bool,
    pub suggestions: Vec<AutocompleteSuggestion>,
    pub message: String,
}

