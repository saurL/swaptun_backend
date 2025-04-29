use actix_web::{HttpResponse, web};
use sea_orm::DbConn;

use swaptun_services::error::AppError;
use swaptun_services::validators::user_validators::process_json_validation;
use swaptun_services::{LoginEmailRequest, LoginRequest, UserService, VerifyTokenRequest};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/login_email", web::post().to(login_email))
        .route("/verify_token", web::post().to(verify_token));
}

async fn login(
    db: web::Data<DbConn>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    process_json_validation(&req)?;

    let user_service = UserService::new(db.get_ref().clone().into());
    let login_response = user_service.login(req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(login_response))
}

async fn login_email(
    db: web::Data<DbConn>,
    req: web::Json<LoginEmailRequest>,
) -> Result<HttpResponse, AppError> {
    process_json_validation(&req)?;

    let user_service = UserService::new(db.get_ref().clone().into());
    let login_response = user_service.login_with_email(req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(login_response))
}

async fn verify_token(
    db: web::Data<DbConn>,
    req: web::Json<VerifyTokenRequest>,
) -> Result<HttpResponse, AppError> {
    let user_service = UserService::new(db.get_ref().clone().into());
    let verify_token_response = user_service.verify_token(req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(verify_token_response))
}
