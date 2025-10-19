use std::sync::Arc;

use crate::error::AppError;
use crate::notification::dto::notification_request::*;
use google_fcm1::api::{AndroidConfig, Message, Notification, SendMessageRequest};
use google_fcm1::yup_oauth2::hyper_rustls::HttpsConnector;

use google_fcm1::yup_oauth2::{
    read_service_account_key, ServiceAccountAuthenticator, ServiceAccountKey,
};
use google_fcm1::{hyper_util, FirebaseCloudMessaging};
use log::{error, info};
use sea_orm::DatabaseConnection;
use swaptun_repositories::FcmTokenRepository;
use ytmapi_rs::json::Json;
pub struct NotificationService {
    fcm_hub:
        FirebaseCloudMessaging<HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
    fcm_token_repository: FcmTokenRepository,
    project_id: String,
}

impl NotificationService {
    /// Crée une nouvelle instance du service de notification
    pub async fn new(db: Arc<DatabaseConnection>) -> Result<Self, AppError> {
        // Récupération des variables d'environnement

        let secret: ServiceAccountKey = match read_service_account_key("swaptun.json").await {
            Ok(app_secret) => app_secret,
            Err(e) => {
                error!("failed getting firebase json file {}", e);
                return Err(AppError::InternalServerError);
            }
        };
        let project_id = secret.project_id.clone().unwrap();
        // Instantiate the authenticator. It will choose a suitable authentication flow for you,
        // unless you replace  `None` with the desired Flow.
        // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
        // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
        // retrieve them from storage.
        let auth = ServiceAccountAuthenticator::builder(secret)
            .build()
            .await
            .unwrap();

        let client = google_fcm1::hyper_util::client::legacy::Client::builder(
            google_fcm1::hyper_util::rt::TokioExecutor::new(),
        )
        .build(
            google_fcm1::hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .map_err(|e| {
                    error!("Failed to create HTTPS connector: {}", e);
                    AppError::InternalServerError
                })?
                .https_or_http()
                .enable_http1()
                .build(),
        );

        let fcm_hub = FirebaseCloudMessaging::new(client, auth);
        let fcm_token_repository = FcmTokenRepository::new(db);

        Ok(Self {
            fcm_hub,
            fcm_token_repository,
            project_id,
        })
    }

