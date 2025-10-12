#[cfg(test)]
mod shared_playlist_tests {
    use sea_orm::DatabaseConnection;
    use std::sync::Arc;
    use swaptun_services::PlaylistService;
    use swaptun_services::UserService;
    use swaptun_services::{CreatePlaylistRequest, CreateUserRequest};
    use swaptun_models::playlist::PlaylistOrigin;

    async fn setup_test_db() -> Arc<DatabaseConnection> {
        let (db, _container, _user) = swaptun_services::test::setup_db().await;
        Arc::new(db)
    }

    async fn create_test_user(
        service: &UserService,
        username: &str,
        email: &str,
    ) -> i32 {
        let request = CreateUserRequest {
            username: username.to_string(),
            password: "TestPassword123!".to_string(),
            email: email.to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
        };

        service.create_user(request).await.unwrap();
        let user = service.find_by_username(username.to_string()).await.unwrap().unwrap();
        user.id
    }

    async fn create_test_playlist(
        service: &PlaylistService,
        user_id: i32,
        name: &str,
    ) -> i32 {
        let request = CreatePlaylistRequest {
            name: name.to_string(),
            description: Some(format!("Test playlist {}", name)),
            origin: PlaylistOrigin::Spotify,
            origin_id: "test_id".to_string(),
        };

        let playlist = service.create(request, user_id).await.unwrap();
        playlist.id
    }

    #[tokio::test]
    async fn test_share_playlist_with_user() {
        let db = setup_test_db().await;
        let user_service = UserService::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());

        // Create two users
        let owner_id = create_test_user(&user_service, "owner", "owner@test.com").await;
        let receiver_id = create_test_user(&user_service, "receiver", "receiver@test.com").await;

        // Create a playlist
        let playlist_id = create_test_playlist(&playlist_service, owner_id, "Shared Playlist").await;

        // Get user models
        let owner = user_service.find_by_id(owner_id).await.unwrap().unwrap();
        let receiver = user_service.find_by_id(receiver_id).await.unwrap().unwrap();
        let playlist = playlist_service.get_playlist(playlist_id).await.unwrap();

        // Share playlist
        let result = playlist_service
            .share_playlist(&receiver, &playlist, &owner)
            .await;

