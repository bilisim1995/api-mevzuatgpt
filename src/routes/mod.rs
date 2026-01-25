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
            .service(web::scope("/v2/regulations").configure(handlers::regulation_yargitay::routes))
            .service(web::scope("/v1/statistics").configure(handlers::statistics::routes))
            .service(web::scope("/v1/kurum-duyuru").configure(handlers::kurum_duyuru::routes))
            .service(web::scope("/v1/search").configure(handlers::search::routes))
            .service(web::scope("/v2/search").configure(handlers::search_yargitay::routes_v2))
            .service(web::scope("/v2/documents").configure(handlers::documents_yargitay::routes_v2))
            .service(web::scope("/v1/sitemap").configure(handlers::sitemap::routes))
            // Yeni route'lar buraya eklenecek
    )
    .route("/sitemap.xml", web::get().to(handlers::sitemap::get_sitemap_xml));
}

