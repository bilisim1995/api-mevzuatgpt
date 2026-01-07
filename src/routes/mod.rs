use actix_web::web;
use crate::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health::health_check))
            .service(web::scope("/v1/institutions").configure(handlers::kurum::routes))
            .service(web::scope("/v1/documents").configure(handlers::document::routes))
            .service(web::scope("/v1/announcements").configure(handlers::announcement::routes))
            .service(web::scope("/v1/links").configure(handlers::link::routes))
            .service(web::scope("/v1/regulations").configure(handlers::regulation::routes))
            .service(web::scope("/v1/statistics").configure(handlers::statistics::routes))
            .service(web::scope("/v1/kurum-duyuru").configure(handlers::kurum_duyuru::routes))
            // Yeni route'lar buraya eklenecek
    );
}

