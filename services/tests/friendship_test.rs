#[cfg(test)]
mod friendship_tests {
    use std::sync::Arc;
    use swaptun_services::CreateUserRequest;
    use swaptun_services::TestDatabase;
    use swaptun_services::UserService;

    async fn setup_test_db() -> Arc<TestDatabase> {
        let db = TestDatabase::new().await;
        Arc::new(db)
    }

    async fn create_test_user(service: &UserService, username: &str, email: &str) -> i32 {
        let request = CreateUserRequest {
            username: username.to_string(),
            password: "TestPassword123!".to_string(),
            email: email.to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
        };

        service.create_user(request).await.unwrap();
        let user = service
            .find_by_username(username.into())
            .await
            .unwrap()
            .unwrap();
        user.id
    }

    #[tokio::test]
    async fn test_add_friend_unidirectional_not_mutual_yet() {
        let test_database: Arc<TestDatabase> = setup_test_db().await;
        let service = UserService::new(test_database.get_db().clone());

        // Create two users
        let user1_id = create_test_user(&service, "user1", "user1@test.com").await;
        let user2_id = create_test_user(&service, "user2", "user2@test.com").await;

        // User1 adds User2 as friend (unidirectional)
        let result = service.add_friend(user1_id, user2_id).await;
        assert!(
            result.is_ok(),
            "User1 should be able to add User2 as friend"
        );

        // User1 should NOT see User2 as friend yet (not mutual)
        let user1 = service.find_by_id(user1_id).await.unwrap().unwrap();
        let friends1 = service.get_friends(&user1).await.unwrap();
        assert_eq!(
            friends1.len(),
            0,
            "User1 should have 0 friends (not mutual yet)"
        );

        // User2 should also have 0 friends (hasn't added User1 back)
        let user2 = service.find_by_id(user2_id).await.unwrap().unwrap();
        let friends2 = service.get_friends(&user2).await.unwrap();
        assert_eq!(
            friends2.len(),
            0,
            "User2 should have 0 friends (hasn't added back)"
        );
    }

    #[tokio::test]
    async fn test_mutual_friendship_both_users_add_each_other() {
        let test_database: Arc<TestDatabase> = setup_test_db().await;
        let service = UserService::new(test_database.get_db().clone());

        // Create two users
        let user1_id = create_test_user(&service, "alice", "alice@test.com").await;
        let user2_id = create_test_user(&service, "bob", "bob@test.com").await;

        // User1 adds User2
        service.add_friend(user1_id, user2_id).await.unwrap();

        // Not mutual yet
        let user1 = service.find_by_id(user1_id).await.unwrap().unwrap();
        let friends1_before = service.get_friends(&user1).await.unwrap();
        assert_eq!(friends1_before.len(), 0, "Not mutual yet");

        // User2 adds User1 back
        service.add_friend(user2_id, user1_id).await.unwrap();

        // NOW both should see each other as friends (mutual)
        let friends1_after = service.get_friends(&user1).await.unwrap();
        assert_eq!(friends1_after.len(), 1, "User1 should now have 1 friend");
        assert_eq!(
            friends1_after[0].id, user2_id,
            "User1's friend should be User2"
        );

        let user2 = service.find_by_id(user2_id).await.unwrap().unwrap();
        let friends2_after = service.get_friends(&user2).await.unwrap();
        assert_eq!(friends2_after.len(), 1, "User2 should now have 1 friend");
        assert_eq!(
            friends2_after[0].id, user1_id,
            "User2's friend should be User1"
        );
    }

