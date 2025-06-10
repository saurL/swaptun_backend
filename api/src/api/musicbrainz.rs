use actix_web::{HttpResponse, web};
use swaptun_services::error::AppError;
use swaptun_services::musicbrainz::MusicBrainzService;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(test_musicbrainz));
}


pub async fn test_musicbrainz() -> Result<HttpResponse, AppError> {
    let service = MusicBrainzService::new();
    let result = service.search_track("Bohemian Rhapsody", "Queen")?;

    println!("Résultat MusicBrainz : {:?}", result);


    Ok(HttpResponse::Ok().json(result))
}
