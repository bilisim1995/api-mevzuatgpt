use actix_web::{web, HttpResponse};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::sitemap::{
    SitemapInstitution, SitemapDocument,
    SitemapInstitutionsResponse, SitemapDocumentsResponse
};
use futures::stream::TryStreamExt;

const DOMAIN: &str = "https://mevzuatgpt.org";

// Helper function to create slug from institution name
fn create_slug_from_name(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .replace('ç', "c")
        .replace('ğ', "g")
        .replace('ı', "i")
        .replace('ö', "o")
        .replace('ş', "s")
        .replace('ü', "u")
}

// GetSitemapInstitutions returns all institutions for sitemap
pub async fn get_sitemap_institutions(state: web::Data<AppState>) -> HttpResponse {
    let kurum_collection: Collection<MongoDocument> = state.db.collection("kurumlar");

    // Tüm kurumları al
    let kurum_docs = match kurum_collection.find(None, None).await {
        Ok(cursor) => {
            match cursor.try_collect::<Vec<_>>().await {
                Ok(docs) => docs,
                Err(e) => {
                    log::error!("Kurum deserialize hatası: {}", e);
                    return HttpResponse::InternalServerError().json(SitemapInstitutionsResponse {
                        success: false,
                        data: vec![],
                        count: 0,
                        message: "Kurumlar alınamadı".to_string(),
                    });
                }
            }
        }
        Err(e) => {
            log::error!("MongoDB sorgu hatası: {}", e);
            return HttpResponse::InternalServerError().json(SitemapInstitutionsResponse {
                success: false,
                data: vec![],
                count: 0,
                message: "Kurumlar alınamadı".to_string(),
            });
        }
    };

    let mut institutions: Vec<SitemapInstitution> = Vec::new();

    for doc_map in kurum_docs {
        let kurum_adi = doc_map
            .get_str("kurum_adi")
            .or_else(|_| doc_map.get_str("kurumAdi"))
            .unwrap_or("")
            .to_string();

        // Boş veya "Bilinmeyen Kurum" olanları atla
        if kurum_adi.is_empty() || kurum_adi == "Bilinmeyen Kurum" {
            continue;
        }

        // Create slug from institution name
        let slug = create_slug_from_name(&kurum_adi);

        institutions.push(SitemapInstitution {
            kurum_adi,
            slug,
        });
    }

    let count = institutions.len();
    HttpResponse::Ok().json(SitemapInstitutionsResponse {
        success: true,
        data: institutions,
        count,
        message: "Başarılı".to_string(),
    })
}

// GetSitemapAllDocuments returns all documents for sitemap
pub async fn get_sitemap_all_documents(state: web::Data<AppState>) -> HttpResponse {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");

    let filter = doc! {
        "status": "aktif"
    };

    let find_options = mongodb::options::FindOptions::builder()
        .sort(doc! { "olusturulma_tarihi": -1 })
        .projection(doc! {
            "url_slug": 1,
            "olusturulma_tarihi": 1
        })
        .build();

    let cursor = match metadata_collection.find(filter, find_options).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("Tüm belgeler sorgu hatası: {}", e);
            return HttpResponse::InternalServerError().json(SitemapDocumentsResponse {
                success: false,
                data: vec![],
                count: 0,
                message: "Tüm belgeler alınamadı".to_string(),
            });
        }
    };

    let raw_documents: Vec<MongoDocument> = match cursor.try_collect().await {
        Ok(docs) => docs,
        Err(e) => {
            log::error!("Belgeler deserialize hatası: {}", e);
            return HttpResponse::InternalServerError().json(SitemapDocumentsResponse {
                success: false,
                data: vec![],
                count: 0,
                message: "Belgeler decode edilemedi".to_string(),
            });
        }
    };

    // Convert to sitemap format
    let mut documents: Vec<SitemapDocument> = Vec::new();
    for doc_map in raw_documents {
        let id = doc_map
            .get_object_id("_id")
            .map(|oid| oid.to_hex())
            .unwrap_or_default();
        let url_slug = doc_map.get_str("url_slug").unwrap_or("").to_string();
        let olusturulma_tarihi = doc_map
            .get_str("olusturulma_tarihi")
            .unwrap_or("")
            .to_string();

        documents.push(SitemapDocument {
            id,
            url_slug,
            olusturulma_tarihi,
        });
    }

    let count = documents.len();
    HttpResponse::Ok().json(SitemapDocumentsResponse {
        success: true,
        data: documents,
        count,
        message: "Tüm sitemap belgeleri başarıyla alındı".to_string(),
    })
}

