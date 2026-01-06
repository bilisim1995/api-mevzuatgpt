use actix_web::{web, HttpResponse};
use mongodb::Collection;
use futures::stream::TryStreamExt;
use crate::config::AppState;
use crate::models::kurum::{Kurum, KurumResponse};
use crate::models::ApiResponse;

pub async fn get_kurumlar(state: web::Data<AppState>) -> HttpResponse {
    let collection: Collection<Kurum> = state.db.collection("kurumlar");

    match collection.find(None, None).await {
        Ok(cursor) => {
            let mut kurumlar: Vec<KurumResponse> = Vec::new();

            let results: Result<Vec<_>, _> = cursor.try_collect().await;
            
            match results {
                Ok(docs) => {
                    for kurum in docs {
                        kurumlar.push(KurumResponse::from(kurum));
                    }
                    HttpResponse::Ok().json(ApiResponse::success(kurumlar))
                }
                Err(e) => {
                    log::error!("Kurum deserialize hatası: {}", e);
                    HttpResponse::InternalServerError().json(
                        ApiResponse::<Vec<KurumResponse>>::error(
                            "Kurumlar listesi alınamadı".to_string()
                        )
                    )
                }
            }
        }
        Err(e) => {
            log::error!("MongoDB sorgu hatası: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<KurumResponse>>::error(
                    "Kurumlar listesi alınamadı".to_string()
                )
            )
        }
    }
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_kurumlar));
}

