use actix_web::{HttpResponse, web};
use sea_orm::DbConn;

use swaptun_services::error::AppError;
use swaptun_services::{GetAuthorizationUrlRequest, SpotifyService};
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/authorization-url", web::get().to(get_authorization_url));
}

async fn get_authorization_url(
    db: web::Data<DbConn>,
    req: web::Json<GetAuthorizationUrlRequest>,
) -> Result<HttpResponse, AppError> {
    let spotify_service = SpotifyService::new(db.get_ref().clone().into());
    let authorization_url = spotify_service.get_authorization_url(req.port).await?;
    Ok(HttpResponse::Ok().json(authorization_url))
}
