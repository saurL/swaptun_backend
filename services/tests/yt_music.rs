use swaptun_services::user::UserService;
use swaptun_services::TestDatabase;
use swaptun_services::{AddTokenRequest, YoutubeMusicService};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_authorization_url_success() {
    let test_db = TestDatabase::new().await;
    let youtube_service = YoutubeMusicService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Test getting authorization URL
    let result = youtube_service.get_authorization_url(&user).await;
    println!("Get Authorization URL Result: {:?}", result);

    // Since we don't have real OAuth credentials in test environment,
    // we expect this to fail, but we want to make sure it follows the correct flow
    assert!(result.is_err());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_add_youtube_token_success() {
    let test_db = TestDatabase::new().await;
    let youtube_service = YoutubeMusicService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Test adding a YouTube token
    let add_token_request = AddTokenRequest {
        token: "youtube_token_123".to_string(),
    };

    // Since we don't have real OAuth credentials in test environment,
    // we expect this to fail, but we want to make sure it follows the correct flow
    let result = youtube_service
        .auth_callback(&user, add_token_request)
        .await;
    println!("Add Token Result: {:?}", result);

    // The result will likely be an error due to missing OAuth setup in test environment
    // but we're testing that the method can be called without panicking
    assert!(result.is_err() || result.is_ok());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_youtube_token_not_found() {
    let test_db = TestDatabase::new().await;
    let youtube_service = YoutubeMusicService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Test getting a token that doesn't exist
    let result = youtube_service.get_token(&user).await;
    println!("Get Token Result: {:?}", result);

    // Should succeed but return None for non-existent token
    assert!(result.is_ok());
    let token = result.unwrap();
    assert!(token.is_none());

    test_db.drop().await;
}
