use swaptun_services::TestDatabase;
use swaptun_services::{CreateUserRequest, UpdateUserRequest, UserService};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_search_users_exclude_self_enabled() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Create additional users for testing
    let create_user_request1 = CreateUserRequest {
        username: "test_user_1".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Test".to_string(),
        last_name: "User1".to_string(),
        email: "test_user_1@gmail.com".to_string(),
    };
    let _ = user_service
        .create_user(create_user_request1)
        .await
        .unwrap();

    let create_user_request2 = CreateUserRequest {
        username: "test_user_2".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Test".to_string(),
        last_name: "User2".to_string(),
        email: "test_user_2@gmail.com".to_string(),
    };
    let _ = user_service
        .create_user(create_user_request2)
        .await
        .unwrap();

    // Get the current user ID
    let current_user_id = test_db.get_user().id;

    // Test search with exclude_self enabled
    let request = swaptun_services::GetUsersRequest {
        include_deleted: Some(false),
        search: None, // No search filter to get all users
        search_field: None,
        limit: Some(100),
        offset: Some(0),
        friends_priority: false,
        exclude_friends: false,
        exclude_self: Some(true),
    };

    let users = user_service
        .get_users(current_user_id, request)
        .await
        .unwrap();

    // Should not include the current user in the results
    let found_self = users.iter().any(|u| u.id == current_user_id);
    assert!(!found_self, "Current user should be excluded from results");

    // Should include other users
    assert!(!users.is_empty(), "Should find other users");

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_search_users_exclude_self_disabled() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Create an additional user for testing
    let create_user_request = CreateUserRequest {
        username: "another_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Another".to_string(),
        last_name: "User".to_string(),
        email: "another_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await.unwrap();

    // Get the current user ID
    let current_user_id = test_db.get_user().id;

    // Test search with exclude_self disabled (default behavior)
    let request = swaptun_services::GetUsersRequest {
        include_deleted: Some(false),
        search: None,
        search_field: None,
        limit: Some(100),
        offset: Some(0),
        friends_priority: false,
        exclude_friends: false,
        exclude_self: Some(false),
    };

    let users = user_service
        .get_users(current_user_id, request)
        .await
        .unwrap();

    // Should include the current user in the results
    let found_self = users.iter().any(|u| u.id == current_user_id);
    assert!(found_self, "Current user should be included in results");

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_search_users_exclude_self_default() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Create an additional user for testing
    let create_user_request = CreateUserRequest {
        username: "yet_another_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Yet".to_string(),
        last_name: "Another".to_string(),
        email: "yet_another_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await.unwrap();

    // Get the current user ID
    let current_user_id = test_db.get_user().id;

    // Test search with exclude_self not specified (should default to false)
    let request = swaptun_services::GetUsersRequest {
        include_deleted: Some(false),
        search: None,
        search_field: None,
        limit: Some(100),
        offset: Some(0),
        friends_priority: false,
        exclude_friends: false,
        exclude_self: None, // Not specified, should default to false
    };

    let users = user_service
        .get_users(current_user_id, request)
        .await
        .unwrap();

    // Should include the current user by default
    let found_self = users.iter().any(|u| u.id == current_user_id);
    assert!(
        found_self,
        "Current user should be included by default when exclude_self is not specified"
    );

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_search_users_exclude_self_with_search_filter() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Update the current user's username to match the search
    let current_user_id = test_db.get_user().id;
    let update_request = UpdateUserRequest {
        username: Some("searchable_user".to_string()),
        first_name: None,
        last_name: None,
        email: None,
    };
    let _ = user_service
        .update_user(update_request, current_user_id)
        .await
        .unwrap();

    // Create another user with a similar username
    let create_user_request = CreateUserRequest {
        username: "searchable_friend".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Searchable".to_string(),
        last_name: "Friend".to_string(),
        email: "searchable_friend@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await.unwrap();

    // Test search with exclude_self enabled and a search filter
    let request = swaptun_services::GetUsersRequest {
        include_deleted: Some(false),
        search: Some("searchable".to_string()),
        search_field: Some(swaptun_services::model::SearchField::Username),
        limit: Some(100),
        offset: Some(0),
        friends_priority: false,
        exclude_friends: false,
        exclude_self: Some(true),
    };

    let users = user_service
        .get_users(current_user_id, request)
        .await
        .unwrap();

    // Should not include the current user even though username matches search
    let found_self = users.iter().any(|u| u.id == current_user_id);
    assert!(
        !found_self,
        "Current user should be excluded even with matching search term"
    );

    // Should include other matching users
    assert!(!users.is_empty(), "Should find other matching users");

    test_db.drop().await;
}
