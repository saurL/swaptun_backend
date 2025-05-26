use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use log::info;
use sea_orm::DbConn;

use swaptun_services::auth::Claims;
use swaptun_services::error::AppError;
use swaptun_services::{AddTokenRequest, GetAuthorizationUrlRequest, SpotifyService, UserService};
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/authorization-url", web::get().to(get_authorization_url))
        .service(web::resource("/token").post(set_token))
        .route("/test", web::get().to(get_user_playlists));
}

async fn get_authorization_url(
    db: web::Data<DbConn>,
    req: web::Json<GetAuthorizationUrlRequest>,
) -> Result<HttpResponse, AppError> {
    let spotify_service = SpotifyService::new(db.get_ref().clone().into());
    let authorization_url = spotify_service.get_authorization_url(req.port).await?;
    Ok(HttpResponse::Ok().json(authorization_url))
}

async fn set_token(
    db: web::Data<DbConn>,
    req: web::Json<AddTokenRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let spotify_service = SpotifyService::new(db.get_ref().clone().into());
    let user_service = UserService::new(db.get_ref().clone().into());
    let claims = http_req.extensions().get::<Claims>().cloned();
    if let Some(claims) = claims {
        let user = user_service.get_user_from_claims(claims).await?;
        spotify_service.add_token(req.into_inner(), user).await?;
        info!("Token added for user");
    } else {
        return Err(AppError::Unauthorized("Unauthorized".to_string()));
    }
    Ok(HttpResponse::Ok().json(true))
}

async fn get_user_playlists(
    db: web::Data<DbConn>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let spotify_service = SpotifyService::new(db.get_ref().clone().into());
    let user_service = UserService::new(db.get_ref().clone().into());
    let claims = http_req.extensions().get::<Claims>().cloned();

    if let Some(claims) = claims {
        let user = user_service.get_user_from_claims(claims).await?;
        let playlists = spotify_service.get_user_playlists(user).await?;
        info!("playlists {:?}", playlists);
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(AppError::Unauthorized("Unauthorized".to_string()))
    }
}