        assert!(result.is_ok(), "Should be able to share playlist");
    }

    #[tokio::test]
    async fn test_get_shared_playlists_empty() {
        let db = setup_test_db().await;
        let user_service = UserService::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());

        // Create user
        let user_id = create_test_user(&user_service, "lonely_user", "lonely@test.com").await;
        let user = user_service.find_by_id(user_id).await.unwrap().unwrap();

        // Get shared playlists (should be empty)
        let result = playlist_service
            .get_shared_playlists_with_details(&user)
            .await
            .unwrap();

        assert_eq!(
            result.shared_playlists.len(),
            0,
            "User should have no shared playlists"
        );
    }

    #[tokio::test]
    async fn test_get_shared_playlists_single() {
        let db = setup_test_db().await;
        let user_service = UserService::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());

        // Create two users
        let owner_id = create_test_user(&user_service, "alice", "alice@test.com").await;
        let receiver_id = create_test_user(&user_service, "bob", "bob@test.com").await;

        // Create a playlist
        let playlist_id = create_test_playlist(&playlist_service, owner_id, "Rock Classics").await;

        // Get user models
        let owner = user_service.find_by_id(owner_id).await.unwrap().unwrap();
        let receiver = user_service.find_by_id(receiver_id).await.unwrap().unwrap();
        let playlist = playlist_service.get_playlist(playlist_id).await.unwrap();

        // Share playlist
        playlist_service
            .share_playlist(&receiver, &playlist, &owner)
            .await
            .unwrap();

        // Get shared playlists for receiver
        let result = playlist_service
            .get_shared_playlists_with_details(&receiver)
            .await
            .unwrap();

        println!("Shared playlists: {:?}", result);

        assert_eq!(
            result.shared_playlists.len(),
            1,
            "Receiver should have 1 shared playlist"
        );

        let shared = &result.shared_playlists[0];
        assert_eq!(shared.playlist.name, "Rock Classics");
        assert_eq!(shared.playlist.id, playlist_id);
        assert_eq!(shared.shared_by.id, owner_id);
        assert_eq!(shared.shared_by.username, "alice");
    }

    #[tokio::test]
    async fn test_get_shared_playlists_multiple() {
        let db = setup_test_db().await;
        let user_service = UserService::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());

        // Create three users
        let user1_id = create_test_user(&user_service, "user1", "user1@test.com").await;
        let user2_id = create_test_user(&user_service, "user2", "user2@test.com").await;
        let receiver_id = create_test_user(&user_service, "receiver2", "receiver2@test.com").await;

        // Create playlists
        let playlist1_id = create_test_playlist(&playlist_service, user1_id, "Jazz").await;
        let playlist2_id = create_test_playlist(&playlist_service, user2_id, "Classical").await;

        // Get user models
        let user1 = user_service.find_by_id(user1_id).await.unwrap().unwrap();
        let user2 = user_service.find_by_id(user2_id).await.unwrap().unwrap();
        let receiver = user_service.find_by_id(receiver_id).await.unwrap().unwrap();
        let playlist1 = playlist_service.get_playlist(playlist1_id).await.unwrap();
        let playlist2 = playlist_service.get_playlist(playlist2_id).await.unwrap();

        // Share both playlists with receiver
        playlist_service
            .share_playlist(&receiver, &playlist1, &user1)
            .await
            .unwrap();
        playlist_service
            .share_playlist(&receiver, &playlist2, &user2)
            .await
            .unwrap();

        // Get shared playlists for receiver
        let result = playlist_service
            .get_shared_playlists_with_details(&receiver)
            .await
            .unwrap();

        assert_eq!(
            result.shared_playlists.len(),
            2,
            "Receiver should have 2 shared playlists"
        );

        // Verify playlist names
        let playlist_names: Vec<&str> = result
            .shared_playlists
            .iter()
            .map(|sp| sp.playlist.name.as_str())
            .collect();

        assert!(
            playlist_names.contains(&"Jazz"),
            "Should contain Jazz playlist"
        );
        assert!(
            playlist_names.contains(&"Classical"),
            "Should contain Classical playlist"
        );

        // Verify shared_by information
        let sharer_usernames: Vec<&str> = result
            .shared_playlists
            .iter()
            .map(|sp| sp.shared_by.username.as_str())
            .collect();

        assert!(
            sharer_usernames.contains(&"user1"),
            "Should have share from user1"
        );
        assert!(
            sharer_usernames.contains(&"user2"),
            "Should have share from user2"
        );
    }

    #[tokio::test]
    async fn test_owner_does_not_see_own_playlist_as_shared() {
        let db = setup_test_db().await;
        let user_service = UserService::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());

        // Create two users
        let owner_id = create_test_user(&user_service, "owner2", "owner2@test.com").await;
        let receiver_id = create_test_user(&user_service, "receiver3", "receiver3@test.com").await;

        // Create a playlist
        let playlist_id = create_test_playlist(&playlist_service, owner_id, "My Playlist").await;

        // Get user models
        let owner = user_service.find_by_id(owner_id).await.unwrap().unwrap();
        let receiver = user_service.find_by_id(receiver_id).await.unwrap().unwrap();
        let playlist = playlist_service.get_playlist(playlist_id).await.unwrap();

        // Share playlist
        playlist_service
            .share_playlist(&receiver, &playlist, &owner)
            .await
            .unwrap();

        // Owner should not see it in their shared playlists
        let owner_shared = playlist_service
            .get_shared_playlists_with_details(&owner)
            .await
            .unwrap();

        assert_eq!(
            owner_shared.shared_playlists.len(),
            0,
            "Owner should not see their own playlist as shared"
        );

        // But receiver should
        let receiver_shared = playlist_service
            .get_shared_playlists_with_details(&receiver)
            .await
            .unwrap();

        assert_eq!(
            receiver_shared.shared_playlists.len(),
            1,
            "Receiver should see the shared playlist"
        );
    }

    #[tokio::test]
    async fn test_shared_playlist_details_complete() {
        let db = setup_test_db().await;
        let user_service = UserService::new(db.clone());
        let playlist_service = PlaylistService::new(db.clone());

        // Create users with specific data
        let owner_request = CreateUserRequest {
            username: "john_doe".to_string(),
            password: "TestPassword123!".to_string(),
            email: "john@test.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };
        user_service.create_user(owner_request).await.unwrap();
        let owner = user_service.find_by_username("john_doe".to_string()).await.unwrap().unwrap();

        let receiver_id = create_test_user(&user_service, "jane", "jane@test.com").await;
        let receiver = user_service.find_by_id(receiver_id).await.unwrap().unwrap();

        // Create playlist
        let playlist_id = create_test_playlist(&playlist_service, owner.id, "Test List").await;
        let playlist = playlist_service.get_playlist(playlist_id).await.unwrap();

        // Share
        playlist_service
            .share_playlist(&receiver, &playlist, &owner)
            .await
            .unwrap();

        // Get and verify details
        let result = playlist_service
            .get_shared_playlists_with_details(&receiver)
            .await
            .unwrap();

        assert_eq!(result.shared_playlists.len(), 1);

        let shared = &result.shared_playlists[0];

        // Verify playlist details
        assert_eq!(shared.playlist.name, "Test List");
        assert_eq!(shared.playlist.user_id, owner.id);

        // Verify shared_by details
        assert_eq!(shared.shared_by.username, "john_doe");
        assert_eq!(shared.shared_by.id, owner.id);

        // Verify shared_at is set
        // Just check it exists, don't compare exact time
        assert!(shared.shared_at.timestamp() > 0);
    }
}
