use actix_web::{HttpResponse, web};
use swaptun_services::error::AppError;
use swaptun_services::musicbrainz::MusicBrainzService;
<<<<<<< HEAD
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
=======
use std::collections::HashMap;
>>>>>>> 4c855a5 (implement musicbrainz dans spotify)


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/track_metadata", web::get().to(get_track_metadata));
}

pub async fn get_track_metadata(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse, AppError> {
    let title = query.get("title").ok_or_else(|| AppError::InternalServerError)?;
    let artist = query.get("artist").ok_or_else(|| AppError::InternalServerError)?;

    let service = MusicBrainzService::new();
    let track_info = service.search_track(title, artist)?;

<<<<<<< HEAD
    println!("Résultat MusicBrainz : {:?}", result);


    Ok(HttpResponse::Ok().json(result))
>>>>>>> cec52d8 (api musicbrainz install + test)
=======
    Ok(HttpResponse::Ok().json(track_info))
>>>>>>> 4c855a5 (implement musicbrainz dans spotify)
}
