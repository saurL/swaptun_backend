use swaptun_services::TestDatabase;

use swaptun_models::PlaylistOrigin;
use swaptun_services::CreateUserRequest;
use swaptun_services::{CreatePlaylistRequest, GetPlaylistsParams, PlaylistService, UserService};

#[tokio::test]
async fn test_get_playlists_by_origin() {
    // Setup test database and services
    let test_db = TestDatabase::new().await;
    let db = test_db.get_db();

    let playlist_service = PlaylistService::new(db.clone().into());
    let user = test_db.get_user();
    // Create test playlists
    create_test_playlists(&playlist_service, user.id).await;
    let get_playlists_params = GetPlaylistsParams {
        origin: None, // No specific origin, will fetch all
        include_musics: false,
    };
    // Test getting all playlists
    let all_playlists = playlist_service
        .get_user_playlist(user.clone(), get_playlists_params)
        .await
        .unwrap();
    assert_eq!(all_playlists.vec.len(), 3); // Should have 3 playlists total

    // Test getting Spotify playlists
    let spotify_params = GetPlaylistsParams {
        origin: Some(PlaylistOrigin::Spotify),
        include_musics: false,
    };
    let spotify_playlists = playlist_service
        .get_user_playlist(user.clone(), spotify_params)
        .await
        .unwrap();
    assert_eq!(spotify_playlists.vec.len(), 2); // Should have 2 Spotify playlists
    assert!(spotify_playlists
        .vec
        .iter()
        .all(|p| p.origin == PlaylistOrigin::Spotify));

    // Test getting Deezer playlists
    let deezer_params = GetPlaylistsParams {
        origin: Some(PlaylistOrigin::Deezer),
        include_musics: false,
    };
    let deezer_playlists = playlist_service
        .get_user_playlist(user.clone(), deezer_params)
        .await
        .unwrap();
    assert_eq!(deezer_playlists.vec.len(), 1); // Should have 1 Deezer playlist
    assert!(deezer_playlists
        .vec
        .iter()
        .all(|p| p.origin == PlaylistOrigin::Deezer));
}

#[tokio::test]
async fn test_get_shared_playlists() {
    // Setup test database and services
    let test_db = TestDatabase::new().await;
    let db = test_db.get_db();
    let playlist_service = PlaylistService::new(db.clone().into());

    // Create two users
    let user1 = test_db.get_user();
    let user_service = UserService::new(db.clone().into());
    let create_user_request = CreateUserRequest {
        username: "test_user2".to_string(),
        password: "hashed_passwor12D!".to_string(),
        first_name: "first_name2".to_string(),
        last_name: "last_name2".to_string(),
        email: "test_user2@gmail.com".to_string(),
    };
    let user2 = user_service.create_user(create_user_request).await.unwrap();

    // Create a playlist for user1
    let playlist_request = CreatePlaylistRequest {
        name: "Shared Playlist".to_string(),
        description: Some("A playlist to share".to_string()),
        origin: PlaylistOrigin::Spotify,
        origin_id: "shared123".into(),
    };

    let playlist = playlist_service
        .create(playlist_request, user1.id)
        .await
        .unwrap();

    // Share the playlist with user2
    playlist_service
        .share_playlist(&user2, &playlist, &user1)
        .await
        .unwrap();

    // Verify user2 can get the shared playlist
    let shared_playlists = playlist_service
        .get_shared_playlists(user2.clone())
        .await
        .unwrap();

    assert_eq!(shared_playlists.vec.len(), 1);
    assert_eq!(shared_playlists.vec[0].id, playlist.id);
    assert_eq!(shared_playlists.vec[0].name, "Shared Playlist");

    // Verify user1's own playlists are separate
    let user1_playlists = playlist_service
        .get_user_playlist(user1.clone(), GetPlaylistsParams { origin: None, include_musics: false })
        .await
        .unwrap();

    // User1 should have the playlist in their own playlists
    assert_eq!(user1_playlists.vec.len(), 1);
    assert_eq!(user1_playlists.vec[0].id, playlist.id);

    // User1 should have 0 shared playlists
    let user1_shared_playlists = playlist_service
        .get_shared_playlists(user1.clone())
        .await
        .unwrap();

    assert_eq!(user1_shared_playlists.vec.len(), 0);
}

