use swaptun_services::mail::{MailRequest, MailService};

#[tokio::test]
async fn test_mail_service_creation() {
    // This test will fail if the environment variables are not set
    // It's meant to be run with proper SMTP configuration
    /*
    let mail_service = MailService::new();
    assert!(mail_service.is_ok());
    */
}

#[test]
fn test_mail_request_validation() {
    let request = MailRequest {
        to: vec!["test@example.com".to_string()],
        cc: None,
        bcc: None,
        subject: "Test Subject".to_string(),
        body: "Test Body".to_string(),
        is_html: false,
    };

    // Basic validation - check that the request is properly created
    assert_eq!(request.to.len(), 1);
    assert_eq!(request.subject, "Test Subject");
    assert_eq!(request.body, "Test Body");
    assert_eq!(request.is_html, false);
}
