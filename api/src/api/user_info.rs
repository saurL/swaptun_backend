use actix_web::{web, HttpResponse};
use sea_orm::DbConn;

use swaptun_services::{
    error::AppError, 
    user_info::model::UserInfoRequest, 
    user_info::service::UserInfoService,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/user_info", web::post().to(create_user_info));
}

async fn create_user_info(
    db: web::Data<DbConn>,
    req: web::Json<UserInfoRequest>,
) -> Result<HttpResponse, AppError> {
    let service = UserInfoService::new(db.get_ref().clone().into());
    service.save_user_info(req.into_inner()).await?;
    Ok(HttpResponse::Created().finish())
}
