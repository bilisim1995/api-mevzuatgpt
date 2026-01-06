use serde::{Deserialize, Serialize};

// Bu modül veri modellerinizi içerecek
// Her model için ayrı dosya oluşturabilirsiniz

pub mod kurum;
pub mod document;
pub mod announcement;
pub mod link;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