#[tokio::test]
async fn test_user_and_shared_playlists_separation() {
    // Setup test database and services
    let test_db = TestDatabase::new().await;
    let db = test_db.get_db();
    let playlist_service = PlaylistService::new(db.clone().into());

    // Create two users
    let user1 = test_db.get_user();
    let user_service = UserService::new(db.clone().into());
    let create_user_request = CreateUserRequest {
        username: "test_user3".to_string(),
        password: "hashed_passwor12D!".to_string(),
        first_name: "first_name3".to_string(),
        last_name: "last_name3".to_string(),
        email: "test_user3@gmail.com".to_string(),
    };
    let user2 = user_service.create_user(create_user_request).await.unwrap();

    // Create a playlist for user1 (user-owned)
    let user_playlist_request = CreatePlaylistRequest {
        name: "User Playlist".to_string(),
        description: Some("User's own playlist".to_string()),
        origin: PlaylistOrigin::Spotify,
        origin_id: "user123".into(),
    };

    let _user_playlist = playlist_service
        .create(user_playlist_request, user1.id)
        .await
        .unwrap();

    // Create a playlist for user2 and share it with user1
    let shared_playlist_request = CreatePlaylistRequest {
        name: "Shared Playlist".to_string(),
        description: Some("A shared playlist".to_string()),
        origin: PlaylistOrigin::Deezer,
        origin_id: "shared456".into(),
    };

    let shared_playlist = playlist_service
        .create(shared_playlist_request, user2.id)
        .await
        .unwrap();

    // Share the playlist with user1
    playlist_service
        .share_playlist(&user1, &shared_playlist, &user2)
        .await
        .unwrap();

    // Verify user1 gets their own playlists and shared playlists separately
    let user_playlists = playlist_service
        .get_user_playlist(user1.clone(), GetPlaylistsParams { origin: None, include_musics: false })
        .await
        .unwrap();

    let shared_playlists = playlist_service
        .get_shared_playlists(user1.clone())
        .await
        .unwrap();

    // User1 should have 1 user-owned playlist
    assert_eq!(user_playlists.vec.len(), 1);
    assert_eq!(user_playlists.vec[0].name, "User Playlist");

    // User1 should have 1 shared playlist
    assert_eq!(shared_playlists.vec.len(), 1);
    assert_eq!(shared_playlists.vec[0].name, "Shared Playlist");

    // Verify they are different playlists
    assert_ne!(user_playlists.vec[0].id, shared_playlists.vec[0].id);
}

async fn create_test_playlists(playlist_service: &PlaylistService, user_id: i32) {
    // Create Spotify playlists
    let spotify_playlist1 = CreatePlaylistRequest {
        name: "Spotify Playlist 1".to_string(),
        description: Some("My first Spotify playlist".to_string()),
        origin: PlaylistOrigin::Spotify,
        origin_id: "aae".into(),
    };
    let spotify_playlist2 = CreatePlaylistRequest {
        name: "Spotify Playlist 2".to_string(),
        description: Some("My second Spotify playlist".to_string()),
        origin: PlaylistOrigin::Spotify,
        origin_id: "aae".into(),
    };

    // Create Deezer playlist
    let deezer_playlist = CreatePlaylistRequest {
        name: "Deezer Playlist".to_string(),
        description: Some("My Deezer playlist".to_string()),
        origin: PlaylistOrigin::Deezer,
        origin_id: "aae".into(),
    };

    // Save playlists
    playlist_service
        .create(spotify_playlist1, user_id)
        .await
        .unwrap();
    playlist_service
        .create(spotify_playlist2, user_id)
        .await
        .unwrap();
    playlist_service
        .create(deezer_playlist, user_id)
        .await
        .unwrap();
}
