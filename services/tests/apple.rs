use swaptun_services::user::UserService;
use swaptun_services::TestDatabase;
use swaptun_services::{AddTokenRequest, AppleMusicService};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_and_verify_apple_token() {
    let test_db = TestDatabase::new().await;
    let apple_service = AppleMusicService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Create an Apple Music token
    let create_token_request: AddTokenRequest = AddTokenRequest {
        token: "apple_music_token_123".to_string(),
    };

    let result = apple_service
        .add_user_token(create_token_request, user.id)
        .await;
    println!("Create Apple Token Result: {:?}", result);
    assert!(result.is_ok());

    // Verify that the token exists
    let token = apple_service.get_token(&user).await;
    println!("Get Apple Token Result: {:?}", token);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(token.is_some());
    let token = token.unwrap();
    assert_eq!(token.access_token, "apple_music_token_123");
    assert_eq!(token.user_id, 1);

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_apple_token_nonexistent() {
    let test_db = TestDatabase::new().await;
    let apple_service = AppleMusicService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Try to get token for user without token
    let token = apple_service.get_token(&user).await;
    println!("Get Non-existent Apple Token Result: {:?}", token);
    assert!(token.is_ok());
    assert!(token.unwrap().is_none());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_save_apple_token() {
    let test_db = TestDatabase::new().await;
    let apple_service = AppleMusicService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Create and save a token
    let create_token_request: AddTokenRequest = AddTokenRequest {
        token: "apple_music_token_456".to_string(),
    };

    let result = apple_service
        .add_user_token(create_token_request, user.id)
        .await;
    println!("Save Apple Token Result: {:?}", result);
    assert!(result.is_ok());

    // Verify the token was saved
    let token = apple_service.get_token(&user).await;
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(token.is_some());

    test_db.drop().await;
}
