use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, web};
use sea_orm::DbConn;
use swaptun_services::auth::jwt::{JwtMiddleware, RoleGuard};

mod auth;
mod playlist;
mod spotify;
mod users;
mod musicbrainz;
<<<<<<< HEAD
mod user_info;
=======
>>>>>>> cec52d8 (api musicbrainz install + test)

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
                        .service(web::scope("/playlists").configure(|c| playlist::configure(c)))
                        .service(web::scope("/musicbrainz").configure(musicbrainz::configure))
                        .service(web::scope("/user_info").configure(|c| user_info::configure(c))),
                ),
        );
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "UP",
        "message": "Service is running"
    }))
}
