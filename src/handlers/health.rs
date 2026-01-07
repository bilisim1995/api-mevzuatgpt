use actix_web::{web, HttpResponse};
use serde_json::json;
use crate::config::AppState;

pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    // MongoDB bağlantısını kontrol et
    let mongodb_status = match state.db.list_collection_names(None).await {
        Ok(_) => {
            json!({
                "status": "connected",
                "message": "MongoDB bağlantısı başarılı"
            })
        }
        Err(e) => {
            log::error!("MongoDB bağlantı hatası: {}", e);
            json!({
                "status": "disconnected",
                "message": format!("MongoDB bağlantı hatası: {}", e)
            })
        }
    };

    let server_status = json!({
        "status": "running",
        "message": "Sunucu çalışıyor"
    });

    let success = mongodb_status["status"] == "connected";

    HttpResponse::Ok().json(json!({
        "success": success,
        "server": server_status,
        "mongodb": mongodb_status,
        "message": if success {
            "Sunucu ve MongoDB bağlantısı başarılı"
        } else {
            "Sunucu çalışıyor ancak MongoDB bağlantısı başarısız"
        }
    }))
}

