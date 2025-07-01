use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sea_orm::DbConn;

use swaptun_services::auth::Claims;
use swaptun_services::error::AppError;
use swaptun_services::{ UserService, YoutubeMusicService  };
use log::info;
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/authorization-url", web::get().to(get_authorization_url))
        .service(web::resource("/token").post(set_token))
        .route("/playlists", web::get().to(get_playlists));
}

async fn get_authorization_url(
    db: web::Data<DbConn>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    info!("dans get_authorization_url");
    let user_service = UserService::new(db.get_ref().clone().into());
    let claims = match http_req.extensions().get::<Claims>().cloned(){
        Some(claims) => claims,
        None => return Err(AppError::Unauthorized("Unauthorized".to_string())),
    };
    let user: swaptun_services::UserModel = match user_service.get_user_from_claims(claims).await {
        Ok(user) => user,
        Err(_) => return Err(AppError::Unauthorized("Unauthorized".to_string())),
    };
    let youtube_service = YoutubeMusicService::new(db.get_ref().clone().into());
    let authorization_url = youtube_service.get_authorization_url(&user).await?;
    Ok(HttpResponse::Ok().json(authorization_url))
}

async fn set_token(
    db: web::Data<DbConn>,
    req: web::Json<swaptun_services::AddTokenRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let youtube_service = YoutubeMusicService::new(db.get_ref().clone().into());
    let user_service = UserService::new(db.get_ref().clone().into());
    let claims = http_req.extensions().get::<Claims>().cloned();
    if let Some(claims) = claims {
        let user = user_service.get_user_from_claims(claims).await?;
        youtube_service.auth_callback(&user, req.into_inner()).await?;
        info!("Token added for user");
    } else {
        return Err(AppError::Unauthorized("Unauthorized".to_string()));
    }
    Ok(HttpResponse::Ok().json(true))
}

async fn get_playlists(
    db: web::Data<DbConn>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let youtube_service = YoutubeMusicService::new(db.get_ref().clone().into());
    let user_service = UserService::new(db.get_ref().clone().into());
    let claims = http_req.extensions().get::<Claims>().cloned();
    if let Some(claims) = claims {
        let user = user_service.get_user_from_claims(claims).await?;
        let playlists = youtube_service.get_user_playlists(&user).await?;
        Ok(HttpResponse::Ok().json(playlists))
    } else {
        Err(AppError::Unauthorized("Unauthorized".to_string()))
    }
}