# Mail Service Implementation Plan

## Overview

This document outlines the implementation plan for adding a mail service to the Swaptun backend using the lettre crate for SMTP email sending.

## Implementation Steps

### 1. Add lettre crate as a dependency

Add the lettre crate to `services/Cargo.toml`:

```toml
lettre = "0.11"
lettre-email = "0.11"  # Optional, for easier email creation
```

### 2. Create mail service module structure

Create the following directory structure:

```
services/src/mail/
├── mod.rs
├── mail_service.rs
└── dto/
    ├── mod.rs
    ├── mail_request.rs
    └── mail_response.rs
```

### 3. Create mail DTOs

Create DTOs for email requests and responses:

- `MailRequest` struct with fields for:
  - to: Vec<String> (recipient email addresses)
  - cc: Option<Vec<String>> (carbon copy recipients)
  - bcc: Option<Vec<String>> (blind carbon copy recipients)
  - from: String (sender email address)
  - subject: String (email subject)
  - body: String (email body content)
  - is_html: bool (whether the body is HTML or plain text)
- `MailResponse` struct with fields for:
  - success: bool
  - message_id: Option<String>
  - error: Option<String>

### 4. Implement mail service

Create `MailService` struct with methods:

- `new()` - Initialize the service with SMTP configuration
- `send_mail()` - Send a single email
- `send_bulk_mail()` - Send multiple emails

### 5. SMTP Configuration

Add the following environment variables to `.env`:

```
SMTP_HOST=your.smtp.server.com
SMTP_PORT=587
SMTP_USERNAME=your_username
SMTP_PASSWORD=your_password
SMTP_FROM_ADDRESS=noreply@yoursite.com
SMTP_FROM_NAME=Your Site Name
```

### 6. Integration with existing notification service

Extend the existing notification service to also send emails, or create a unified notification interface that can handle both FCM and email notifications.

### 7. Update services module

Update `services/src/lib.rs` to export the new mail module.

## Code Structure Example

### Mail Service Implementation

```rust
use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub struct MailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
    from_name: String,
}

impl MailService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let smtp_host = std::env::var("SMTP_HOST")?;
        let smtp_port = std::env::var("SMTP_PORT")?.parse()?;
        let smtp_username = std::env::var("SMTP_USERNAME")?;
        let smtp_password = std::env::var("SMTP_PASSWORD")?;
        let from_address = std::env::var("SMTP_FROM_ADDRESS")?;
        let from_name = std::env::var("SMTP_FROM_NAME")?;

        let credentials = Credentials::new(smtp_username, smtp_password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
            .credentials(credentials)
            .port(smtp_port)
            .build();

        Ok(Self {
            mailer,
            from_address,
            from_name,
        })
    }

    pub async fn send_mail(
        &self,
        to: &[String],
        subject: &str,
        body: &str,
        is_html: bool,
    ) -> Result<MailResponse, Box<dyn std::error::Error>> {
        // Implementation here
    }
}
```

## Testing

Create unit tests for the mail service to ensure it works correctly with different SMTP configurations.
