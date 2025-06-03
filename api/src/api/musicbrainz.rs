use actix_web::{HttpResponse, web};
use swaptun_services::error::AppError;
use swaptun_services::musicbrainz::MusicBrainzService;
<<<<<<< HEAD
use std::collections::HashMap;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/track_metadata", web::get().to(get_track_metadata));
}

pub async fn get_track_metadata(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse, AppError> {
    let title = query.get("title").ok_or_else(|| AppError::InternalServerError)?;
    let artist = query.get("artist").ok_or_else(|| AppError::InternalServerError)?;

    let service = MusicBrainzService::new();
    let track_info = service.search_track(title, artist)?;

    Ok(HttpResponse::Ok().json(track_info))
=======


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(test_musicbrainz));
}


pub async fn test_musicbrainz() -> Result<HttpResponse, AppError> {
    let service = MusicBrainzService::new();
    let result = service.search_track("Bohemian Rhapsody", "Queen")?;

    println!("Résultat MusicBrainz : {:?}", result);


    Ok(HttpResponse::Ok().json(result))
>>>>>>> cec52d8 (api musicbrainz install + test)
}
