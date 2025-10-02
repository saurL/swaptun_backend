use actix_web::{web, HttpResponse};
use log::info;
use sea_orm::DbConn;

use swaptun_services::auth::Claims;
use swaptun_services::error::AppError;
use swaptun_services::{AddTokenRequest, AppleMusicService, UserService};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/token").post(set_token))
        .service(web::resource("/developer-token").get(get_developer_token))
        .service(web::resource("/synchronize").post(synchronize_playlist));
}

async fn set_token(
    db: web::Data<DbConn>,
    req: web::Json<AddTokenRequest>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let apple_service = AppleMusicService::new(db.get_ref().clone().into());

    apple_service
        .add_user_token(req.into_inner(), claims.user_id)
        .await?;
    info!("Apple Music token added for user");

    Ok(HttpResponse::Ok().json(true))
}

async fn get_developer_token(
    db: web::Data<DbConn>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let apple_service = AppleMusicService::new(db.get_ref().clone().into());
    let user_service = UserService::new(db.get_ref().clone().into());

    let user = user_service
        .get_user_from_claims(claims.into_inner())
        .await?;

    let token = apple_service.generate_developer_token().await?;
    info!("Retrieved Apple Music token for user {}", user.id);
    Ok(HttpResponse::Ok().json(token))
}
async fn synchronize_playlist(
    db: web::Data<DbConn>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let claims = claims.into_inner();
    let user_service = UserService::new(db.get_ref().clone().into());
    let user = user_service.get_user_from_claims(claims).await?;

    let apple_music_service = AppleMusicService::new(db.get_ref().clone().into());
    let _ = apple_music_service.import_playlists(&user).await?;
    Ok(HttpResponse::Ok().finish())
}
