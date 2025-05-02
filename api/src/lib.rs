#[cfg(feature = "full")]
pub mod api;
#[cfg(feature = "full")]
pub mod config;
#[cfg(feature = "full")]
use crate::config::AppConfig;
#[cfg(feature = "full")]
use actix_web::{App, HttpServer, middleware::Logger, web};
#[cfg(feature = "full")]
use dotenv::dotenv;
#[cfg(feature = "full")]
use sea_orm::{Database, DbConn};
#[cfg(feature = "full")]
use std::io;
#[cfg(feature = "full")]
use swaptun_migrations::{Migrator, MigratorTrait};
pub use swaptun_services::dto::*;
#[cfg(feature = "full")]
#[tokio::main]
pub async fn main() -> io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let app_config = AppConfig::from_env();

    log::info!(
        "Starting server at {}:{}",
        app_config.server.host,
        app_config.server.port
    );

    let db: DbConn = Database::connect(&app_config.database.url)
        .await
        .expect("Error connecting to the database");

    log::info!("Running database migrations...");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    log::info!("Database migrations completed successfully");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(|config| api::configure_routes(config, db.clone()))
            .wrap(Logger::default())
    })
    .bind(format!(
        "{}:{}",
        app_config.server.host, app_config.server.port
    ))?
    .run()
    .await
}
