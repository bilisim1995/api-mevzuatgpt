use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

pub fn json_response<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

pub fn error_response(message: &str, status_code: u16) -> HttpResponse {
    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status_code).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    )
    .json(json!({
        "error": message
    }))
}

pub fn success_response(message: &str) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": message
    }))
}

