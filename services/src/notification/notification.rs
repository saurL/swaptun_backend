use std::sync::Arc;

use crate::dto::notification_request::*;
use crate::error::AppError;
use fcm::{Client, MessageBuilder, NotificationBuilder};
use log::error;
use sea_orm::DatabaseConnection;
use swaptun_repositories::FcmTokenRepository;

pub struct NotificationService {
    client: Client,
    fcm_token_repository: FcmTokenRepository,
}

impl NotificationService {
    /// Crée une nouvelle instance du service de notification
    pub fn new(server_key: String, db: Arc<DatabaseConnection>) -> Result<Self, AppError> {
        let client = Client::new();
        let fcm_token_repository = FcmTokenRepository::new(db);
        Ok(Self {
            client,
            fcm_token_repository,
        })
    }

    /// Envoie une notification à un token spécifique
    pub async fn send_notification(
        &self,
        server_key: &str,
        request: NotificationRequest,
    ) -> Result<NotificationResponse, AppError> {
        let mut message_builder = MessageBuilder::new(server_key, &request.token);

        // Construction de la notification
        let mut notification_builder = NotificationBuilder::new();
        notification_builder
            .title(&request.title)
            .body(&request.body);
        if let Some(image) = &request.image {
            notification_builder.icon(image);
        }

        if let Some(sound) = &request.sound {
            notification_builder.sound(sound);
        }

        if let Some(badge) = &request.badge {
            notification_builder.badge(badge);
        }

        if let Some(click_action) = &request.click_action {
            notification_builder.click_action(click_action);
        }

        let notification = notification_builder.finalize();
        message_builder.notification(notification);
        // Ajout des données personnalisées
        if let Some(data) = &request.data {
            message_builder.data(data).map_err(|e| {
                error!("Failed to add data to message: {}", e);
                AppError::InternalServerError
            })?;
        }

        // Définition de la priorité
        if let Some(priority) = &request.priority {
            match priority {
                NotificationPriority::High => {
                    message_builder.priority(fcm::Priority::High);
                }
                NotificationPriority::Normal => {
                    message_builder.priority(fcm::Priority::Normal);
                }
            }
        }

        let message = message_builder.finalize();

        match self.client.send(message).await {
            Ok(response) => {
                let success = response.error.is_none();
                Ok(NotificationResponse {
                    success,
                    message_id: response.message_id,
                    error: response.error,
                    multicast_id: None,
                    success_count: if success { Some(1) } else { Some(0) },
                    failure_count: if success { Some(0) } else { Some(1) },
                    canonical_ids: None,
                    results: None,
                })
            }
            Err(e) => {
                error!("Failed to send notification: {}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    /// Envoie une notification à plusieurs tokens (multicast)
    pub async fn send_multicast_notification(
        &self,
        server_key: &str,
        request: MulticastNotificationRequest,
    ) -> Result<NotificationResponse, AppError> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for token in &request.tokens {
            let single_request = NotificationRequest {
                token: token.clone(),
                title: request.title.clone(),
                body: request.body.clone(),
                data: request.data.clone(),
                image: request.image.clone(),
                sound: request.sound.clone(),
                badge: request.badge.clone(),
                click_action: request.click_action.clone(),
                priority: request.priority.clone(),
            };

            match self.send_notification(server_key, single_request).await {
                Ok(response) => {
                    if response.success {
                        success_count += 1;
                        results.push(NotificationResult {
                            message_id: response.message_id,
                            registration_id: Some(token.clone()),
                            error: None,
                        });
                    } else {
                        failure_count += 1;
                        results.push(NotificationResult {
                            message_id: None,
                            registration_id: Some(token.clone()),
                            error: response.error,
                        });
                    }
                }
                Err(_) => {}
            }
        }

        Ok(NotificationResponse {
            success: failure_count == 0,
            message_id: None,
            error: None,
            multicast_id: Some(chrono::Utc::now().timestamp()),
            success_count: Some(success_count),
            failure_count: Some(failure_count),
            canonical_ids: Some(0),
            results: Some(results),
        })
    }

    /// Envoie une notification à un topic
    pub async fn send_topic_notification(
        &self,
        server_key: &str,
        request: TopicNotificationRequest,
        target: &str, // target can be a topic name like "/topics/my_topic" or a specific token
    ) -> Result<NotificationResponse, AppError> {
        let mut message_builder = MessageBuilder::new(server_key, target);

        // Construction de la notification
        let mut notification_builder = NotificationBuilder::new();
        notification_builder
            .title(&request.title)
            .body(&request.body);
        if let Some(image) = &request.image {
            notification_builder.icon(image);
        }

        if let Some(sound) = &request.sound {
            notification_builder.sound(sound);
        }

        if let Some(badge) = &request.badge {
            notification_builder.badge(badge);
        }

        if let Some(click_action) = &request.click_action {
            notification_builder.click_action(click_action);
        }

        let notification = notification_builder.finalize();
        message_builder.notification(notification);
        // Ajout des données personnalisées
        if let Some(data) = &request.data {
            message_builder
                .data(data)
                .map_err(|_| AppError::InternalServerError)?;
        }

        // Définition de la priorité
        if let Some(priority) = &request.priority {
            match priority {
                NotificationPriority::High => {
                    message_builder.priority(fcm::Priority::High);
                }
                NotificationPriority::Normal => {
                    message_builder.priority(fcm::Priority::Normal);
                }
            }
        }

        let message = message_builder.finalize();

        match self.client.send(message).await {
            Ok(response) => {
                let success = response.error.is_none();
                Ok(NotificationResponse {
                    success,
                    message_id: response.message_id,
                    error: response.error,
                    multicast_id: None,
                    success_count: if success { Some(1) } else { Some(0) },
                    failure_count: if success { Some(0) } else { Some(1) },
                    canonical_ids: None,
                    results: None,
                })
            }
            Err(e) => {
                error!(" Error sending topic notification: {}", e);
                Err(AppError::InternalServerError)
            }
        }
    }

    /// Abonne des tokens à un topic
    pub async fn subscribe_to_topic(
        &self,
        server_key: &str,
        request: SubscribeToTopicRequest,
    ) -> Result<TopicManagementResponse, AppError> {
        // Note: La crate fcm ne supporte pas directement la gestion des topics
        // Il faudrait utiliser l'API REST de Firebase directement
        let client = reqwest::Client::new();
        let url = "https://iid.googleapis.com/iid/v1:batchAdd";

        let body = serde_json::json!({
            "to": format!("/topics/{}", request.topic),
            "registration_tokens": request.tokens
        });

        let response = client
            .post(url)
            .header("Authorization", format!("key={}", server_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Error sending topic subscription request: {}", e);
                AppError::InternalServerError
            })?;

        if response.status().is_success() {
            Ok(TopicManagementResponse {
                success: true,
                error: None,
                results: None,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Ok(TopicManagementResponse {
                success: false,
                error: Some(error_text),
                results: None,
            })
        }
    }

    /// Désabonne des tokens d'un topic
    pub async fn unsubscribe_from_topic(
        &self,
        server_key: &str,
        request: SubscribeToTopicRequest,
    ) -> Result<TopicManagementResponse, AppError> {
        let client = reqwest::Client::new();
        let url = "https://iid.googleapis.com/iid/v1:batchRemove";

        let body = serde_json::json!({
            "to": format!("/topics/{}", request.topic),
            "registration_tokens": request.tokens
        });

        let response = client
            .post(url)
            .header("Authorization", format!("key={}", server_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                error!("Error sending topic unsubscription request: {}", e);
                AppError::InternalServerError
            })?;

        if response.status().is_success() {
            Ok(TopicManagementResponse {
                success: true,
                error: None,
                results: None,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Ok(TopicManagementResponse {
                success: false,
                error: Some(error_text),
                results: None,
            })
        }
    }

    /// Valide un token FCM
    pub async fn validate_token(&self, server_key: &str, token: &str) -> Result<bool, AppError> {
        // Envoie une notification de test pour valider le token
        let test_request = NotificationRequest {
            token: token.to_string(),
            title: "Test".to_string(),
            body: "Token validation".to_string(),
            data: None,
            image: None,
            sound: None,
            badge: None,
            click_action: None,
            priority: Some(NotificationPriority::Normal),
        };

        match self.send_notification(server_key, test_request).await {
            Ok(response) => Ok(response.success),
            Err(_) => Ok(false),
        }
    }

    /// Enregistre ou met à jour un token FCM pour un utilisateur
    pub async fn register_fcm_token(
        &self,
        user_id: i32,
        token: String,
        device_id: Option<String>,
        platform: Option<String>,
    ) -> Result<(), AppError> {
        self.fcm_token_repository
            .upsert_token(user_id, token, device_id, platform)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    /// Récupère le token FCM actif d'un utilisateur
    pub async fn get_user_fcm_token(&self, user_id: i32) -> Result<Option<String>, AppError> {
        match self
            .fcm_token_repository
            .find_active_by_user_id(user_id)
            .await
        {
            Ok(Some(token_model)) => Ok(Some(token_model.token)),
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    /// Désactive un token FCM
    pub async fn deactivate_fcm_token(&self, user_id: i32) -> Result<(), AppError> {
        if let Some(token) = self.fcm_token_repository.find_by_user_id(user_id).await? {
            self.fcm_token_repository.deactivate_token(token.id).await?;
        }
        Ok(())
    }

    /// Envoie une notification à un utilisateur spécifique via son token FCM
    pub async fn send_notification_to_user(
        &self,
        server_key: &str,
        user_id: i32,
        title: String,
        body: String,
        data: Option<std::collections::HashMap<String, String>>,
    ) -> Result<NotificationResponse, AppError> {
        if let Some(token) = self.get_user_fcm_token(user_id).await? {
            let request = NotificationRequest {
                token,
                title,
                body,
                data,
                image: None,
                sound: None,
                badge: None,
                click_action: None,
                priority: Some(NotificationPriority::Normal),
            };
            self.send_notification(server_key, request).await
        } else {
            Err(AppError::NotFound(format!(
                "No FCM token found for user {}",
                user_id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_notification_service_creation() {
        use sea_orm::{Database, DatabaseConnection};
        use std::sync::Arc;

        // Pour les tests, on peut utiliser une base de données en mémoire
        let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();
        let service = NotificationService::new("test_key".to_string(), Arc::new(db));
        assert!(service.is_ok());
    }

    #[test]
    fn test_notification_request_validation() {
        let request = NotificationRequest {
            token: "test_token".to_string(),
            title: "Test Title".to_string(),
            body: "Test Body".to_string(),
            data: None,
            image: None,
            sound: None,
            badge: None,
            click_action: None,
            priority: Some(NotificationPriority::High),
        };

        // Test de validation basique
        assert!(!request.token.is_empty());
        assert!(!request.title.is_empty());
        assert!(!request.body.is_empty());
    }
}
