use actix_web::{web, HttpResponse, Result};
use log::{error, info};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use swaptun_services::{
    auth::Claims,
    dto::{
        RegisterFcmTokenRequest, RegisterFcmTokenResponse, SendTestNotificationRequest,
        SendTestNotificationResponse,
    },
    error::AppError,
    notification::NotificationService,
    validators::user_validators::process_validation_errors,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/fcm-token", web::post().to(register_fcm_token))
        .route("/test-notification", web::post().to(send_test_notification));
}

/// Endpoint pour enregistrer un token FCM pour l'utilisateur connecté
pub async fn register_fcm_token(
    db: web::Data<DatabaseConnection>,
    claims: web::ReqData<Claims>,
    request: web::Json<RegisterFcmTokenRequest>,
) -> Result<HttpResponse, AppError> {
    info!("Registering FCM token for user {}", claims.user_id);
    let request = request.into_inner();
    // Validation des données d'entrée
    process_validation_errors(&request)?;

    // Récupération de la clé serveur depuis les variables d'environnement
    let server_key = std::env::var("FCM_SERVER_KEY").map_err(|_| AppError::InternalServerError)?;

    // Création du service de notification
    let notification_service =
        NotificationService::new(server_key, Arc::new(db.get_ref().clone()))?;

    // Enregistrement du token FCM
    match notification_service
        .register_fcm_token(
            claims.user_id,
            request.token.clone(),
            request.device_id.clone(),
            request.platform.clone(),
        )
        .await
    {
        Ok(_) => {
            info!(
                "FCM token registered successfully for user {}",
                claims.user_id
            );
            Ok(HttpResponse::Ok().json(RegisterFcmTokenResponse {
                success: true,
                message: "FCM token registered successfully".to_string(),
            }))
        }
        Err(e) => {
            error!(
                "Failed to register FCM token for user {}: {:?}",
                claims.user_id, e
            );
            Err(e)
        }
    }
}

/// Endpoint pour envoyer une notification de test à un utilisateur spécifique
pub async fn send_test_notification(
    db: web::Data<DatabaseConnection>,
    claims: web::ReqData<Claims>,
    request: web::Json<SendTestNotificationRequest>,
) -> Result<HttpResponse, AppError> {
    info!(
        "Sending test notification to user {} from user {}",
        request.user_id, claims.user_id
    );
    let request = request.into_inner();

    // Validation des données d'entrée
    process_validation_errors(&request)?;

    // Récupération de la clé serveur depuis les variables d'environnement
    let server_key = std::env::var("FCM_SERVER_KEY").map_err(|_| AppError::InternalServerError)?;

    // Création du service de notification
    let notification_service =
        NotificationService::new(server_key.clone(), Arc::new(db.get_ref().clone()))?;

    // Envoi de la notification de test
    match notification_service
        .send_notification_to_user(
            &server_key,
            request.user_id,
            request.title.clone(),
            request.body.clone(),
            request.data.clone(),
        )
        .await
    {
        Ok(response) => {
            if response.success {
                info!(
                    "Test notification sent successfully to user {}",
                    request.user_id
                );
                Ok(HttpResponse::Ok().json(SendTestNotificationResponse {
                    success: true,
                    message: "Test notification sent successfully".to_string(),
                    notification_sent: true,
                }))
            } else {
                error!(
                    "Failed to send test notification to user {}: {:?}",
                    request.user_id, response.error
                );
                Ok(HttpResponse::Ok().json(SendTestNotificationResponse {
                    success: false,
                    message: "Failed to send notification".to_string(),
                    notification_sent: false,
                }))
            }
        }
        Err(AppError::NotFound(msg)) => {
            info!("No FCM token found for user {}: {}", request.user_id, msg);
            Ok(HttpResponse::Ok().json(SendTestNotificationResponse {
                success: false,
                message: format!("No FCM token registered for user {}", request.user_id),
                notification_sent: false,
            }))
        }
        Err(e) => {
            error!(
                "Error sending test notification to user {}: {:?}",
                request.user_id, e
            );
            Err(e)
        }
    }
}