// GetSitemapXML returns XML sitemap for all documents
pub async fn get_sitemap_xml(state: web::Data<AppState>) -> HttpResponse {
    let metadata_collection: Collection<MongoDocument> = state.db.collection("metadata");

    // Get all active documents
    let filter = doc! { "status": "aktif" };
    let find_options = mongodb::options::FindOptions::builder()
        .sort(doc! { "belge_yayin_tarihi": -1 })
        .projection(doc! {
            "url_slug": 1,
            "belge_yayin_tarihi": 1,
            "olusturulma_tarihi": 1
        })
        .build();

    let cursor = match metadata_collection.find(filter, find_options).await {
        Ok(cursor) => cursor,
        Err(e) => {
            log::error!("XML sitemap sorgu hatası: {}", e);
            return HttpResponse::InternalServerError()
                .content_type("application/xml")
                .body("<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"></urlset>");
        }
    };

    let documents: Vec<MongoDocument> = match cursor.try_collect().await {
        Ok(docs) => docs,
        Err(e) => {
            log::error!("XML sitemap deserialize hatası: {}", e);
            return HttpResponse::InternalServerError()
                .content_type("application/xml")
                .body("<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"></urlset>");
        }
    };

    // Generate XML sitemap
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Add static pages
    xml.push_str(&format!(
        "  <url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>\n",
        DOMAIN
    ));
    xml.push_str(&format!(
        "  <url><loc>{}/hakkinda</loc><changefreq>weekly</changefreq><priority>0.8</priority></url>\n",
        DOMAIN
    ));
    xml.push_str(&format!(
        "  <url><loc>{}/iletisim</loc><changefreq>weekly</changefreq><priority>0.8</priority></url>\n",
        DOMAIN
    ));

    // Add document pages
    for doc_map in documents {
        if let Ok(url_slug) = doc_map.get_str("url_slug") {
            if !url_slug.is_empty() {
                let lastmod = doc_map
                    .get_str("olusturulma_tarihi")
                    .or_else(|_| doc_map.get_str("belge_yayin_tarihi"))
                    .unwrap_or("");

                // Format date if needed (ensure ISO 8601 format)
                let formatted_date = if lastmod.contains('T') || lastmod.contains('Z') {
                    lastmod.to_string()
                } else if !lastmod.is_empty() {
                    format!("{}T00:00:00Z", lastmod)
                } else {
                    String::new()
                };

                if !formatted_date.is_empty() {
                    xml.push_str(&format!(
                        "  <url><loc>{}/belge/{}</loc><lastmod>{}</lastmod><changefreq>monthly</changefreq><priority>0.9</priority></url>\n",
                        DOMAIN, url_slug, formatted_date
                    ));
                } else {
                    xml.push_str(&format!(
                        "  <url><loc>{}/belge/{}</loc><changefreq>monthly</changefreq><priority>0.9</priority></url>\n",
                        DOMAIN, url_slug
                    ));
                }
            }
        }
    }

    xml.push_str("</urlset>");

    HttpResponse::Ok()
        .content_type("application/xml")
        .body(xml)
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/institutions", web::get().to(get_sitemap_institutions))
        .route("/all-documents", web::get().to(get_sitemap_all_documents));
}

