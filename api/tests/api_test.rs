#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_service::Service;
    use actix_web::{App, Error, dev::ServiceResponse, test, web};
    use serde_json;
    use swaptun_api::api;
    use swaptun_services::{
        TestDatabase,
        dto::{CreateUserRequest, LoginRequest, UpdateUserRequest},
    };

    // Helper function pour obtenir un token JWT
    async fn authenticate_user(
        app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    ) -> String {
        println!("Authenticating with default test user");
        let login_request = LoginRequest {
            username: "unique_user".to_string(),
            password: "hashed_passwor12D!".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("Authentication response status: {}", resp.status());

        // Read response body
        let body = test::read_body(resp).await;
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse authentication response as JSON");

        match json.get("token") {
            Some(token) => {
                let token_str = token.as_str().expect("Token value is not a string");
                println!("Received token: {}", token_str);
                token_str.to_string()
            }
            None => {
                println!("No token found in authentication response");
                println!("Response body: {}", String::from_utf8_lossy(&body));
                panic!("Authentication failed: No token received");
            }
        }
    }

    #[actix_web::test]
    async fn test_create_user_success() {
        println!("Starting test_create_user_success");
        let test_db = TestDatabase::new().await;
        println!("Test database created");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;
        println!("Test service initialized");

        let create_user_request = CreateUserRequest {
            username: "test_user".to_string(),
            password: "TestPass123!".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test.user@example.com".to_string(),
        };
        println!(
            "Creating user with username: {}",
            create_user_request.username
        );

        let req = test::TestRequest::post()
            .uri("/api/register")
            .set_json(&create_user_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("Create user response status: {}", resp.status());
        assert!(resp.status().is_success(), "Failed to create user");

        test_db.drop().await;
        println!("Test completed successfully");
    }

    #[actix_web::test]
    async fn test_login_success() {
        println!("Starting test_login_success");
        let test_db = TestDatabase::new().await;
        println!("Test database created");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;
        println!("Test service initialized");

        // Try to authenticate and get token
        let token = authenticate_user(&app).await;
        assert!(!token.is_empty(), "Token should not be empty");
        println!("Successfully authenticated and received token");

        test_db.drop().await;
        println!("Test completed successfully");
    }

    #[actix_web::test]
    async fn test_get_user_profile() {
        println!("Starting test_get_user_profile");
        let test_db = TestDatabase::new().await;
        println!("Test database created");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;
        println!("Test service initialized");

        // Authentification avec l'utilisateur par défaut
        let token = authenticate_user(&app).await;

        // Get the user profile
        println!("Attempting to get user profile");
        let req = test::TestRequest::get()
            .uri("/api/users/me")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("Get profile response status: {}", resp.status());

        if !resp.status().is_success() {
            let body = test::read_body(resp).await;
            println!("Get profile response body: {:?}", body);
            assert!(false, "Failed to get user profile");
        } else {
            assert!(resp.status().is_success());
        }

        test_db.drop().await;
        println!("Test completed successfully");
    }

    #[actix_web::test]
    async fn test_update_user_profile() {
        println!("Starting test_update_user_profile");
        let test_db = TestDatabase::new().await;
        println!("Test database created");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;
        println!("Test service initialized");

        // Authentification avec l'utilisateur par défaut
        let token = authenticate_user(&app).await;

        // Update the user profile
        let update_request = UpdateUserRequest {
            username: Some("updated_test_user".to_string()),
            first_name: Some("Updated".to_string()),
            last_name: Some("User".to_string()),
            email: Some("updated.test@example.com".to_string()),
        };
        println!(
            "Attempting to update user profile with new username: {:?}",
            update_request.username
        );

        let req = test::TestRequest::put()
            .uri("/api/users/me")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&update_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        println!("Update profile response status: {}", resp.status());

        if !resp.status().is_success() {
            let body = test::read_body(resp).await;
            println!("Update profile response body: {:?}", body);
            assert!(false, "Failed to update user profile");
        } else {
            assert!(resp.status().is_success());
        }

        test_db.drop().await;
        println!("Test completed successfully");
    }
}
