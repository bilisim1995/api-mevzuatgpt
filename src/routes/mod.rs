use actix_web::web;
use crate::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(handlers::health::health_check))
            // Yeni route'lar buraya eklenecek
            // .service(web::scope("/example").configure(handlers::example_handler::routes))
    );
}