    /// Envoie une notification à un token spécifique
    pub async fn send_notification(
        &self,
        request: NotificationRequest,
    ) -> Result<NotificationResponse, AppError> {
        // Construction de la notification
        let mut notification = Notification::default();
        notification.title = Some(request.title.clone());
        notification.body = Some(request.body.clone());

        if let Some(image) = &request.image {
            notification.image = Some(image.clone());
        }

        // Construction du message
        let mut message = Message::default();
        message.token = Some(request.token.clone());
        message.notification = Some(notification);

        // Ajout des données personnalisées
        if let Some(data) = &request.data {
            // Convertir serde_json::Value en HashMap<String, String>
            // FCM attend toutes les valeurs en tant que strings
            info!("Adding data to notification: {:?}", data);
            let data_map: std::collections::HashMap<String, String> = data
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| {
                            let value_str = match v {
                                serde_json::Value::String(s) => s.clone(),
                                _ => v.to_string(),
                            };
                            (k.clone(), value_str)
                        })
                        .collect()
                })
                .unwrap_or_default();

            message.data = Some(data_map);
        }

        // Configuration Android si nécessaire
        let mut android_config = AndroidConfig::default();
        if let Some(priority) = &request.priority {
            match priority {
                NotificationPriority::High => {
                    android_config.priority = Some("high".to_string());
                }
                NotificationPriority::Normal => {
                    android_config.priority = Some("normal".to_string());
                }
            }
            message.android = Some(android_config);
        }

        info!("Sending notification to token: {}", request.token);

        // Envoi du message
        let send_request = SendMessageRequest {
            message: Some(message),
            validate_only: Some(false),
        };

        match self
            .fcm_hub
            .projects()
            .messages_send(send_request, &format!("projects/{}", self.project_id))
            .doit()
            .await
        {
            Ok((_, response)) => {
                info!("Notification sent successfully: {:?}", response.name);
                Ok(NotificationResponse {
                    success: true,
                    message_id: response.name.and_then(|name| {
                        // Extraire l'ID du message du nom complet
                        name.split('/').last().and_then(|id| id.parse().ok())
                    }),
                    error: None,
                    multicast_id: None,
                    success_count: Some(1),
                    failure_count: Some(0),
                    canonical_ids: None,
                    results: None,
                })
            }
            Err(e) => {
                error!("Failed to send notification: {}", e);
                Ok(NotificationResponse {
                    success: false,
                    message_id: None,
                    error: Some(format!("FCM Error: {}", e)),
                    multicast_id: None,
                    success_count: Some(0),
                    failure_count: Some(1),
                    canonical_ids: None,
                    results: None,
                })
            }
        }
    }

    /// Envoie une notification à plusieurs tokens (multicast)
    pub async fn send_multicast_notification(
        &self,
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

            match self.send_notification(single_request).await {
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
                Err(e) => {
                    failure_count += 1;
                    results.push(NotificationResult {
                        message_id: None,
                        registration_id: Some(token.clone()),
                        error: Some(format!("Error: {}", e)),
                    });
                }
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
        request: TopicNotificationRequest,
        target: &str,
    ) -> Result<NotificationResponse, AppError> {
        // Construction de la notification
        let mut notification = Notification::default();
        notification.title = Some(request.title.clone());
        notification.body = Some(request.body.clone());

        if let Some(image) = &request.image {
            notification.image = Some(image.clone());
        }

        // Construction du message
        let mut message = Message::default();
        message.topic = Some(target.to_string());
        message.notification = Some(notification);

        // Ajout des données personnalisées
        if let Some(data) = &request.data {
            // Convertir serde_json::Value en HashMap<String, String>
            // FCM attend toutes les valeurs en tant que strings
            let data_map: std::collections::HashMap<String, String> = data
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| {
                            let value_str = match v {
                                serde_json::Value::String(s) => s.clone(),
                                _ => v.to_string(),
                            };
                            (k.clone(), value_str)
                        })
                        .collect()
                })
                .unwrap_or_default();

            message.data = Some(data_map);
        }

        // Configuration Android si nécessaire
        let mut android_config = AndroidConfig::default();
        if let Some(priority) = &request.priority {
            match priority {
                NotificationPriority::High => {
                    android_config.priority = Some("high".to_string());
                }
                NotificationPriority::Normal => {
                    android_config.priority = Some("normal".to_string());
                }
            }
            message.android = Some(android_config);
        }

        // Envoi du message
        let send_request = SendMessageRequest {
            message: Some(message),
            validate_only: Some(false),
        };

        match self
            .fcm_hub
            .projects()
            .messages_send(send_request, &format!("projects/{}", self.project_id))
            .doit()
            .await
        {
            Ok((_, response)) => {
                info!("Topic notification sent successfully: {:?}", response.name);
                Ok(NotificationResponse {
                    success: true,
                    message_id: response
                        .name
                        .and_then(|name| name.split('/').last().and_then(|id| id.parse().ok())),
                    error: None,
                    multicast_id: None,
                    success_count: Some(1),
                    failure_count: Some(0),
                    canonical_ids: None,
                    results: None,
                })
            }
            Err(e) => {
                error!("Failed to send topic notification: {}", e);
                Ok(NotificationResponse {
                    success: false,
                    message_id: None,
                    error: Some(format!("FCM Error: {}", e)),
                    multicast_id: None,
                    success_count: Some(0),
                    failure_count: Some(1),
                    canonical_ids: None,
                    results: None,
                })
            }
        }
    }
    /*
    /// Abonne des tokens à un topic
    pub async fn subscribe_to_topic(
        &self,
        request: SubscribeToTopicRequest,
    ) -> Result<TopicManagementResponse, AppError> {
        // Utilisation de l'API REST pour la gestion des topics
        let client = reqwest::Client::new();
        let url = "https://iid.googleapis.com/iid/v1:batchAdd";

        // Récupération du token d'accès
        let token = self
            .fcm_hub
            .auth
            .get_token(&["https://www.googleapis.com/auth/firebase.messaging"])
            .await
            .map_err(|e| {
                error!("Failed to get access token: {}", e);
                AppError::InternalServerError
            })?;

        let body = serde_json::json!({
            "to": format!("/topics/{}", request.topic),
            "registration_tokens": request.tokens
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token.as_str()))
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
        request: SubscribeToTopicRequest,
    ) -> Result<TopicManagementResponse, AppError> {
        let client = reqwest::Client::new();
        let url = "https://iid.googleapis.com/iid/v1:batchRemove";

        // Récupération du token d'accès
        let token = self
            .fcm_hub
            .auth()
            .token(&["https://www.googleapis.com/auth/firebase.messaging"])
            .await
            .map_err(|e| {
                error!("Failed to get access token: {}", e);
                AppError::InternalServerError
            })?;

        let body = serde_json::json!({
            "to": format!("/topics/{}", request.topic),
            "registration_tokens": request.tokens
        });

        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", token.as_str()))
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
    */
    /// Valide un token FCM
    pub async fn validate_token(&self, token: &str) -> Result<bool, AppError> {
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

        match self.send_notification(test_request).await {
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
    pub async fn get_user_fcm_token(&self, user_id: i32) -> Result<Vec<String>, AppError> {
        match self
            .fcm_token_repository
            .find_active_by_user_id(user_id)
            .await
        {
            Ok(tokens) => Ok(tokens.into_iter().map(|t| t.token).collect()),
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
        user_id: i32,
        title: String,
        body: String,
        data: Option<serde_json::Value>,
    ) -> Result<Vec<NotificationResponse>, AppError> {
        let tokens = self.get_user_fcm_token(user_id).await?;
        let mut responses = Vec::new();
        for token in tokens {
            let request = NotificationRequest {
                token,
                title: title.clone(),
                body: body.clone(),
                data: data.clone(),
                image: None,
                sound: None,
                badge: None,
                click_action: None,
                priority: Some(NotificationPriority::Normal),
            };
            let response = self.send_notification(request).await?;
            responses.push(response);
        }
        Ok(responses)
    }

    /// Envoie une notification silencieuse (data-only) à un utilisateur
    /// Cette notification ne s'affiche pas à l'écran mais permet de transmettre des données
    /// au frontend pour mettre à jour l'UI en arrière-plan
    pub async fn send_silent_data_to_user(
        &self,
        user_id: i32,
        data: serde_json::Value,
    ) -> Result<Vec<NotificationResponse>, AppError> {
        let tokens = self.get_user_fcm_token(user_id).await?;
        let mut responses = Vec::new();

        for token in tokens {
            // Construction du message SANS notification (silencieux)
            let mut message = Message::default();
            message.token = Some(token.clone());

            // Convertir serde_json::Value en HashMap<String, String>
            let data_map: std::collections::HashMap<String, String> = data
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| {
                            let value_str = match v {
                                serde_json::Value::String(s) => s.clone(),
                                _ => v.to_string(),
                            };
                            (k.clone(), value_str)
                        })
                        .collect()
                })
                .unwrap_or_default();

            message.data = Some(data_map);

            // Configuration Android pour notification silencieuse
            let mut android_config = AndroidConfig::default();
            android_config.priority = Some("high".to_string()); // High priority pour livraison immédiate
            message.android = Some(android_config);

            info!("Sending silent data notification to user {}", user_id);

            // Envoi du message
            let send_request = SendMessageRequest {
                message: Some(message),
                validate_only: Some(false),
            };

            match self
                .fcm_hub
                .projects()
                .messages_send(send_request, &format!("projects/{}", self.project_id))
                .doit()
                .await
            {
                Ok((_, response)) => {
                    info!("Silent notification sent successfully");
                    responses.push(NotificationResponse {
                        success: true,
                        message_id: response.name.and_then(|name| {
                            name.split('/').last().and_then(|id| id.parse().ok())
                        }),
                        error: None,
                        multicast_id: None,
                        success_count: Some(1),
                        failure_count: Some(0),
                        canonical_ids: None,
                        results: None,
                    });
                }
                Err(e) => {
                    error!("Failed to send silent notification: {}", e);
                    responses.push(NotificationResponse {
                        success: false,
                        message_id: None,
                        error: Some(format!("FCM Error: {}", e)),
                        multicast_id: None,
                        success_count: Some(0),
                        failure_count: Some(1),
                        canonical_ids: None,
                        results: None,
                    });
                }
            }
        }

        Ok(responses)
    }
}
