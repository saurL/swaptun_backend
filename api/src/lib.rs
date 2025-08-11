#[cfg(feature = "full")]
pub mod api;
#[cfg(feature = "full")]
pub mod config;
#[cfg(feature = "full")]
use crate::config::AppConfig;
#[cfg(feature = "full")]
use actix_web::{middleware::Logger, web, App, HttpServer};
#[cfg(feature = "full")]
use dotenv::dotenv;
#[cfg(feature = "full")]
use sea_orm::{Database, DbConn};
#[cfg(feature = "full")]
use std::io;
#[cfg(feature = "full")]
use swaptun_migrations::{Migrator, MigratorTrait};
pub use swaptun_services::*;
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

    // Test mail service connection
    if cfg!(not(debug_assertions)) {
        log::info!("Running mail service connection test...");
        test_mail_service_connection().await;
        log::info!("Mail service connection test completed successfully");
        log::info!("Running notification service connection test...");
        test_notification_service_connection(db.clone()).await;
        log::info!("Notification service connection test completed successfully");
    } else {
        log::warn!("Mail service connection test skipped in debug mode");
    }

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

/// Test the mail service connection at startup
async fn test_mail_service_connection() {
    let service =
        swaptun_services::mail::MailService::new().expect("Failed to create mail service");
    service
        .test_connection()
        .await
        .expect("Failed to connect to the mail service");
}

async fn test_notification_service_connection(db: DbConn) {
    swaptun_services::notification::NotificationService::new(db.into())
        .await
        .expect("Failed to create notification service");
}
