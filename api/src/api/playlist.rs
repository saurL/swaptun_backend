use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use sea_orm::DbConn;

use swaptun_services::auth::Claims;
use swaptun_services::error::AppError;
use swaptun_services::{
    CreateMusicRequest, CreatePlaylistRequest, DeletePlaylistRequest, GetPlaylistsParams,
    PlaylistService, UpdatePlaylistRequest, UserService,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .get(get_user_playlists)
            .post(create_playlist),
    )
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
    );
}

async fn get_user_playlists(
    db: web::Data<DbConn>,
    req: HttpRequest,
    query: web::Json<GetPlaylistsParams>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

    let user_service = UserService::new(db.get_ref().clone().into());
    let user = user_service.get_user_from_claims(claims).await?;

    let playlist_service = PlaylistService::new(db.get_ref().clone().into());
    let playlists = playlist_service
        .get_user_playlist(user, query.into_inner())
        .await?;

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
    req: HttpRequest,
    path: web::Path<i32>,
    request: web::Json<CreateMusicRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("No authentication token found".to_string()))?;

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
