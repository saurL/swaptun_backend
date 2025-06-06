use swaptun_services::TestDatabase;

use swaptun_models::PlaylistOrigin;
use swaptun_services::{CreatePlaylistRequest, GetPlaylistsParams, PlaylistService};

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
