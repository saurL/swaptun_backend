use std::sync::Arc;

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use log::error;
use sea_orm::{DatabaseConnection, DbConn};

use swaptun_services::auth::Claims;
use swaptun_services::error::AppError;
use swaptun_services::{
    AppleMusicService, CreateMusicRequest, CreatePlaylistRequest, DeezerService,
    DeletePlaylistRequest, GetPlaylistsParams, NotificationService, PlaylistOrigin,
    PlaylistService, SendPlaylistRequest, SharePlaylistRequest, SpotifyService,
    UpdatePlaylistRequest, UserService, YoutubeMusicService,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .get(get_user_playlists)
            .post(create_playlist),
    )
    .service(web::resource("/shared").get(get_shared_playlists))
    .service(
        web::resource("/{id}")
            .get(get_playlist)
            .put(update_playlist)
            .delete(delete_playlist),
    )
    .service(
        web::resource("/{id}/music")
            .post(add_music_to_playlist)
            .delete(remove_music_from_playlist),
    )
    .service(web::resource("/{id}/send").post(send_playlist_to_origin))
    .service(web::resource("/{id}/share").post(share_playlist));
}

async fn get_user_playlists(
    db: web::Data<DbConn>,
    query: web::Json<GetPlaylistsParams>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let claims = claims.into_inner();

    let user_service = UserService::new(db.get_ref().clone().into());
    let user = user_service.get_user_from_claims(claims).await?;

    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let playlists = playlist_service
        .get_user_playlist(user, query.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(playlists))
}

