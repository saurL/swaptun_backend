use swaptun_services::{
    CreateMusicRequest, CreatePlaylistRequest, MusicService, PlaylistService,
    UpdatePlaylistRequest, UserService,
};
mod test_database;
use test_database::TestDatabase;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_playlist_success() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());

    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };

    let result = playlist_service.create(create_playlist_request, 1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_playlist_success() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());
    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };
    // Suppose qu'une playlist avec ID 1 existe
    playlist_service
        .create(create_playlist_request, 1)
        .await
        .unwrap();
    let result = playlist_service.get_playlist(1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    let playlist = result.unwrap();
    assert_eq!(playlist.id, 1);
    assert_eq!(playlist.name, "My Playlist");
    test_db.drop().await;
}

/*

let test_db = TestDatabase::new().await;
   let playlist_service = PlaylistService::new(test_db.get_db());
   let user_service = UserService::new(test_db.get_db());
   let user = user_service.get_user(1).await.unwrap().unwrap();*/

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_playlist_not_found() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());

    let result = playlist_service.get_playlist(999).await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_user_playlist() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };
    // Suppose qu'une playlist avec ID 1 existe
    playlist_service
        .create(create_playlist_request, 1)
        .await
        .unwrap();
    // Suppose qu'un utilisateur avec ID 1 existe
    let user = user_service.get_user(1).await.unwrap().unwrap();
    let playlists = playlist_service.get_user_playlist(user).await.unwrap();
    println!("Playlists: {:?}", playlists);
    assert!(!playlists.is_empty());
    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_user_playlist_not_empty() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };
    // Suppose qu'une playlist avec ID 1 existe
    playlist_service
        .create(create_playlist_request, 1)
        .await
        .unwrap();
    // Suppose qu'un utilisateur avec ID 1 existe
    let user = user_service.get_user(1).await.unwrap().unwrap();
    let playlists = playlist_service.get_user_playlist(user).await.unwrap();
    println!("Playlists: {:?}", playlists);
    assert!(!playlists.is_empty());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_update_playlist_success() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());
    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };
    // Suppose qu'une playlist avec ID 1 existe
    playlist_service
        .create(create_playlist_request, 1)
        .await
        .unwrap();
    let update_playlist_request = UpdatePlaylistRequest {
        name: Some("Updated Playlist".to_string()),
        description: Some("Updated description".to_string()),
        playlist_id: 1,
    };

    let result = playlist_service.update(update_playlist_request, 1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_playlist_success() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());
    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };
    // Suppose qu'une playlist avec ID 1 existe
    playlist_service
        .create(create_playlist_request, 1)
        .await
        .unwrap();

    let delete_playlist_request = swaptun_services::DeletePlaylistRequest { id: 1 };
    let result = playlist_service.delete(delete_playlist_request, 1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_add_music_to_playlist_success() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());
    let music_service = MusicService::new(test_db.get_db());

    // Crée une playlist
    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };
    playlist_service
        .create(create_playlist_request, 1)
        .await
        .unwrap();
    let playlist = playlist_service.get_playlist(1).await.unwrap();

    // Crée une musique
    let new_music = CreateMusicRequest {
        title: "Test Music".to_string(),
        artist: "Test Artist".to_string(),
        album: "Test Album".to_string(),
        release_date: "2025-01-01".parse().unwrap(),
        genre: Some("Pop".to_string()),
        description: None,
    };
    let music = music_service.create(new_music).await.unwrap();

    // Ajoute la musique à la playlist
    let result = playlist_service
        .add_music(playlist.clone(), music.clone())
        .await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());

    // Vérifie que la musique est dans la playlist
    let musics = playlist_service
        .music_playlist_repository
        .find_musics_by_playlist(playlist)
        .await
        .unwrap();
    assert_eq!(musics.len(), 1);
    assert_eq!(musics[0].title, "Test Music");

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_remove_music_from_playlist_success() {
    let test_db = TestDatabase::new().await;
    let playlist_service = PlaylistService::new(test_db.get_db());
    let music_service = MusicService::new(test_db.get_db());

    // Crée une playlist
    let create_playlist_request = CreatePlaylistRequest {
        name: "My Playlist".to_string(),
        description: Some("A test playlist".to_string()),
    };
    playlist_service
        .create(create_playlist_request, 1)
        .await
        .unwrap();
    let playlist = playlist_service.get_playlist(1).await.unwrap();

    // Crée une musique
    let new_music = CreateMusicRequest {
        title: "Test Music".to_string(),
        artist: "Test Artist".to_string(),
        album: "Test Album".to_string(),
        release_date: "2025-01-01".parse().unwrap(),
        genre: Some("Pop".to_string()),
        description: None,
    };
    let music = music_service.create(new_music).await.unwrap();

    // Ajoute la musique à la playlist
    playlist_service
        .add_music(playlist.clone(), music.clone())
        .await
        .unwrap();

    // Supprime la musique de la playlist
    let result = playlist_service
        .remove_music(playlist.clone(), music.clone())
        .await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());

    // Vérifie que la musique n'est plus dans la playlist
    let musics = playlist_service
        .music_playlist_repository
        .find_musics_by_playlist(playlist)
        .await
        .unwrap();
    assert!(musics.is_empty());

    test_db.drop().await;
}
