use crate::{error::AppError, MailRequest, MailResponse};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use log::{error, info};

pub struct MailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
    from_name: String,
}

impl MailService {
    /// Creates a new instance of the MailService
    pub fn new() -> Result<Self, AppError> {
        let smtp_host = std::env::var("SMTP_HOST").map_err(|e| {
            error!("SMTP_HOST environment variable not set: {}", e);
            AppError::InternalServerError
        })?;
        let smtp_port = std::env::var("SMTP_PORT")
            .map_err(|e| {
                error!("SMTP_PORT environment variable not set: {}", e);
                AppError::InternalServerError
            })?
            .parse::<u16>()
            .map_err(|e| {
                error!("SMTP_PORT must be a valid number: {}", e);
                AppError::InternalServerError
            })?;
        let smtp_username = std::env::var("SMTP_USERNAME").map_err(|e| {
            error!("SMTP_USERNAME environment variable not set: {}", e);
            AppError::InternalServerError
        })?;
        let smtp_password = std::env::var("SMTP_PASSWORD").map_err(|e| {
            error!("SMTP_PASSWORD environment variable not set: {}", e);
            AppError::InternalServerError
        })?;
        let from_address = std::env::var("SMTP_FROM_ADDRESS").map_err(|e| {
            error!("SMTP_FROM_ADDRESS environment variable not set: {}", e);
            AppError::InternalServerError
        })?;
        let from_name = std::env::var("SMTP_FROM_NAME").map_err(|e| {
            error!("SMTP_FROM_NAME environment variable not set: {}", e);
            AppError::InternalServerError
        })?;

        let credentials = Credentials::new(smtp_username, smtp_password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
            .map_err(|e| {
                error!("Failed to create SMTP transport: {}", e);
                AppError::InternalServerError
            })?
            .credentials(credentials)
            .port(smtp_port)
            .build();

        Ok(Self {
            mailer,
            from_address,
            from_name,
        })
    }

    /// Sends a single email
    pub async fn send_mail(
        &self,
        request: MailRequest,
    ) -> Result<MailResponse, Box<dyn std::error::Error>> {
        // Create the email message
        let mut email_builder = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_address).parse()?)
            .subject(&request.subject);

        // Add recipients
        for recipient in &request.to {
            email_builder = email_builder.to(recipient.parse()?);
        }

        if let Some(cc_recipients) = &request.cc {
            for recipient in cc_recipients {
                email_builder = email_builder.cc(recipient.parse()?);
            }
        }

        if let Some(bcc_recipients) = &request.bcc {
            for recipient in bcc_recipients {
                email_builder = email_builder.bcc(recipient.parse()?);
            }
        }

        // Set content type based on is_html flag
        let email = if request.is_html {
            email_builder
                .header(ContentType::TEXT_HTML)
                .body(request.body)?
        } else {
            email_builder
                .header(ContentType::TEXT_PLAIN)
                .body(request.body)?
        };

        // Send the email
        match self.mailer.send(email).await {
            Ok(_) => {
                info!("Email sent successfully");
                Ok(MailResponse {
                    success: true,
                    message_id: None,
                    error: None,
                })
            }
            Err(e) => {
                error!("Failed to send email: {}", e);
                Ok(MailResponse {
                    success: false,
                    message_id: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}
