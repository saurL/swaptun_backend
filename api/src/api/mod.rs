use actix_web::web::ServiceConfig;
use actix_web::{web, HttpResponse};
use sea_orm::DbConn;
use swaptun_services::auth::jwt::{JwtMiddleware, RoleGuard};
mod apple;
mod auth;
mod musicbrainz;
mod notification;
mod playlist;
mod spotify;
mod user_info;
mod users;
mod youtube;

pub fn configure_routes(cfg: &mut ServiceConfig, db: DbConn) {
    let db_data = web::Data::new(db);

    cfg.app_data(db_data.clone())
        .service(web::scope("/test").configure(musicbrainz::configure))
        .route("/health", web::get().to(health_check))
        .service(
            web::scope("/api")
                .service(web::scope("/auth").configure(|c| auth::configure(c)))
                .service(web::scope("/register").configure(|c| users::configure_public(c)))
                .service(
                    web::scope("")
                        .wrap(JwtMiddleware)
                        .service(
                            web::scope("/users")
                                .wrap(RoleGuard::user())
                                .configure(|c| users::configure_protected(c)),
                        )
                        .service(web::scope("/spotify").configure(|c| spotify::configure(c)))
                        .service(web::scope("/apple").configure(|c| apple::configure(c)))
                        .service(web::scope("/playlists").configure(|c| playlist::configure(c)))
                        .service(web::scope("/youtube").configure(|c| youtube::configure(c)))
                        .service(web::scope("/musicbrainz").configure(musicbrainz::configure))
                        .service(web::scope("/user_info").configure(|c| user_info::configure(c)))
                        .service(
                            web::scope("/notifications").configure(|c| notification::configure(c)),
                        ),
                ),
        );
}

async fn health_check() -> HttpResponse {
    // Test mail service connection
    let mail_status = match swaptun_services::mail::MailService::new() {
        Ok(_) => "UP",
        Err(_) => "DOWN",
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "UP",
        "message": "Service is running",
        "mail_service": mail_status
    }))
}
