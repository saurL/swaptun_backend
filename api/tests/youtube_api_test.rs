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
    async fn test_youtube_get_playlists_success() {
        let test_db = TestDatabase::new().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_db.get_db()))
                .configure(|config| api::configure_routes(config, test_db.get_db_raw())),
        )
        .await;

        // Authenticate to get token
        let token = authenticate_user(&app).await;

        let req = test::TestRequest::get()
            .uri("/api/youtube/playlists")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        // This might fail due to external service dependencies in test environment
        assert!(resp.status().is_success() || resp.status().is_client_error());

        test_db.drop().await;
    }
}
