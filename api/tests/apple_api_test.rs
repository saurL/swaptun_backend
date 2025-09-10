#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_service::Service;
    use actix_web::{dev::ServiceResponse, test, web, App, Error};
    use serde_json;
    use swaptun_api::api;
    use swaptun_services::TestDatabase;

    // Helper function to authenticate user and get token
    async fn authenticate_user(
        app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    ) -> String {
        let login_request = swaptun_services::LoginRequest {
            username: "unique_user".to_string(),
            password: "hashed_passwor12D!".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let json: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse authentication response as JSON");

        match json.get("token") {
            Some(token) => {
                let token_str = token.as_str().expect("Token value is not a string");
                token_str.to_string()
            }
            None => {
                panic!("Authentication failed: No token received");
            }
        }
    }

    #[actix_web::test]
    async fn test_apple_get_authorization_url_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let token = authenticate_user(&app).await;
        let req = test::TestRequest::get()
            .uri("/api/apple/authorization-url")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("url").is_some());

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_apple_set_token_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let token = authenticate_user(&app).await;
        let token_request = swaptun_services::AddTokenRequest {
            token: "apple_music_auth_code_123".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/apple/token")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&token_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(response, true);

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_apple_get_developer_token_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let token = authenticate_user(&app).await;

        // First set a token
        let token_request = swaptun_services::AddTokenRequest {
            token: "apple_music_auth_code_456".to_string(),
        };

        let set_req = test::TestRequest::post()
            .uri("/api/apple/token")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&token_request)
            .to_request();

        let _ = test::call_service(&app, set_req).await;

        // Now get the developer token
        let req = test::TestRequest::get()
            .uri("/api/apple/developer-token")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("access_token").is_some());
        assert!(json.get("expires_in").is_some());

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_apple_get_developer_token_not_found() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let token = authenticate_user(&app).await;

        // Try to get developer token without setting one first
        let req = test::TestRequest::get()
            .uri("/api/apple/developer-token")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404); // Not Found

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_apple_endpoints_unauthorized() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        // Test without authorization header
        let req = test::TestRequest::get()
            .uri("/api/apple/authorization-url")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401); // Unauthorized

        test_db.drop().await;
    }
}
