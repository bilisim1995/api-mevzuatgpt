// Utility fonksiyonları gelecekte kullanılmak üzere hazırlanmıştır
// Şu an için kullanılmıyor, ancak ileride gerekebilir

#[allow(dead_code)]
pub fn json_response<T: serde::Serialize>(data: T) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(data)
}

#[allow(dead_code)]
pub fn error_response(message: &str, status_code: u16) -> actix_web::HttpResponse {
    actix_web::HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status_code).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    )
    .json(serde_json::json!({
        "error": message
    }))
}

#[allow(dead_code)]
pub fn success_response(message: &str) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": message
    }))
}

