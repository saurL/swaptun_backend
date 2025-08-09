use swaptun_services::TestDatabase;
use swaptun_services::{music::dto::CreateMusicRequest, MusicService};
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_music_success() {
    let test_db = TestDatabase::new().await;
    let music_service = MusicService::new(test_db.get_db());
    let crate_music_request = CreateMusicRequest {
        title: "New Music".to_string(),
        artist: "New Artist".to_string(),
        album: "New Album".to_string(),
        release_date: "2025-01-01".parse().unwrap(),
        genre: Some("Pop".to_string()),
        description: Some("New Description".to_string()),
    };
    let result = music_service.create(crate_music_request).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());

    let music = music_service
        .find_by_id(
            "New Music".to_string(),
            "New Artist".to_string(),
            "New Album".to_string(),
        )
        .await
        .unwrap();
    println!("Music: {:?}", music);
    assert!(music.is_some());
    assert_eq!(music.unwrap().title, "New Music".to_string());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_music_success() {
    let test_db = TestDatabase::new().await;
    let music_service = MusicService::new(test_db.get_db());

    let crate_music_request = CreateMusicRequest {
        title: "New Music".to_string(),
        artist: "New Artist".to_string(),
        album: "New Album".to_string(),
        release_date: "2025-01-01".parse().unwrap(),
        genre: Some("Pop".to_string()),
        description: Some("New Description".to_string()),
    };
    let created_music = music_service.create(crate_music_request).await.unwrap();

    // Supprime la musique
    let result = music_service
        .delete(
            created_music.title.clone(),
            created_music.artist.clone(),
            created_music.album.clone(),
        )
        .await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_music_duplicate() {
    let test_db = TestDatabase::new().await;
    let music_service = MusicService::new(test_db.get_db());

    let crate_music_request = CreateMusicRequest {
        title: "New Music".to_string(),
        artist: "New Artist".to_string(),
        album: "New Album".to_string(),
        release_date: "2025-01-01".parse().unwrap(),
        genre: Some("Pop".to_string()),
        description: Some("New Description".to_string()),
    };

    // Crée une première musique
    let result1 = music_service.create(crate_music_request).await;
    println!("First Creation Result: {:?}", result1);
    assert!(result1.is_ok());

    let crate_music_request2 = CreateMusicRequest {
        title: "New Music".to_string(),
        artist: "New Artist".to_string(),
        album: "New Album".to_string(),
        release_date: "2025-01-01".parse().unwrap(),
        genre: Some("Pop".to_string()),
        description: Some("New Description".to_string()),
    };

    // Essaye de créer la même musique
    let result2 = music_service.create(crate_music_request2).await;
    println!("Second Creation Result: {:?}", result2);
    assert!(result2.is_ok());
    let all_music = music_service.find_all().await.unwrap();
    println!("All Music: {:?}", all_music);
    // Vérifie que la musique n'a pas été dupliquée
    assert_eq!(all_music.len(), 1);
    test_db.drop().await;
}
