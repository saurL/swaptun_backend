use actix_web::{web, HttpResponse};
use sea_orm::DbConn;

use swaptun_services::auth::Claims;
use swaptun_services::error::AppError;
use swaptun_services::{DeezerService, UserService};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/disconnect", web::delete().to(disconnect));
}

async fn disconnect(
    db: web::Data<DbConn>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let deezer_service = DeezerService::new(db.get_ref().clone().into());
    let user_service = UserService::new(db.get_ref().clone().into());

    let user = user_service
        .get_user_from_claims(claims.into_inner())
        .await?;

    deezer_service.disconnect(&user).await?;

    Ok(HttpResponse::NoContent().finish())
}
