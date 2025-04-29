use swaptun_services::user::UserService;
use swaptun_services::{AddTokenRequest, DeleteTokenRequest, SpotifyService, UpdateTokenRequest};
mod test_database;
use test_database::TestDatabase;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_and_verify_spotify_token() {
    let test_db = TestDatabase::new().await;
    let spotify_service = SpotifyService::new(test_db.get_db());

    // Crée un utilisateur et son token Spotify
    let create_token_request: AddTokenRequest = AddTokenRequest {
        user_id: 1,
        token: "spotify_token_123".to_string(),
    };

    let result = spotify_service.add_token(create_token_request).await;
    println!("Create Token Result: {:?}", result);
    assert!(result.is_ok());

    // Vérifie que le token existe
    let token = spotify_service.get_token_by_user_id(1).await;
    println!("Get Token Result: {:?}", token);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(token.is_some());
    let token = token.unwrap();
    assert_eq!(token.token, "spotify_token_123");
    assert_eq!(token.user_id, 1);

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_spotify_token() {
    let test_db = TestDatabase::new().await;
    let spotify_service = SpotifyService::new(test_db.get_db());

    // Crée un utilisateur et son token Spotify
    let create_token_request = AddTokenRequest {
        user_id: 1,
        token: "spotify_token_123".to_string(),
    };

    spotify_service
        .add_token(create_token_request)
        .await
        .unwrap();

    // Supprime le token
    let delete_request = DeleteTokenRequest { user_id: 1 };
    let delete_result = spotify_service.delete_token(delete_request).await;
    println!("Delete Token Result: {:?}", delete_result);
    assert!(delete_result.is_ok());

    // Vérifie que le token a été supprimé
    let token = spotify_service.get_token_by_user_id(1).await;
    println!("Get Token After Delete Result: {:?}", token);
    assert!(token.is_ok());
    assert!(token.unwrap().is_none());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_update_spotify_token() {
    let test_db = TestDatabase::new().await;
    let spotify_service = SpotifyService::new(test_db.get_db());

    // Crée un utilisateur et son token Spotify
    let create_token_request = AddTokenRequest {
        user_id: 1,
        token: "old_spotify_token".to_string(),
    };

    spotify_service
        .add_token(create_token_request)
        .await
        .unwrap();

    // Met à jour le token
    let update_token_request = UpdateTokenRequest {
        new_token: "new_spotify_token".to_string(),
        user_id: 1,
    };

    let update_result = spotify_service.update_token(update_token_request).await;
    println!("Update Token Result: {:?}", update_result);
    assert!(update_result.is_ok());

    // Vérifie que le token a été mis à jour
    let token = spotify_service.get_token_by_user_id(1).await;
    println!("Get Token After Update Result: {:?}", token);
    assert!(token.is_ok());
    let token = token.unwrap();
    assert!(token.is_some());
    let token = token.unwrap();
    assert_eq!(token.token, "new_spotify_token");
    assert_eq!(token.user_id, 1);

    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_spotify_token() {
    let test_db = TestDatabase::new().await;
    let spotify_service = SpotifyService::new(test_db.get_db());
    let user_service = UserService::new(test_db.get_db());
    // Crée un utilisateur et son token Spotify
    let create_token_request = AddTokenRequest {
        user_id: 1,
        token: "spotify_token_123".to_string(),
    };

    spotify_service
        .add_token(create_token_request)
        .await
        .unwrap();
    let user = user_service.get_user(1).await.unwrap().unwrap();
    println!("Get User Result: {:?}", user);

    // Vérifie que le token peut être récupéré
    let token = spotify_service.get_token(user).await;
    println!("Get Token Result: {:?}", token);
    assert!(token.is_ok());
    let token = token.unwrap();

    assert_eq!(token.token, "spotify_token_123");
    assert_eq!(token.user_id, 1);

    // Vérifie qu'un utilisateur inexistant retourne None
    let non_existent_token = spotify_service.get_token_by_user_id(999).await;
    println!("Get Non-Existent Token Result: {:?}", non_existent_token);
    assert!(non_existent_token.is_ok());
    assert!(non_existent_token.unwrap().is_none());

    test_db.drop().await;
}
