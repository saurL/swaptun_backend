#[cfg(test)]
mod tests {
    use actix_http::Request;
    use actix_service::Service;
    use actix_web::{dev::ServiceResponse, test, web, App, Error};
    use serde_json;
    use swaptun_api::api;
    use swaptun_services::{
        ForgotPasswordRequest, LoginEmailRequest, LoginRequest, TestDatabase, VerifyTokenRequest,
    };

    // Helper function to authenticate user and get token
    async fn authenticate_user(
        app: &impl Service<Request, Response = ServiceResponse, Error = Error>,
    ) -> String {
        let login_request = LoginRequest {
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
    async fn test_auth_login_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let login_request = LoginRequest {
            username: "unique_user".to_string(),
            password: "hashed_passwor12D!".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_auth_login_invalid_credentials() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let login_request = LoginRequest {
            username: "nonexistent_user".to_string(),
            password: "wrong_password".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_auth_login_email_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let login_request = LoginEmailRequest {
            email: "unique_user@gmail.com".to_string(),
            password: "hashed_passwor12D!".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/login_email")
            .set_json(&login_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_auth_verify_token_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        // First authenticate to get a token
        let token = authenticate_user(&app).await;

        let verify_request = VerifyTokenRequest { token };

        let req = test::TestRequest::post()
            .uri("/api/auth/verify_token")
            .set_json(&verify_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_auth_verify_token_invalid() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let verify_request = VerifyTokenRequest {
            token: "invalid_token".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/verify_token")
            .set_json(&verify_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_auth_forgot_password_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let forgot_request = ForgotPasswordRequest {
            email: "unemailquiexistepas@gmail.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/forgot_password")
            .set_json(&forgot_request)
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Should always return success for security reasons
        println!("Forgot password response status: {:?}", resp);
        println!("Status: {:?}", resp.status());

        assert!(resp.status().is_success());

        test_db.drop().await;
    }

    #[actix_web::test]
    async fn test_auth_forgot_password_nonexistent_email() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        let forgot_request = ForgotPasswordRequest {
            email: "nonexistent@example.com".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/auth/forgot_password")
            .set_json(&forgot_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Should always return success for security reasons
        assert!(resp.status().is_success());

        test_db.drop().await;
    }
}