async fn get_shared_playlists(
    db: web::Data<DbConn>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, AppError> {
    let claims = claims.into_inner();

    let user_service = UserService::new(db.get_ref().clone().into());
    let user = user_service.get_user_from_claims(claims).await?;

    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let playlists = playlist_service.get_shared_playlists_with_details(&user).await?;

    Ok(HttpResponse::Ok().json(playlists))
}

async fn create_playlist(
    db: web::Data<DbConn>,
    req: HttpRequest,
    request: web::Json<CreatePlaylistRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let playlist = playlist_service
        .create(request.into_inner(), claims.user_id)
        .await?;

    Ok(HttpResponse::Created().json(playlist))
}

async fn get_playlist(
    db: web::Data<DbConn>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let playlist_id = path.into_inner();
    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let playlist = playlist_service.get_playlist(playlist_id).await?;

    Ok(HttpResponse::Ok().json(playlist))
}

async fn update_playlist(
    db: web::Data<DbConn>,
    req: HttpRequest,
    request: web::Json<UpdatePlaylistRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    playlist_service
        .update(request.into_inner(), claims.user_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

async fn delete_playlist(
    db: web::Data<DbConn>,
    req: HttpRequest,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

    let playlist_id = path.into_inner();
    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let request = DeletePlaylistRequest { id: playlist_id };
    playlist_service.delete(request, claims.user_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn add_music_to_playlist(
    db: web::Data<DbConn>,
    claims: web::ReqData<Claims>,
    path: web::Path<i32>,
    request: web::Json<CreateMusicRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = claims.into_inner();

    let playlist_id = path.into_inner();
    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let playlist = playlist_service.get_playlist(playlist_id).await?;

    if playlist.user_id != claims.user_id {
        return Err(AppError::Unauthorized(
            "You do not have permission to modify this playlist".to_string(),
        ));
    }

    let music_service = swaptun_services::MusicService::new(db.get_ref().clone().into());
    let music = music_service.create(request.into_inner()).await?;
    playlist_service.add_music(&playlist, music).await?;

    Ok(HttpResponse::Ok().finish())
}

async fn remove_music_from_playlist(
    db: web::Data<DbConn>,
    req: HttpRequest,
    path: web::Path<(i32, String, String, String)>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

    let (playlist_id, title, artist, album) = path.into_inner();
    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let playlist = playlist_service.get_playlist(playlist_id).await?;

    if playlist.user_id != claims.user_id {
        return Err(AppError::Unauthorized(
            "You do not have permission to modify this playlist".to_string(),
        ));
    }

    let music_service = swaptun_services::MusicService::new(db.get_ref().clone().into());
    let music = music_service
        .find_by_id(title, artist, album)
        .await?
        .ok_or_else(|| AppError::NotFound("Music not found".to_string()))?;

    playlist_service.remove_music(&playlist, &music).await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn send_playlist_to_origin(
    db: web::Data<DbConn>,
    req: web::Json<SendPlaylistRequest>,
    claims: web::ReqData<Claims>,
    path: web::Path<i32>,
) -> Result<String, AppError> {
    let db: Arc<DatabaseConnection> = db.get_ref().clone().into();
    let user_service = UserService::new(db.clone());
    let user = user_service
        .get_user_from_claims(claims.into_inner())
        .await?;
    let req = req.into_inner();
    let playlist_id = path.into_inner();
    let destination = req.destination;
    // Send playlist based on its destination
    match destination {
        PlaylistOrigin::Spotify => {
            let spotify_service = SpotifyService::new(db.clone());

            spotify_service
                .create_spotify_playlist_from_db(playlist_id, &user)
                .await
        }
        PlaylistOrigin::YoutubeMusic => {
            let youtube_service = YoutubeMusicService::new(db.clone());

            youtube_service
                .import_playlist_in_yt(&user, playlist_id)
                .await
                .map(|_| "Playlist sent to YouTube Music successfully".to_string())
                .map_err(|e| {
                    error!("Error sending playlist to YouTube Music: {:?}", e);
                    e
                })
        }
        PlaylistOrigin::Deezer => Err(AppError::InternalServerError),

        PlaylistOrigin::AppleMusic => {
            let apple_service = AppleMusicService::new(db.clone());

            apple_service
                .export_playlist_to_apple(playlist_id, &user)
                .await
                .map(|playlist_id| {
                    format!(
                        "Playlist sent to Apple Music successfully with ID: {}",
                        playlist_id
                    )
                })
        }
    }
}
async fn share_playlist(
    db: web::Data<DbConn>,
    req: web::Json<SharePlaylistRequest>,
    claims: web::ReqData<Claims>,
    path: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let db: Arc<DatabaseConnection> = db.get_ref().clone().into();
    let user_service = UserService::new(db.clone());

    // Get the user who is sharing (current authenticated user)
    let current_user = user_service
        .get_user_from_claims(claims.into_inner())
        .await?;

    // Get the user to share with
    let shared_with_user = user_service.find_by_id(req.user_id).await?;
    let playlist_id: i32 = path.into_inner();
    let playlist_service = PlaylistService::new(db.clone());
    let playlist = playlist_service.find_by_id(playlist_id).await?;

    if let (Some(playlist), Some(shared_with_user)) = (playlist, shared_with_user) {
        playlist_service
            .share_playlist(&shared_with_user, &playlist, &current_user)
            .await?;

        // Envoyer une notification à l'utilisateur
        match NotificationService::new(db.clone()).await {
            Ok(notification_service) => {
                let shared_by_name = format!("{} {}", current_user.first_name, current_user.last_name);
                let title = "New Shared Playlist".to_string();
                let body = format!(
                    "{} shared the playlist '{}' with you",
                    shared_by_name, playlist.name
                );

                let mut data = std::collections::HashMap::new();
                data.insert("type".to_string(), "playlist_shared".to_string());
                data.insert("playlist_id".to_string(), playlist.id.to_string());
                data.insert("playlist_name".to_string(), playlist.name.clone());
                data.insert("shared_by_id".to_string(), current_user.id.to_string());
                data.insert("shared_by_username".to_string(), current_user.username.clone());
                data.insert("shared_by_name".to_string(), shared_by_name.clone());
                data.insert("route".to_string(), "/home/shared".to_string());

                match notification_service
                    .send_notification_to_user(shared_with_user.id, title, body, Some(data))
                    .await
                {
                    Ok(_) => {
                        log::info!(
                            "Notification sent to user {} for playlist {} shared by {}",
                            shared_with_user.id,
                            playlist.id,
                            current_user.id
                        );
                    }
                    Err(e) => {
                        error!("Failed to send notification to user {}: {:?}", shared_with_user.id, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to initialize notification service: {:?}", e);
            }
        }
    } else {
        error!("Either playlist or user not found");
    }

    Ok(HttpResponse::NoContent().finish())
}
