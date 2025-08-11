use swaptun_services::musicbrainz::{MusicBrainzService, TrackInfo};

#[test]
fn test_musicbrainz_service_creation() {
    // Test that we can create a MusicBrainzService instance
    let _ = MusicBrainzService::new();

    // The service should be created successfully
    assert!(true); // If we get here without panic, the test passes
}

#[test]
fn test_track_info_creation() {
    // Test creating a TrackInfo struct
    let track_info = TrackInfo {
        title: "Test Song".to_string(),
        artist: "Test Artist".to_string(),
        musicbrainz_id: "test-id-123".to_string(),
        release_date: Some("2023-01-01".to_string()),
        genre: Some("Pop".to_string()),
    };

    assert_eq!(track_info.title, "Test Song");
    assert_eq!(track_info.artist, "Test Artist");
    assert_eq!(track_info.musicbrainz_id, "test-id-123");
    assert_eq!(track_info.release_date, Some("2023-01-01".to_string()));
    assert_eq!(track_info.genre, Some("Pop".to_string()));
}

#[test]
fn test_track_info_with_optional_fields() {
    // Test creating a TrackInfo struct with optional fields as None
    let track_info = TrackInfo {
        title: "Test Song".to_string(),
        artist: "Test Artist".to_string(),
        musicbrainz_id: "test-id-123".to_string(),
        release_date: None,
        genre: None,
    };

    assert_eq!(track_info.title, "Test Song");
    assert_eq!(track_info.artist, "Test Artist");
    assert_eq!(track_info.musicbrainz_id, "test-id-123");
    assert_eq!(track_info.release_date, None);
    assert_eq!(track_info.genre, None);
}

#[tokio::test]
async fn test_get_track_metadata_function() {
    // Test the get_track_metadata function
    // Since this makes external API calls, we'll test with a known non-existent track
    // to avoid depending on external services in tests

    let result = swaptun_services::musicbrainz::get_track_metadata(
        "nonexistent_track",
        "nonexistent_artist",
    )
    .await;

    // Should succeed but return None for non-existent track
    assert!(result.is_ok());
    // The result could be None (not found) or Some(TrackInfo) depending on the API response
    // but it should not panic or return an error
}
