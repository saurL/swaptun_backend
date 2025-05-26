use swaptun_services::DeezerService;
use swaptun_services::TestDatabase;
use swaptun_services::UserService;
use swaptun_services::{AddTokenRequest, DeleteTokenRequest};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_and_verify_deezer_token() {
    let test_db = TestDatabase::new().await;
    let deezer_service = DeezerService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());

    // Get the user first
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Crée un utilisateur et son token Deezer
    let create_token_request = AddTokenRequest {
        token: "deezer_token_123".to_string(),
    };

    let result = deezer_service.add_token(create_token_request, user).await;
    println!("Create Token Result: {:?}", result);
    assert!(result.is_ok());

    // Vérifie que le token existe
    let token = deezer_service.get_token_by_user_id(1).await;
    println!("Get Token Result: {:?}", token);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(token.is_some());
    let token = token.unwrap();
    assert_eq!(token.token, "deezer_token_123");
    assert_eq!(token.user_id, 1);

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_deezer_token() {
    let test_db = TestDatabase::new().await;
    let deezer_service = DeezerService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    // Crée un utilisateur et son token Deezer
    let create_token_request = AddTokenRequest {
        token: "deezer_token_123".to_string(),
    };
    let user = user_service.get_user(1).await.unwrap().unwrap();

    deezer_service
        .add_token(create_token_request, user)
        .await
        .unwrap();

    // Supprime le token
    let delete_request = DeleteTokenRequest { user_id: 1 };
    let delete_result = deezer_service.delete_token(delete_request).await;
    println!("Delete Token Result: {:?}", delete_result);
    assert!(delete_result.is_ok());

    // Vérifie que le token a été supprimé
    let token = deezer_service.get_token_by_user_id(1).await;
    println!("Get Token After Delete Result: {:?}", token);
    assert!(token.is_ok());
    assert!(token.unwrap().is_none());

    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_deezer_token() {
    let test_db = TestDatabase::new().await;
    let deezer_service = DeezerService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());

    // Crée un utilisateur et son token Deezer
    let create_token_request = AddTokenRequest {
        token: "deezer_token_123".to_string(),
    };
    let user = user_service.get_user(1).await.unwrap().unwrap();
    println!("Get User Result: {:?}", user);
    deezer_service
        .add_token(create_token_request, user.clone())
        .await
        .unwrap();

    // Vérifie que le token peut être récupéré
    let token = deezer_service.get_token(user).await;
    println!("Get Token Result: {:?}", token);
    assert!(token.is_ok());
    let token = token.unwrap();

    assert_eq!(token.token, "deezer_token_123");
    assert_eq!(token.user_id, 1);

    // Vérifie qu'un utilisateur inexistant retourne None
    let non_existent_token = deezer_service.get_token_by_user_id(999).await;
    println!("Get Non-Existent Token Result: {:?}", non_existent_token);
    assert!(non_existent_token.is_ok());
    assert!(non_existent_token.unwrap().is_none());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_update_deezer_token_implicit() {
    let test_db = TestDatabase::new().await;
    let deezer_service = DeezerService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    let user = user_service.get_user(1).await.unwrap().unwrap();

    // Crée un utilisateur et son token Deezer
    let create_token_request = AddTokenRequest {
        token: "old_deezer_token".to_string(),
    };

    deezer_service
        .add_token(create_token_request, user.clone())
        .await
        .unwrap();

    // Met à jour le token implicitement en utilisant add_token
    let update_token_request = AddTokenRequest {
        token: "new_deezer_token".to_string(),
    };

    let update_result = deezer_service.add_token(update_token_request, user).await;
    println!("Update Token Result: {:?}", update_result);
    assert!(update_result.is_ok());

    // Vérifie que le token a été mis à jour
    let token = deezer_service.get_token_by_user_id(1).await;
    println!("Get Token After Update Result: {:?}", token);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(token.is_some());
    let token = token.unwrap();
    assert_eq!(token.token, "new_deezer_token");
    assert_eq!(token.user_id, 1);

    test_db.drop().await;
}