    #[tokio::test]
    async fn test_cannot_add_same_friend_twice() {
        let test_db = TestDatabase::new().await;
        let db = test_db.get_db();
        let service = UserService::new(db.clone());

        // Create two users
        let user1_id = create_test_user(&service, "alice", "alice@test.com").await;
        let user2_id = create_test_user(&service, "bob", "bob@test.com").await;

        // Alice adds Bob as friend
        let result1 = service.add_friend(user1_id, user2_id).await;
        assert!(result1.is_ok(), "Alice should be able to add Bob as friend");

        // Alice trying to add Bob again should fail
        let result2 = service.add_friend(user1_id, user2_id).await;
        assert!(
            result2.is_err(),
            "Alice should not be able to add Bob again"
        );

        // Verify the error message
        match result2 {
            Err(err) => {
                let err_msg = format!("{}", err);
                assert!(
                    err_msg.contains("already added"),
                    "Error should mention that user already added this friend"
                );
            }
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    #[tokio::test]
    async fn test_remove_friend_unidirectional() {
        let test_db = TestDatabase::new().await;
        let db = test_db.get_db();
        let service = UserService::new(db.clone());

        // Create two users
        let user1_id = create_test_user(&service, "charlie", "charlie@test.com").await;
        let user2_id = create_test_user(&service, "david", "david@test.com").await;

        // Create mutual friendship (both add each other)
        service.add_friend(user1_id, user2_id).await.unwrap();
        service.add_friend(user2_id, user1_id).await.unwrap();

        // Verify friendship exists for both
        let user1 = service.find_by_id(user1_id).await.unwrap().unwrap();
        let friends1_before = service.get_friends(&user1).await.unwrap();
        assert_eq!(friends1_before.len(), 1);

        let user2 = service.find_by_id(user2_id).await.unwrap().unwrap();
        let friends2_before = service.get_friends(&user2).await.unwrap();
        assert_eq!(friends2_before.len(), 1);

        // User1 removes User2 (should remove bidirectionally)
        let result = service.remove_friend(user1_id, user2_id).await;
        assert!(result.is_ok(), "Should be able to remove friend");

        // Verify friendship removed for both users
        let friends1_after = service.get_friends(&user1).await.unwrap();
        assert_eq!(friends1_after.len(), 0, "User1 should have no friends");

        let friends2_after = service.get_friends(&user2).await.unwrap();
        assert_eq!(friends2_after.len(), 0, "User2 should have no friends");
    }

    #[tokio::test]
    async fn test_cannot_add_self_as_friend() {
        let test_db = TestDatabase::new().await;
        let db = test_db.get_db();
        let service = UserService::new(db.clone());

        // Create user
        let user_id = create_test_user(&service, "solo", "solo@test.com").await;

        // Try to add self as friend
        let result = service.add_friend(user_id, user_id).await;

        assert!(
            result.is_err(),
            "User should not be able to add themselves as friend"
        );
    }

    #[tokio::test]
    async fn test_add_multiple_mutual_friends() {
        let test_db = TestDatabase::new().await;
        let db = test_db.get_db();
        let service = UserService::new(db.clone());

        // Create users
        let user1_id = create_test_user(&service, "eve", "eve@test.com").await;
        let user2_id = create_test_user(&service, "frank", "frank@test.com").await;
        let user3_id = create_test_user(&service, "grace", "grace@test.com").await;

        // User1 adds User2 and User3
        service.add_friend(user1_id, user2_id).await.unwrap();
        service.add_friend(user1_id, user3_id).await.unwrap();

        // User2 and User3 add User1 back (to create mutual friendships)
        service.add_friend(user2_id, user1_id).await.unwrap();
        service.add_friend(user3_id, user1_id).await.unwrap();

        // Verify User1 has 2 friends
        let user1 = service.find_by_id(user1_id).await.unwrap().unwrap();
        let friends = service.get_friends(&user1).await.unwrap();
        assert_eq!(friends.len(), 2, "User1 should have 2 friends");

        let friend_ids: Vec<i32> = friends.iter().map(|f| f.id).collect();
        assert!(friend_ids.contains(&user2_id), "User2 should be a friend");
        assert!(friend_ids.contains(&user3_id), "User3 should be a friend");
    }

    #[tokio::test]
    async fn test_get_friends_returns_correct_users() {
        let test_db = TestDatabase::new().await;
        let db = test_db.get_db();
        let service = UserService::new(db.clone());

        // Create users
        let user1_id = create_test_user(&service, "henry", "henry@test.com").await;
        let user2_id = create_test_user(&service, "iris", "iris@test.com").await;

        // Create mutual friendship (both add each other)
        service.add_friend(user1_id, user2_id).await.unwrap();
        service.add_friend(user2_id, user1_id).await.unwrap();

        // Get friends of User1
        let user1 = service.find_by_id(user1_id).await.unwrap().unwrap();
        let friends = service.get_friends(&user1).await.unwrap();

        assert_eq!(friends.len(), 1);
        assert_eq!(friends[0].username, "iris");
        assert_eq!(friends[0].email, "iris@test.com");
    }

    #[tokio::test]
    async fn test_asymmetric_friendship() {
        let test_db = TestDatabase::new().await;
        let db = test_db.get_db();
        let service = UserService::new(db.clone());

        // Create users
        let user1_id = create_test_user(&service, "john", "john@test.com").await;
        let user2_id = create_test_user(&service, "jane", "jane@test.com").await;

        // Only User1 adds User2 (asymmetric)
        service.add_friend(user1_id, user2_id).await.unwrap();

        // User1 should NOT see User2 in friends (not mutual)
        let user1 = service.find_by_id(user1_id).await.unwrap().unwrap();
        let friends1 = service.get_friends(&user1).await.unwrap();
        assert_eq!(
            friends1.len(),
            0,
            "User1 should not see User2 as friend (not mutual)"
        );

        // User2 should NOT see User1 in friends
        let user2 = service.find_by_id(user2_id).await.unwrap().unwrap();
        let friends2 = service.get_friends(&user2).await.unwrap();
        assert_eq!(
            friends2.len(),
            0,
            "User2 should not see User1 as friend"
        );
    }

    #[tokio::test]
    async fn test_both_users_can_add_each_other_independently() {
        let test_db = TestDatabase::new().await;
        let db = test_db.get_db();
        let service = UserService::new(db.clone());

        // Create users
        let user1_id = create_test_user(&service, "tom", "tom@test.com").await;
        let user2_id = create_test_user(&service, "jerry", "jerry@test.com").await;

        // User1 adds User2
        let result1 = service.add_friend(user1_id, user2_id).await;
        assert!(result1.is_ok(), "User1 should be able to add User2");

        // User2 can also add User1 (this should succeed, not fail)
        let result2 = service.add_friend(user2_id, user1_id).await;
        assert!(
            result2.is_ok(),
            "User2 should be able to add User1 independently"
        );

        // Now they are mutual friends
        let user1 = service.find_by_id(user1_id).await.unwrap().unwrap();
        let friends1 = service.get_friends(&user1).await.unwrap();
        assert_eq!(friends1.len(), 1);

        let user2 = service.find_by_id(user2_id).await.unwrap().unwrap();
        let friends2 = service.get_friends(&user2).await.unwrap();
        assert_eq!(friends2.len(), 1);
    }
}
