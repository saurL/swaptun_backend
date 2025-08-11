use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sea_orm::DbConn;

use swaptun_services::auth::Claims;
use swaptun_services::error::AppError;
use swaptun_services::{
    CreateUserRequest, GetUsersParams, ResetPasswordRequest, UpdateUserRequest, UserBean,
    UserService,
};

pub fn configure_protected(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").get(get_users))
        .service(
            web::resource("/me")
                .get(get_current_user)
                .put(update_current_user),
        )
        .service(web::resource("/reset-password").post(reset_password))
        .service(
            web::resource("/{id:\\d+}")
                .get(get_user)
                .put(update_user)
                .delete(delete_user_physical),
        )
        .service(web::resource("/{id:\\d+}/soft-delete").patch(delete_user_logical))
        .service(web::resource("/{id:\\d+}/restore").patch(restore_user));
}

pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").post(create_user));
}

pub async fn get_users(
    db: web::Data<DbConn>,
    query: web::Query<GetUsersParams>,
) -> Result<HttpResponse, AppError> {
    let user_service = UserService::new(db.get_ref().clone().into());

    let users = user_service.get_users(query.into_inner()).await?;

    Ok(HttpResponse::Ok().json(users))
}

pub async fn get_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let user_service = UserService::new(db.get_ref().clone().into());
    let user = user_service.get_user(user_id).await?;

    match user {
        Some(user) => Ok(HttpResponse::Ok().json(user)),
        None => Err(AppError::NotFound(format!(
            "User with ID {} not found",
            user_id
        ))),
    }
}

pub async fn create_user(
    db: web::Data<DbConn>,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user_service = UserService::new(db.get_ref().clone().into());
    user_service.create_user(request.into_inner()).await?;

    Ok(HttpResponse::Created().finish())
}

pub async fn update_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
    request: web::Json<UpdateUserRequest>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();

    if claims.user_id != user_id {
        return Err(AppError::Unauthorized(
            "You are not authorized to update this user".to_string(),
        ));
    }

    let user_service = UserService::new(db.get_ref().clone().into());
    let updated_user = user_service
        .update_user(request.into_inner(), user_id)
        .await?;
    Ok(HttpResponse::Ok().json(updated_user))
}

pub async fn delete_user_physical(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let user_service = UserService::new(db.get_ref().clone().into());
    user_service.delete_user_physical(user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn delete_user_logical(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let user_service = UserService::new(db.get_ref().clone().into());
    user_service.delete_user_logical(user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn restore_user(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let user_service = UserService::new(db.get_ref().clone().into());
    user_service.restore_user(user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
pub async fn get_current_user(
    db: web::Data<DbConn>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get claims from request extensions that were set by auth middleware
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

    let user_service = UserService::new(db.get_ref().clone().into());
    let user = user_service.get_user_from_claims(claims).await?;
    let user_bean: UserBean = user.into();
    Ok(HttpResponse::Ok().json(user_bean))
}

pub async fn update_current_user(
    db: web::Data<DbConn>,
    req: HttpRequest,
    update_data: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    // Get claims from request extensions that were set by auth middleware
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

    let user_service = UserService::new(db.get_ref().clone().into());
    let updated_user = user_service
        .update_user(update_data.into_inner(), claims.user_id)
        .await?;

    Ok(HttpResponse::Ok().json(updated_user))
}

pub async fn reset_password(
    db: web::Data<DbConn>,
    request: web::Json<ResetPasswordRequest>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    // Get claims from request extensions that were set by auth middleware
    if let Some(expiration) = claims.exp {
        if expiration < chrono::Utc::now().timestamp() as usize {
            return Err(AppError::Unauthorized("Token has expired".to_string()));
        }
    } else {
        return Err(AppError::Unauthorized(
            "No authentication token found".to_string(),
        ));
    }
    let req = request.into_inner();
    let user_service = UserService::new(db.get_ref().clone().into());
    let updated_user = user_service
        .reset_password(claims.into_inner(), req.password)
        .await?;

    Ok(HttpResponse::Ok().json(updated_user))
}
