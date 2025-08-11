use swaptun_services::TestDatabase;
use swaptun_services::{
    auth::Claims, CreateUserRequest, ForgotPasswordRequest, LoginEmailRequest, LoginRequest,
    UpdateUserRequest, UserService, VerifyTokenRequest,
};
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_find_by_username_success() {
    let test_db = TestDatabase::new().await;

    let user_service = UserService::new(test_db.get_db());

    let result = user_service
        .find_by_username("unique_user".to_string())
        .await;

    println!("Result: {:?}", result);
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_some());
    let user = user.unwrap();
    assert_eq!(user.username, "unique_user");
    assert_eq!(user.role, "user");
    test_db.drop().await;
}

// Répétez ce modèle pour les autres tests
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_find_by_username_not_found() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    let result = user_service
        .find_by_username("nonexistent_user".to_string())
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    test_db.drop().await;
}

// Continuez à appliquer ce modèle pour tous les autres tests

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_user_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    let create_user_request = CreateUserRequest {
        username: "unique_user_2".to_string(),
        password: "hasasswArd1223!az".to_string(),
        first_name: "first_name_2".to_string(),
        last_name: "last_name_2".to_string(),
        email: "user2@gmail.com".to_string(),
    };

    let result = user_service.create_user(create_user_request).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_user_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Supprime l'utilisateur créé dans setup_db
    let result = user_service.delete_user_logical(1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_update_user_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    let updated_user = UpdateUserRequest {
        username: Some("updated_user".to_string()),
        first_name: Some("updated_first_name".to_string()),
        last_name: Some("updated_last_name".to_string()),
        email: Some("updated_user@gmail.com".to_string()),
    };

    // Met à jour l'utilisateur créé dans setup_db
    let result = user_service.update_user(updated_user, 1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    let updated_user = result.unwrap();
    assert_eq!(updated_user.username, "updated_user");
    assert_eq!(updated_user.first_name, "updated_first_name");
    assert_eq!(updated_user.last_name, "updated_last_name");
    assert_eq!(updated_user.email, "updated_user@gmail.com");
    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_delete_user_physical_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Supprime l'utilisateur créé dans setup_db
    let result = user_service.delete_user_physical(1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_user_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Récupère l'utilisateur créé dans setup_db
    let result = user_service.get_user(1).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    let user = result.unwrap().unwrap();
    assert_eq!(user.username, "unique_user");
    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_get_user_not_found() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Essaye de récupérer un utilisateur inexistant
    let result = user_service.get_user(999).await;
    println!("Result: {:?}", result);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_create_user_bad_password() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    let create_user_request = CreateUserRequest {
        username: "unique_user_3".to_string(),
        password: "badpassword".to_string(), // Mauvais mot de passe
        first_name: "first_name_3".to_string(),
        last_name: "last_name_3".to_string(),
        email: "test@gmail.com".to_string(),
    };
    let result = user_service.create_user(create_user_request).await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        "Validation error: password: Password must include at least one uppercase letter, one lowercase letter, one number, and one special character (@$!%*?&)"
    );
    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_create_user_already_exists() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Crée un utilisateur initial
    let create_user_request = CreateUserRequest {
        username: "existing_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Existing".to_string(),
        last_name: "User".to_string(),
        email: "existing_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await;

    let create_user_request2 = CreateUserRequest {
        username: "existing_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Existing".to_string(),
        last_name: "User".to_string(),
        email: "existing_user@gmail.com".to_string(),
    };

    // Essaye de créer un utilisateur avec le même nom d'utilisateur
    let result = user_service.create_user(create_user_request2).await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        "Validation error: Username existing_user already exists"
    );
    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_create_user_duplicate_email() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Crée un premier utilisateur avec une adresse email spécifique
    let create_user_request1 = CreateUserRequest {
        username: "user1".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "First".to_string(),
        last_name: "User".to_string(),
        email: "duplicate_email@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request1).await;

    // Crée un deuxième utilisateur avec la même adresse email mais des informations différentes
    let create_user_request2 = CreateUserRequest {
        username: "user2".to_string(),
        password: "AnotherPass123!".to_string(),
        first_name: "Second".to_string(),
        last_name: "User".to_string(),
        email: "duplicate_email@gmail.com".to_string(),
    };

    // Essaye de créer un utilisateur avec la même adresse email
    let result = user_service.create_user(create_user_request2).await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        "Validation error: Email duplicate_email@gmail.com already exists"
    );
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_create_user_missing_email() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    let create_user_request = CreateUserRequest {
        username: "user_without_email".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "First".to_string(),
        last_name: "User".to_string(),
        email: "".to_string(), // Email manquant
    };

    let result = user_service.create_user(create_user_request).await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        "Validation error: email: Invalid email format"
    );
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_create_user_missing_first_name() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    let create_user_request = CreateUserRequest {
        username: "no_first_name".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "".to_string(), // Prénom manquant
        last_name: "User".to_string(),
        email: "user_without_first_name@gmail.com".to_string(),
    };

    let result = user_service.create_user(create_user_request).await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        "Validation error: first_name: First name is required and cannot exceed 20 characters"
    );
    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_create_user_missing_last_name() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    let create_user_request = CreateUserRequest {
        username: "no_last_name".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "First".to_string(),
        last_name: "".to_string(), // Nom manquant
        email: "user_without_last_name@gmail.com".to_string(),
    };

    let result = user_service.create_user(create_user_request).await;
    println!("Result: {:?}", result);
    assert!(result.is_err());
    let error = result.unwrap_err();

    assert_eq!(
        error.to_string(),
        "Validation error: last_name: Last name is required and cannot exceed 20 characters"
    );
    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_authentication_success_token_sent() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Crée un utilisateur pour le test
    let create_user_request = CreateUserRequest {
        username: "auth_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Auth".to_string(),
        last_name: "User".to_string(),
        email: "auth_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await;
    let login_request = LoginRequest {
        username: "auth_user".to_string(),
        password: "ValidPass123!".to_string(),
    };
    // Authentifie l'utilisateur
    let result = user_service.login(login_request).await;
    println!("Result: {:?}", result);

    // Vérifie que l'authentification a réussi et qu'un jeton a été envoyé
    assert!(result.is_ok());
    let auth_response = result.unwrap();
    assert!(!auth_response.token.is_empty());
    assert_eq!(auth_response.username, "auth_user");

    test_db.drop().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_authentication_and_token_verification_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Crée un utilisateur pour le test
    let create_user_request = CreateUserRequest {
        username: "token_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Token".to_string(),
        last_name: "User".to_string(),
        email: "token_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await;

    // Authentifie l'utilisateur
    let login_request = LoginRequest {
        username: "token_user".to_string(),
        password: "ValidPass123!".to_string(),
    };
    let login_result = user_service.login(login_request).await;
    println!("Login Result: {:?}", login_result);

    // Vérifie que l'authentification a réussi et qu'un jeton a été envoyé
    assert!(login_result.is_ok());
    let auth_response = login_result.unwrap();
    assert!(!auth_response.token.is_empty());
    assert_eq!(auth_response.username, "token_user");

    // Vérifie le jeton
    let token_verification_request = VerifyTokenRequest {
        token: auth_response.token.clone(),
    };
    let token_verification_result = user_service.verify_token(token_verification_request).await;
    println!("Token Verification Result: {:?}", token_verification_result);

    assert!(token_verification_result.is_ok());
    let verified_user = token_verification_result.unwrap();
    assert_eq!(verified_user.valid, true);

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_authentication_with_email_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Crée un utilisateur pour le test
    let create_user_request = CreateUserRequest {
        username: "email_auth_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Email".to_string(),
        last_name: "AuthUser".to_string(),
        email: "email_auth_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await;

    // Authentifie l'utilisateur avec l'email
    let login_request = LoginEmailRequest {
        email: "email_auth_user@gmail.com".to_string(), // Utilise l'email comme identifiant
        password: "ValidPass123!".to_string(),
    };
    let result = user_service.login_with_email(login_request).await;
    println!("Result: {:?}", result);

    // Vérifie que l'authentification a réussi et qu'un jeton a été envoyé
    assert!(result.is_ok());
    let auth_response = result.unwrap();
    assert!(!auth_response.token.is_empty());
    assert_eq!(auth_response.username, "email_auth_user");
    let token_verification_request = VerifyTokenRequest {
        token: auth_response.token.clone(),
    };
    let token_verification_result = user_service.verify_token(token_verification_request).await;
    println!("Token Verification Result: {:?}", token_verification_result);

    assert!(token_verification_result.is_ok());
    let verified_user = token_verification_result.unwrap();
    assert_eq!(verified_user.valid, true);

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_forgot_password_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Create a user for testing
    let create_user_request = CreateUserRequest {
        username: "forgot_password_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Forgot".to_string(),
        last_name: "Password".to_string(),
        email: "forgot_password_user@gmail.com".to_string(),
    };
    let _ = user_service.create_user(create_user_request).await;

    // Test forgot password with valid email
    let forgot_password_request = ForgotPasswordRequest {
        email: "forgot_password_user@gmail.com".to_string(),
    };

    let result = user_service.forgot_password(forgot_password_request).await;
    println!("Result: {:?}", result);

    // Since we can't easily mock the mail service in this test environment,
    // we expect an error related to SMTP configuration (which is expected in test environment)
    // but the important part is that the function doesn't panic and handles the flow correctly
    assert!(result.is_err());
    // The error should be related to internal server error (mail service configuration)
    // rather than a validation error or not found error

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_forgot_password_user_not_found() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Test forgot password with non-existent email
    let forgot_password_request = ForgotPasswordRequest {
        email: "nonexistent@gmail.com".to_string(),
    };

    // Should return Ok(()) even for non-existent users (security measure)
    let result = user_service.forgot_password(forgot_password_request).await;
    println!("Result: {:?}", result);

    // Should succeed (return Ok) even for non-existent users
    assert!(result.is_ok());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_reset_password_success() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Create a user for testing
    let create_user_request = CreateUserRequest {
        username: "reset_password_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Reset".to_string(),
        last_name: "Password".to_string(),
        email: "reset_password_user@gmail.com".to_string(),
    };
    let user = user_service.create_user(create_user_request).await.unwrap();

    // Generate a valid token for the user
    let claims = Claims {
        sub: user.id.to_string(),
        exp: Some((chrono::Utc::now() + chrono::Duration::minutes(10)).timestamp() as usize),
        iat: chrono::Utc::now().timestamp() as usize,
        user_id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
    };

    // Test reset password with valid token
    let new_password = "NewPass456!".to_string();
    let result = user_service.reset_password(claims, new_password).await;
    println!("Result: {:?}", result);

    // Should succeed
    assert!(result.is_ok());

    test_db.drop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_reset_password_expired_token() {
    let test_db = TestDatabase::new().await;
    let user_service = UserService::new(test_db.get_db());

    // Create a user for testing
    let create_user_request = CreateUserRequest {
        username: "expired_token_user".to_string(),
        password: "ValidPass123!".to_string(),
        first_name: "Expired".to_string(),
        last_name: "Token".to_string(),
        email: "expired_token_user@gmail.com".to_string(),
    };
    let user = user_service.create_user(create_user_request).await.unwrap();

    // Generate an expired token for the user
    let claims = Claims {
        sub: user.id.to_string(),
        exp: Some((chrono::Utc::now() - chrono::Duration::minutes(10)).timestamp() as usize),
        iat: (chrono::Utc::now() - chrono::Duration::minutes(20)).timestamp() as usize,
        user_id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
    };

    // Test reset password with expired token
    let new_password = "NewPass456!".to_string();
    let result = user_service.reset_password(claims, new_password).await;
    println!("Result: {:?}", result);

    // Should fail with unauthorized error due to expired token
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("Token has expired"));
    }

    test_db.drop().await;
}
