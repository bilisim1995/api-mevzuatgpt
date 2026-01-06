use actix_web::web;
use crate::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health::health_check))
            .service(web::scope("/kurumlar").configure(handlers::kurum::routes))
            .service(web::scope("/v1/documents").configure(handlers::document::routes))
            // Yeni route'lar buraya eklenecek
    );
}

